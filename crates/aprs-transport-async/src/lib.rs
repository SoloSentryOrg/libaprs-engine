#![forbid(unsafe_code)]

//! Runtime-neutral async helpers for APRS packet bytes.

use libaprs_engine::LineTransport;

/// Splits packet bytes asynchronously without choosing a runtime dependency.
pub async fn split_packet_lines(input: &[u8]) -> Vec<Vec<u8>> {
    LineTransport::new(input)
        .packets()
        .into_iter()
        .map(<[u8]>::to_vec)
        .collect()
}
