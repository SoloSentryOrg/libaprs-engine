#![forbid(unsafe_code)]

//! HTTP body APRS ingestion helpers.

use libaprs_engine::LineTransport;

/// Splits an HTTP request/response body into APRS packet byte lines.
#[must_use]
pub fn read_packet_lines_from_body(body: &[u8]) -> Vec<Vec<u8>> {
    LineTransport::new(body)
        .packets()
        .into_iter()
        .map(<[u8]>::to_vec)
        .collect()
}
