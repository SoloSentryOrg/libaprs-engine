//! Protocol-first APRS engine core primitives.
//!
//! The codec boundary accepts untrusted bytes, preserves them exactly, and
//! fails closed when the packet shape is malformed.

/// Conservative upper bound for an APRS packet handled by this skeleton.
pub const MAX_PACKET_LEN: usize = 512;

/// Original packet bytes retained without normalization or lossy conversion.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RawPacket {
    bytes: Vec<u8>,
}

impl RawPacket {
    /// Returns the original packet bytes exactly as supplied to the parser.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

/// Structured packet view backed by the preserved raw packet bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedPacket {
    raw: RawPacket,
    source_end: usize,
    path_start: usize,
    path_end: usize,
    payload_start: usize,
}

impl ParsedPacket {
    /// Returns the preserved raw packet.
    #[must_use]
    pub fn raw(&self) -> &RawPacket {
        &self.raw
    }

    /// Returns the source callsign bytes before the `>` separator.
    #[must_use]
    pub fn source(&self) -> &[u8] {
        &self.raw.bytes[..self.source_end]
    }

    /// Returns the destination/path bytes between `>` and `:`.
    #[must_use]
    pub fn path(&self) -> &[u8] {
        &self.raw.bytes[self.path_start..self.path_end]
    }

    /// Returns the payload bytes after the `:` separator.
    #[must_use]
    pub fn payload(&self) -> &[u8] {
        &self.raw.bytes[self.payload_start..]
    }
}

/// Fail-closed packet parse errors.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParseError {
    /// No bytes were supplied.
    Empty,
    /// Packet exceeds [`MAX_PACKET_LEN`].
    Oversized,
    /// Packet does not contain the required APRS `>` and `:` separators.
    MissingSeparator,
    /// Packet contains an empty source, path, or payload segment.
    EmptySegment,
    /// Packet source or path contains bytes outside the conservative address set.
    InvalidAddress,
}

/// Parses an APRS packet from untrusted bytes.
///
/// This parser intentionally validates only the minimal frame shape for the
/// skeleton: `source>path:payload`. Payload bytes are opaque and may be invalid
/// UTF-8.
pub fn parse_packet(input: &[u8]) -> Result<ParsedPacket, ParseError> {
    if input.is_empty() {
        return Err(ParseError::Empty);
    }

    if input.len() > MAX_PACKET_LEN {
        return Err(ParseError::Oversized);
    }

    let source_end = input
        .iter()
        .position(|byte| *byte == b'>')
        .ok_or(ParseError::MissingSeparator)?;
    let payload_separator = input[source_end + 1..]
        .iter()
        .position(|byte| *byte == b':')
        .map(|offset| source_end + 1 + offset)
        .ok_or(ParseError::MissingSeparator)?;

    let path_start = source_end + 1;
    let path_end = payload_separator;
    let payload_start = payload_separator + 1;

    if source_end == 0 || path_start == path_end || payload_start == input.len() {
        return Err(ParseError::EmptySegment);
    }

    if !is_ax25_like_address(&input[..source_end]) || !is_ax25_like_path(&input[path_start..path_end]) {
        return Err(ParseError::InvalidAddress);
    }

    Ok(ParsedPacket {
        raw: RawPacket {
            bytes: input.to_vec(),
        },
        source_end,
        path_start,
        path_end,
        payload_start,
    })
}

fn is_ax25_like_path(path: &[u8]) -> bool {
    path.split(|byte| *byte == b',')
        .all(is_ax25_like_address)
}

fn is_ax25_like_address(address: &[u8]) -> bool {
    !address.is_empty()
        && address
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'*'))
}
