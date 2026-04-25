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
    path_components: Vec<(usize, usize)>,
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

    /// Returns the destination bytes, which are the first path component.
    #[must_use]
    pub fn destination(&self) -> &[u8] {
        let (start, end) = self.path_components[0];
        &self.raw.bytes[start..end]
    }

    /// Returns digipeater path component byte views after the destination.
    #[must_use]
    pub fn digipeaters(&self) -> Vec<&[u8]> {
        self.path_components[1..]
            .iter()
            .map(|(start, end)| &self.raw.bytes[*start..*end])
            .collect()
    }

    /// Returns all path component byte views, including destination first.
    #[must_use]
    pub fn path_components(&self) -> Vec<&[u8]> {
        self.path_components
            .iter()
            .map(|(start, end)| &self.raw.bytes[*start..*end])
            .collect()
    }

    /// Returns the payload bytes after the `:` separator.
    #[must_use]
    pub fn payload(&self) -> &[u8] {
        &self.raw.bytes[self.payload_start..]
    }

    /// Returns the APRS data type identifier from the first payload byte.
    #[must_use]
    pub fn data_type_identifier(&self) -> DataTypeIdentifier {
        DataTypeIdentifier::from_byte(self.raw.bytes[self.payload_start])
    }

    /// Returns payload bytes after the data type identifier.
    #[must_use]
    pub fn information(&self) -> &[u8] {
        &self.raw.bytes[self.payload_start + 1..]
    }

    /// Returns a semantic view of the APRS information field.
    #[must_use]
    pub fn aprs_data(&self) -> AprsData<'_> {
        parse_aprs_data(self.data_type_identifier(), self.information())
    }
}

/// Semantic APRS information-field data.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AprsData<'a> {
    /// Status report.
    Status {
        /// Status text bytes.
        text: &'a [u8],
    },
    /// Uncompressed position report.
    Position(Position<'a>),
    /// Message, bulletin, or announcement.
    Message(Message<'a>),
    /// Object report.
    Object(Object<'a>),
    /// Item report.
    Item(Item<'a>),
    /// Data format is validly framed but not implemented yet.
    Unsupported {
        /// Original data type identifier byte.
        identifier: u8,
        /// Remaining information-field bytes.
        information: &'a [u8],
    },
    /// Data type is known, but its information bytes are malformed.
    Malformed {
        /// Original data type identifier byte.
        identifier: u8,
        /// Remaining information-field bytes.
        information: &'a [u8],
    },
}

/// Uncompressed APRS position fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Position<'a> {
    /// Whether the data type identifier indicates APRS messaging support.
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

/// APRS message fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Message<'a> {
    /// Nine-byte addressee field.
    pub addressee: &'a [u8],
    /// Message text bytes before an optional message ID.
    pub text: &'a [u8],
    /// Optional message ID bytes after `{`.
    pub id: Option<&'a [u8]>,
}

/// APRS object report fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Object<'a> {
    /// Nine-byte object name.
    pub name: &'a [u8],
    /// Whether the object is live (`*`) rather than killed (`_`).
    pub live: bool,
    /// Seven-byte object timestamp.
    pub timestamp: &'a [u8],
    /// Remaining object body bytes.
    pub body: &'a [u8],
}

/// APRS item report fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Item<'a> {
    /// Item name bytes.
    pub name: &'a [u8],
    /// Whether the item is live (`!`) rather than killed (`_`).
    pub live: bool,
    /// Remaining item body bytes.
    pub body: &'a [u8],
}

/// APRS data type identifier from the first payload byte.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DataTypeIdentifier {
    /// `!`: position without timestamp, no APRS messaging.
    PositionNoTimestamp,
    /// `=`: position without timestamp, APRS messaging supported.
    PositionNoTimestampMessaging,
    /// `/`: position with timestamp, no APRS messaging.
    PositionWithTimestamp,
    /// `@`: position with timestamp, APRS messaging supported.
    PositionWithTimestampMessaging,
    /// `>`: status.
    Status,
    /// `:`: message, bulletin, or announcement.
    Message,
    /// `;`: object.
    Object,
    /// `)`: item.
    Item,
    /// `_`: weather report without position.
    Weather,
    /// Any currently unclassified identifier byte.
    Unknown(u8),
}

impl DataTypeIdentifier {
    fn from_byte(byte: u8) -> Self {
        match byte {
            b'!' => Self::PositionNoTimestamp,
            b'=' => Self::PositionNoTimestampMessaging,
            b'/' => Self::PositionWithTimestamp,
            b'@' => Self::PositionWithTimestampMessaging,
            b'>' => Self::Status,
            b':' => Self::Message,
            b';' => Self::Object,
            b')' => Self::Item,
            b'_' => Self::Weather,
            other => Self::Unknown(other),
        }
    }

    fn as_byte(self) -> u8 {
        match self {
            Self::PositionNoTimestamp => b'!',
            Self::PositionNoTimestampMessaging => b'=',
            Self::PositionWithTimestamp => b'/',
            Self::PositionWithTimestampMessaging => b'@',
            Self::Status => b'>',
            Self::Message => b':',
            Self::Object => b';',
            Self::Item => b')',
            Self::Weather => b'_',
            Self::Unknown(value) => value,
        }
    }
}

fn parse_aprs_data(identifier: DataTypeIdentifier, information: &[u8]) -> AprsData<'_> {
    match identifier {
        DataTypeIdentifier::Status => AprsData::Status { text: information },
        DataTypeIdentifier::PositionNoTimestamp => parse_position(false, b'!', information),
        DataTypeIdentifier::PositionNoTimestampMessaging => parse_position(true, b'=', information),
        DataTypeIdentifier::Message => parse_message(information),
        DataTypeIdentifier::Object => parse_object(information),
        DataTypeIdentifier::Item => parse_item(information),
        other => AprsData::Unsupported {
            identifier: other.as_byte(),
            information,
        },
    }
}

fn parse_position<'a>(messaging: bool, identifier: u8, information: &'a [u8]) -> AprsData<'a> {
    if information.len() < 18 {
        return AprsData::Malformed {
            identifier,
            information,
        };
    }

    let latitude = &information[..8];
    let symbol_table = information[8];
    let longitude = &information[9..18];
    let symbol_code = information[18];
    let comment = &information[19..];

    if !is_latitude(latitude)
        || !is_symbol_table_identifier(symbol_table)
        || !is_longitude(longitude)
        || !is_printable_ascii(symbol_code)
    {
        return AprsData::Malformed {
            identifier,
            information,
        };
    }

    AprsData::Position(Position {
        messaging,
        latitude,
        symbol_table,
        longitude,
        symbol_code,
        comment,
    })
}

fn parse_object(information: &[u8]) -> AprsData<'_> {
    if information.len() < 17 || !matches!(information[9], b'*' | b'_') {
        return AprsData::Malformed {
            identifier: b';',
            information,
        };
    }

    AprsData::Object(Object {
        name: &information[..9],
        live: information[9] == b'*',
        timestamp: &information[10..17],
        body: &information[17..],
    })
}

fn parse_item(information: &[u8]) -> AprsData<'_> {
    let Some(separator) = information.iter().position(|byte| matches!(*byte, b'!' | b'_')) else {
        return AprsData::Malformed {
            identifier: b')',
            information,
        };
    };

    if separator == 0 || separator > 9 {
        return AprsData::Malformed {
            identifier: b')',
            information,
        };
    }

    AprsData::Item(Item {
        name: &information[..separator],
        live: information[separator] == b'!',
        body: &information[separator + 1..],
    })
}

fn parse_message(information: &[u8]) -> AprsData<'_> {
    if information.len() < 10 || information[9] != b':' {
        return AprsData::Malformed {
            identifier: b':',
            information,
        };
    }

    let addressee = &information[..9];
    let body = &information[10..];
    let (text, id) = match body.iter().position(|byte| *byte == b'{') {
        Some(separator) => (&body[..separator], Some(&body[separator + 1..])),
        None => (body, None),
    };

    AprsData::Message(Message {
        addressee,
        text,
        id,
    })
}

fn is_latitude(value: &[u8]) -> bool {
    value.len() == 8
        && value[0].is_ascii_digit()
        && value[1].is_ascii_digit()
        && value[2].is_ascii_digit()
        && value[3].is_ascii_digit()
        && value[4] == b'.'
        && value[5].is_ascii_digit()
        && value[6].is_ascii_digit()
        && matches!(value[7], b'N' | b'S')
}

fn is_longitude(value: &[u8]) -> bool {
    value.len() == 9
        && value[0].is_ascii_digit()
        && value[1].is_ascii_digit()
        && value[2].is_ascii_digit()
        && value[3].is_ascii_digit()
        && value[4].is_ascii_digit()
        && value[5] == b'.'
        && value[6].is_ascii_digit()
        && value[7].is_ascii_digit()
        && matches!(value[8], b'E' | b'W')
}

fn is_symbol_table_identifier(value: u8) -> bool {
    matches!(value, b'/' | b'\\') || value.is_ascii_alphanumeric()
}

fn is_printable_ascii(value: u8) -> bool {
    (0x20..=0x7e).contains(&value)
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

    let Some(path_components) = path_component_ranges(input, path_start, path_end) else {
        return Err(ParseError::InvalidAddress);
    };

    if !is_ax25_like_source(&input[..source_end])
        || !path_components
            .iter()
            .all(|(start, end)| is_ax25_like_path_component(&input[*start..*end]))
    {
        return Err(ParseError::InvalidAddress);
    }

    Ok(ParsedPacket {
        raw: RawPacket {
            bytes: input.to_vec(),
        },
        source_end,
        path_start,
        path_end,
        path_components,
        payload_start,
    })
}

fn path_component_ranges(input: &[u8], path_start: usize, path_end: usize) -> Option<Vec<(usize, usize)>> {
    let mut components = Vec::new();
    let mut component_start = path_start;

    for (offset, byte) in input[path_start..path_end].iter().enumerate() {
        if *byte == b',' {
            let index = path_start + offset;
            if component_start == index {
                return None;
            }
            components.push((component_start, index));
            component_start = index + 1;
        }
    }

    if component_start == path_end {
        return None;
    }

    components.push((component_start, path_end));
    Some(components)
}

fn is_ax25_like_source(source: &[u8]) -> bool {
    is_ax25_like_address(source, false)
}

fn is_ax25_like_path_component(component: &[u8]) -> bool {
    is_ax25_like_address(component, true)
}

fn is_ax25_like_address(address: &[u8], allow_repeated_marker: bool) -> bool {
    let address = if allow_repeated_marker {
        address.strip_suffix(b"*").unwrap_or(address)
    } else {
        address
    };

    if address.is_empty() || address.contains(&b'*') {
        return false;
    }

    let (callsign, ssid) = match address.iter().position(|byte| *byte == b'-') {
        Some(separator) => (&address[..separator], Some(&address[separator + 1..])),
        None => (address, None),
    };

    is_ax25_like_callsign(callsign) && ssid.is_none_or(is_ax25_like_ssid)
}

fn is_ax25_like_callsign(callsign: &[u8]) -> bool {
    (1..=6).contains(&callsign.len())
        && callsign
            .iter()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())
}

fn is_ax25_like_ssid(ssid: &[u8]) -> bool {
    if ssid.is_empty() || ssid.len() > 2 || !ssid.iter().all(u8::is_ascii_digit) {
        return false;
    }

    let mut value = 0u8;
    for digit in ssid {
        value = value * 10 + (digit - b'0');
    }

    value <= 15
}
