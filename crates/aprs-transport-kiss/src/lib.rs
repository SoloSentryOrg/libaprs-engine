#![forbid(unsafe_code)]

//! KISS frame helpers for APRS packet bytes.

const FEND: u8 = 0xc0;
const FESC: u8 = 0xdb;
const TFEND: u8 = 0xdc;
const TFESC: u8 = 0xdd;

/// Decoded KISS frame.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KissFrame {
    /// KISS port number from the high nibble.
    pub port: u8,
    /// KISS command from the low nibble. APRS packet data uses command `0`.
    pub command: u8,
    /// Unescaped frame payload bytes.
    pub payload: Vec<u8>,
}

/// KISS framing error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KissError {
    /// A frame was opened but not closed.
    UnclosedFrame,
    /// A KISS escape sequence used an unsupported byte.
    InvalidEscape,
    /// The requested KISS port is outside the 4-bit field.
    InvalidPort,
}

impl KissError {
    /// Stable machine-readable error code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::UnclosedFrame => "kiss_unclosed_frame",
            Self::InvalidEscape => "kiss_invalid_escape",
            Self::InvalidPort => "kiss_invalid_port",
        }
    }
}

/// Encodes an APRS packet as a KISS data frame.
pub fn encode_data_frame(port: u8, payload: &[u8]) -> Result<Vec<u8>, KissError> {
    if port > 0x0f {
        return Err(KissError::InvalidPort);
    }

    let mut frame = Vec::with_capacity(payload.len().saturating_add(3));
    frame.push(FEND);
    frame.push(port << 4);
    for &byte in payload {
        match byte {
            FEND => frame.extend_from_slice(&[FESC, TFEND]),
            FESC => frame.extend_from_slice(&[FESC, TFESC]),
            _ => frame.push(byte),
        }
    }
    frame.push(FEND);
    Ok(frame)
}

/// Decodes all complete KISS frames from `input`.
pub fn decode_frames(input: &[u8]) -> Result<Vec<KissFrame>, KissError> {
    let mut frames = Vec::new();
    let mut current: Option<Vec<u8>> = None;

    for &byte in input {
        if byte == FEND {
            if let Some(frame) = current.take() {
                if !frame.is_empty() {
                    frames.push(decode_one_frame(&frame)?);
                }
            }
            current = Some(Vec::new());
        } else if let Some(frame) = current.as_mut() {
            frame.push(byte);
        }
    }

    if current.is_some_and(|frame| !frame.is_empty()) {
        return Err(KissError::UnclosedFrame);
    }

    Ok(frames)
}

fn decode_one_frame(frame: &[u8]) -> Result<KissFrame, KissError> {
    let command = frame[0] & 0x0f;
    let port = frame[0] >> 4;
    let mut payload = Vec::with_capacity(frame.len().saturating_sub(1));
    let mut index = 1;

    while index < frame.len() {
        match frame[index] {
            FESC => {
                let escaped = *frame.get(index + 1).ok_or(KissError::InvalidEscape)?;
                match escaped {
                    TFEND => payload.push(FEND),
                    TFESC => payload.push(FESC),
                    _ => return Err(KissError::InvalidEscape),
                }
                index += 2;
            }
            byte => {
                payload.push(byte);
                index += 1;
            }
        }
    }

    Ok(KissFrame {
        port,
        command,
        payload,
    })
}
