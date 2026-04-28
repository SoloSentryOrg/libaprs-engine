#![forbid(unsafe_code)]

//! KISS frame helpers for APRS packet bytes.

use libaprs_engine::MAX_PACKET_LEN;

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
    /// The decoded payload exceeds the configured frame limit.
    OversizedFrame,
}

impl KissError {
    /// Stable machine-readable error code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::UnclosedFrame => "kiss_unclosed_frame",
            Self::InvalidEscape => "kiss_invalid_escape",
            Self::InvalidPort => "kiss_invalid_port",
            Self::OversizedFrame => "kiss_oversized_frame",
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
    decode_frames_with_limit(input, MAX_PACKET_LEN)
}

/// Decodes all complete KISS frames while enforcing a decoded payload limit.
pub fn decode_frames_with_limit(
    input: &[u8],
    max_payload_len: usize,
) -> Result<Vec<KissFrame>, KissError> {
    let mut frames = Vec::new();
    let mut current: Option<Vec<u8>> = None;
    let max_encoded_frame_len = max_payload_len.saturating_mul(2).saturating_add(1);

    for &byte in input {
        if byte == FEND {
            if let Some(frame) = current.take() {
                if !frame.is_empty() {
                    frames.push(decode_one_frame(&frame, max_payload_len)?);
                }
            }
            current = Some(Vec::new());
        } else if let Some(frame) = current.as_mut() {
            if frame.len() >= max_encoded_frame_len {
                return Err(KissError::OversizedFrame);
            }
            frame.push(byte);
        }
    }

    if current.is_some_and(|frame| !frame.is_empty()) {
        return Err(KissError::UnclosedFrame);
    }

    Ok(frames)
}

fn decode_one_frame(frame: &[u8], max_payload_len: usize) -> Result<KissFrame, KissError> {
    let command = frame[0] & 0x0f;
    let port = frame[0] >> 4;
    let mut payload = Vec::with_capacity(frame.len().saturating_sub(1));
    let mut index = 1;

    while index < frame.len() {
        match frame[index] {
            FESC => {
                let escaped = *frame.get(index + 1).ok_or(KissError::InvalidEscape)?;
                match escaped {
                    TFEND => push_payload_byte(&mut payload, FEND, max_payload_len)?,
                    TFESC => push_payload_byte(&mut payload, FESC, max_payload_len)?,
                    _ => return Err(KissError::InvalidEscape),
                }
                index += 2;
            }
            byte => {
                push_payload_byte(&mut payload, byte, max_payload_len)?;
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

fn push_payload_byte(
    payload: &mut Vec<u8>,
    byte: u8,
    max_payload_len: usize,
) -> Result<(), KissError> {
    if payload.len() >= max_payload_len {
        return Err(KissError::OversizedFrame);
    }
    payload.push(byte);
    Ok(())
}
