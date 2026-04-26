#![forbid(unsafe_code)]

//! MQTT payload helpers for APRS packet bytes.

/// Copies one MQTT publish payload into owned APRS packet bytes.
#[must_use]
pub fn packet_from_publish_payload(payload: &[u8]) -> Vec<u8> {
    payload.to_vec()
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
