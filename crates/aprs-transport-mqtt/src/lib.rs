#![forbid(unsafe_code)]

//! MQTT payload helpers for APRS packet bytes.

use std::io;

use libaprs_engine::{oversized_input_error, MAX_PACKET_LEN};

/// Copies one MQTT publish payload into owned APRS packet bytes.
#[must_use]
pub fn packet_from_publish_payload(payload: &[u8]) -> Vec<u8> {
    payload.to_vec()
}

/// Copies one MQTT publish payload while enforcing the APRS packet limit.
pub fn try_packet_from_publish_payload(payload: &[u8]) -> io::Result<Vec<u8>> {
    packet_from_publish_payload_with_limit(payload, MAX_PACKET_LEN)
}

/// Copies one MQTT publish payload while enforcing an explicit byte limit.
pub fn packet_from_publish_payload_with_limit(
    payload: &[u8],
    max_packet_len: usize,
) -> io::Result<Vec<u8>> {
    if payload.len() > max_packet_len {
        return Err(oversized_input_error());
    }
    Ok(payload.to_vec())
}

/// Matches an MQTT topic filter supporting exact segments and `+` wildcards.
#[must_use]
pub fn topic_matches(filter: &str, topic: &str) -> bool {
    let mut filter_segments = filter.split('/');
    let mut topic_segments = topic.split('/');

    loop {
        match (filter_segments.next(), topic_segments.next()) {
            (None, None) => return true,
            (Some("#"), _) => return filter_segments.next().is_none(),
            (Some("+"), Some(_)) => {}
            (Some(expected), Some(actual)) if expected == actual => {}
            _ => return false,
        }
    }
}
