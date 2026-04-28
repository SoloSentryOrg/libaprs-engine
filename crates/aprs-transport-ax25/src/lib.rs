#![forbid(unsafe_code)]

//! AX.25 UI frame helpers for APRS packet bytes.

use libaprs_engine::MAX_PACKET_LEN;

/// Default maximum accepted AX.25 UI frame bytes.
pub const MAX_AX25_UI_FRAME_LEN: usize = MAX_PACKET_LEN + 72;

/// Decoded AX.25 UI frame fields relevant to APRS.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Ax25UiFrame {
    /// Source callsign bytes.
    pub source: Vec<u8>,
    /// Destination callsign bytes.
    pub destination: Vec<u8>,
    /// Information field bytes.
    pub information: Vec<u8>,
}

/// AX.25 frame decoding error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Ax25Error {
    /// Frame ended before the address/control/PID fields were complete.
    Truncated,
    /// Address bytes are not valid shifted AX.25 address bytes.
    InvalidAddress,
    /// Frame is not a UI/no-layer-3 APRS frame.
    NotAprsUi,
    /// Frame exceeds the configured byte limit.
    OversizedFrame,
}

impl Ax25Error {
    /// Stable machine-readable error code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Truncated => "ax25_truncated",
            Self::InvalidAddress => "ax25_invalid_address",
            Self::NotAprsUi => "ax25_not_aprs_ui",
            Self::OversizedFrame => "ax25_oversized_frame",
        }
    }
}

/// Decodes a complete AX.25 UI frame and extracts the APRS information field.
pub fn decode_ax25_ui_frame(frame: &[u8]) -> Result<Ax25UiFrame, Ax25Error> {
    decode_ax25_ui_frame_with_limit(frame, MAX_AX25_UI_FRAME_LEN)
}

/// Decodes an AX.25 UI frame while enforcing a maximum frame length.
pub fn decode_ax25_ui_frame_with_limit(
    frame: &[u8],
    max_frame_len: usize,
) -> Result<Ax25UiFrame, Ax25Error> {
    if frame.len() > max_frame_len {
        return Err(Ax25Error::OversizedFrame);
    }
    if frame.len() < 16 {
        return Err(Ax25Error::Truncated);
    }

    let destination = decode_address(&frame[0..7])?;
    let source = decode_address(&frame[7..14])?;
    let mut offset = 14;
    while offset >= 7 && frame[offset - 1] & 0x01 == 0 {
        if frame.len() < offset + 7 {
            return Err(Ax25Error::Truncated);
        }
        offset += 7;
    }
    if frame.len() < offset + 2 {
        return Err(Ax25Error::Truncated);
    }
    if frame[offset] != 0x03 || frame[offset + 1] != 0xf0 {
        return Err(Ax25Error::NotAprsUi);
    }

    Ok(Ax25UiFrame {
        source,
        destination,
        information: frame[offset + 2..].to_vec(),
    })
}

fn decode_address(bytes: &[u8]) -> Result<Vec<u8>, Ax25Error> {
    if bytes.len() != 7 {
        return Err(Ax25Error::Truncated);
    }
    let mut callsign = Vec::new();
    for &byte in &bytes[..6] {
        if byte & 0x01 != 0 {
            return Err(Ax25Error::InvalidAddress);
        }
        let decoded = byte >> 1;
        if decoded != b' ' {
            callsign.push(decoded);
        }
    }
    if callsign.is_empty() {
        return Err(Ax25Error::InvalidAddress);
    }
    Ok(callsign)
}
