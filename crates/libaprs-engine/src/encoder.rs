//! Owned-byte APRS packet construction helpers.
//!
//! Encoders validate conservative packet shape before emitting bytes. They do
//! not transmit, log, normalize, or display packet contents.

use crate::{
    is_ax25_like_path_component, is_ax25_like_source, is_latitude, is_longitude,
    is_printable_ascii, is_symbol_table_identifier, is_timestamp, MAX_PACKET_LEN,
};

/// APRS encoder failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EncodeError {
    /// The path list is empty.
    EmptyPath,
    /// Source or path address bytes are not conservative AX.25-like metadata.
    InvalidAddress,
    /// Source or path address bytes contain lowercase ASCII.
    LowercaseAddress,
    /// A payload or semantic field is invalid for the selected packet type.
    InvalidField,
    /// Packet payload is empty.
    EmptyPayload,
    /// Encoded packet would exceed [`MAX_PACKET_LEN`].
    OversizedPacket,
}

impl EncodeError {
    /// Stable machine-readable error code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::EmptyPath => "encode.empty_path",
            Self::InvalidAddress => "encode.invalid_address",
            Self::LowercaseAddress => "encode.lowercase_address",
            Self::InvalidField => "encode.invalid_field",
            Self::EmptyPayload => "encode.empty_payload",
            Self::OversizedPacket => "encode.oversized_packet",
        }
    }
}

impl std::fmt::Display for EncodeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.code())
    }
}

impl std::error::Error for EncodeError {}

/// Uncompressed position fields for packet construction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UncompressedPositionEncoding<'a> {
    /// Whether to use the APRS messaging-capable data type identifier.
    pub messaging: bool,
    /// Latitude bytes in APRS `DDMM.mmN/S` form.
    pub latitude: &'a [u8],
    /// Symbol table identifier byte.
    pub symbol_table: u8,
    /// Longitude bytes in APRS `DDDMM.mmE/W` form.
    pub longitude: &'a [u8],
    /// Symbol code byte.
    pub symbol_code: u8,
    /// Optional comment bytes after the symbol code.
    pub comment: &'a [u8],
}

/// Object fields for packet construction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ObjectEncoding<'a> {
    /// Nine-byte object name.
    pub name: &'a [u8],
    /// Whether the object is live (`*`) rather than killed (`_`).
    pub live: bool,
    /// Seven-byte object timestamp.
    pub timestamp: &'a [u8],
    /// Object body bytes, usually a position and comment.
    pub body: &'a [u8],
}

/// Item fields for packet construction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ItemEncoding<'a> {
    /// Item name bytes, one through nine printable bytes.
    pub name: &'a [u8],
    /// Whether the item is live (`!`) rather than killed (`_`).
    pub live: bool,
    /// Item body bytes, usually a position and comment.
    pub body: &'a [u8],
}

/// Telemetry metadata packet kind for packet construction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TelemetryMetadataEncodingKind {
    /// `PARM.` telemetry parameter names.
    Parameters,
    /// `UNIT.` telemetry unit names.
    Units,
    /// `EQNS.` telemetry equation coefficients.
    Equations,
    /// `BITS.` telemetry bit labels and project title.
    BitSense,
}

impl TelemetryMetadataEncodingKind {
    fn addressee(self) -> &'static [u8; 9] {
        match self {
            Self::Parameters => b"PARM.    ",
            Self::Units => b"UNIT.    ",
            Self::Equations => b"EQNS.    ",
            Self::BitSense => b"BITS.    ",
        }
    }
}

/// Encodes an arbitrary APRS packet payload into `source>path:payload` bytes.
pub fn encode_packet(
    source: &[u8],
    path: &[&[u8]],
    payload: &[u8],
) -> Result<Vec<u8>, EncodeError> {
    if payload.is_empty() {
        return Err(EncodeError::EmptyPayload);
    }
    ensure_packet_shape(source, path, payload.len())?;

    let mut encoded = Vec::with_capacity(packet_len(source, path, payload.len())?);
    encoded.extend_from_slice(source);
    encoded.push(b'>');
    encoded.extend_from_slice(path[0]);
    for component in &path[1..] {
        encoded.push(b',');
        encoded.extend_from_slice(component);
    }
    encoded.push(b':');
    encoded.extend_from_slice(payload);

    Ok(encoded)
}

/// Encodes a status packet.
pub fn encode_status(source: &[u8], path: &[&[u8]], text: &[u8]) -> Result<Vec<u8>, EncodeError> {
    ensure_packet_shape(source, path, 1usize.saturating_add(text.len()))?;
    let mut payload = Vec::with_capacity(text.len().saturating_add(1));
    payload.push(b'>');
    payload.extend_from_slice(text);
    encode_packet(source, path, &payload)
}

/// Encodes an uncompressed position packet.
pub fn encode_uncompressed_position(
    source: &[u8],
    path: &[&[u8]],
    position: UncompressedPositionEncoding<'_>,
) -> Result<Vec<u8>, EncodeError> {
    if !is_latitude(position.latitude)
        || !is_symbol_table_identifier(position.symbol_table)
        || !is_longitude(position.longitude)
        || !is_printable_ascii(position.symbol_code)
    {
        return Err(EncodeError::InvalidField);
    }

    ensure_packet_shape(source, path, 20usize.saturating_add(position.comment.len()))?;
    let mut payload = Vec::with_capacity(20usize.saturating_add(position.comment.len()));
    payload.push(if position.messaging { b'=' } else { b'!' });
    payload.extend_from_slice(position.latitude);
    payload.push(position.symbol_table);
    payload.extend_from_slice(position.longitude);
    payload.push(position.symbol_code);
    payload.extend_from_slice(position.comment);
    encode_packet(source, path, &payload)
}

/// Encodes an APRS message, acknowledgement, rejection, bulletin, or
/// announcement payload.
pub fn encode_message(
    source: &[u8],
    path: &[&[u8]],
    addressee: &[u8],
    text: &[u8],
    id: Option<&[u8]>,
) -> Result<Vec<u8>, EncodeError> {
    if !is_fixed_printable(addressee, 9)
        || id.is_some_and(|message_id| !is_valid_message_id(message_id))
    {
        return Err(EncodeError::InvalidField);
    }

    let id_len = id.map_or(0, |message_id| 1usize.saturating_add(message_id.len()));
    let payload_len = 11usize.saturating_add(text.len()).saturating_add(id_len);
    ensure_packet_shape(source, path, payload_len)?;

    let mut payload = Vec::with_capacity(payload_len);
    payload.push(b':');
    payload.extend_from_slice(addressee);
    payload.push(b':');
    payload.extend_from_slice(text);
    if let Some(id) = id {
        payload.push(b'{');
        payload.extend_from_slice(id);
    }

    encode_packet(source, path, &payload)
}

/// Encodes a message acknowledgement packet.
pub fn encode_ack(
    source: &[u8],
    path: &[&[u8]],
    addressee: &[u8],
    message_id: &[u8],
) -> Result<Vec<u8>, EncodeError> {
    encode_message_response(source, path, addressee, b"ack", message_id)
}

/// Encodes a message rejection packet.
pub fn encode_reject(
    source: &[u8],
    path: &[&[u8]],
    addressee: &[u8],
    message_id: &[u8],
) -> Result<Vec<u8>, EncodeError> {
    encode_message_response(source, path, addressee, b"rej", message_id)
}

/// Encodes a bulletin packet with a one-byte numeric bulletin identifier.
pub fn encode_bulletin(
    source: &[u8],
    path: &[&[u8]],
    bulletin_id: u8,
    text: &[u8],
) -> Result<Vec<u8>, EncodeError> {
    if !bulletin_id.is_ascii_digit() {
        return Err(EncodeError::InvalidField);
    }

    let mut addressee = *b"BLN      ";
    addressee[3] = bulletin_id;
    encode_message(source, path, &addressee, text, None)
}

/// Encodes an announcement packet with a one-byte uppercase announcement
/// identifier.
pub fn encode_announcement(
    source: &[u8],
    path: &[&[u8]],
    announcement_id: u8,
    text: &[u8],
) -> Result<Vec<u8>, EncodeError> {
    if !announcement_id.is_ascii_uppercase() {
        return Err(EncodeError::InvalidField);
    }

    let mut addressee = *b"BLN      ";
    addressee[3] = announcement_id;
    encode_message(source, path, &addressee, text, None)
}

/// Encodes a telemetry report packet.
pub fn encode_telemetry(
    source: &[u8],
    path: &[&[u8]],
    sequence: u16,
    analog: [u16; 5],
    digital: Option<[bool; 8]>,
) -> Result<Vec<u8>, EncodeError> {
    let payload_len = if digital.is_some() { 34 } else { 25 };
    ensure_packet_shape(source, path, payload_len)?;

    let mut payload = Vec::with_capacity(payload_len);
    payload.extend_from_slice(b"T#");
    push_three_digits(&mut payload, sequence)?;
    for value in analog {
        payload.push(b',');
        push_three_digits(&mut payload, value)?;
    }
    if let Some(digital) = digital {
        payload.push(b',');
        for bit in digital {
            payload.push(if bit { b'1' } else { b'0' });
        }
    }

    encode_packet(source, path, &payload)
}

/// Encodes a telemetry metadata packet.
pub fn encode_telemetry_metadata(
    source: &[u8],
    path: &[&[u8]],
    kind: TelemetryMetadataEncodingKind,
    body: &[u8],
) -> Result<Vec<u8>, EncodeError> {
    if body.is_empty() {
        return Err(EncodeError::InvalidField);
    }

    encode_message(source, path, kind.addressee(), body, None)
}

/// Encodes an object packet.
pub fn encode_object(
    source: &[u8],
    path: &[&[u8]],
    object: ObjectEncoding<'_>,
) -> Result<Vec<u8>, EncodeError> {
    if !is_fixed_printable(object.name, 9)
        || !is_timestamp(object.timestamp)
        || object.body.is_empty()
    {
        return Err(EncodeError::InvalidField);
    }

    ensure_packet_shape(source, path, 18usize.saturating_add(object.body.len()))?;
    let mut payload = Vec::with_capacity(18usize.saturating_add(object.body.len()));
    payload.push(b';');
    payload.extend_from_slice(object.name);
    payload.push(if object.live { b'*' } else { b'_' });
    payload.extend_from_slice(object.timestamp);
    payload.extend_from_slice(object.body);
    encode_packet(source, path, &payload)
}

/// Encodes an item packet.
pub fn encode_item(
    source: &[u8],
    path: &[&[u8]],
    item: ItemEncoding<'_>,
) -> Result<Vec<u8>, EncodeError> {
    if item.name.is_empty()
        || item.name.len() > 9
        || !item
            .name
            .iter()
            .all(|byte| is_printable_ascii(*byte) && !matches!(*byte, b'!' | b'_'))
        || item.body.is_empty()
    {
        return Err(EncodeError::InvalidField);
    }

    ensure_packet_shape(
        source,
        path,
        2usize.saturating_add(item.name.len().saturating_add(item.body.len())),
    )?;
    let mut payload =
        Vec::with_capacity(2usize.saturating_add(item.name.len().saturating_add(item.body.len())));
    payload.push(b')');
    payload.extend_from_slice(item.name);
    payload.push(if item.live { b'!' } else { b'_' });
    payload.extend_from_slice(item.body);
    encode_packet(source, path, &payload)
}

fn validate_addresses(source: &[u8], path: &[&[u8]]) -> Result<(), EncodeError> {
    if path.is_empty() {
        return Err(EncodeError::EmptyPath);
    }

    if source.iter().any(u8::is_ascii_lowercase)
        || path
            .iter()
            .any(|component| component.iter().any(u8::is_ascii_lowercase))
    {
        return Err(EncodeError::LowercaseAddress);
    }

    if !is_ax25_like_source(source)
        || !path
            .iter()
            .all(|component| is_ax25_like_path_component(component))
    {
        return Err(EncodeError::InvalidAddress);
    }

    Ok(())
}

fn ensure_packet_shape(
    source: &[u8],
    path: &[&[u8]],
    payload_len: usize,
) -> Result<(), EncodeError> {
    validate_addresses(source, path)?;
    if packet_len(source, path, payload_len)? > MAX_PACKET_LEN {
        return Err(EncodeError::OversizedPacket);
    }
    Ok(())
}

fn packet_len(source: &[u8], path: &[&[u8]], payload_len: usize) -> Result<usize, EncodeError> {
    let path_len = path.iter().try_fold(0usize, |accumulator, component| {
        accumulator.checked_add(component.len())
    });
    let Some(path_len) = path_len else {
        return Err(EncodeError::OversizedPacket);
    };

    source
        .len()
        .checked_add(1)
        .and_then(|len| len.checked_add(path_len))
        .and_then(|len| len.checked_add(path.len().saturating_sub(1)))
        .and_then(|len| len.checked_add(1))
        .and_then(|len| len.checked_add(payload_len))
        .ok_or(EncodeError::OversizedPacket)
}

fn is_fixed_printable(value: &[u8], len: usize) -> bool {
    value.len() == len && value.iter().all(|byte| is_printable_ascii(*byte))
}

fn is_valid_message_id(value: &[u8]) -> bool {
    (1..=5).contains(&value.len())
        && value
            .iter()
            .all(|byte| is_printable_ascii(*byte) && *byte != b'{')
}

fn encode_message_response(
    source: &[u8],
    path: &[&[u8]],
    addressee: &[u8],
    prefix: &[u8; 3],
    message_id: &[u8],
) -> Result<Vec<u8>, EncodeError> {
    if !is_valid_message_id(message_id) {
        return Err(EncodeError::InvalidField);
    }

    let mut text = Vec::with_capacity(3usize.saturating_add(message_id.len()));
    text.extend_from_slice(prefix);
    text.extend_from_slice(message_id);
    encode_message(source, path, addressee, &text, None)
}

fn push_three_digits(output: &mut Vec<u8>, value: u16) -> Result<(), EncodeError> {
    if value > 999 {
        return Err(EncodeError::InvalidField);
    }

    output.push(b'0' + u8::try_from(value / 100).map_err(|_| EncodeError::InvalidField)?);
    output.push(b'0' + u8::try_from((value / 10) % 10).map_err(|_| EncodeError::InvalidField)?);
    output.push(b'0' + u8::try_from(value % 10).map_err(|_| EncodeError::InvalidField)?);
    Ok(())
}
