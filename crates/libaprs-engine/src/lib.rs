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
