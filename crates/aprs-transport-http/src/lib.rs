#![forbid(unsafe_code)]

//! HTTP body APRS ingestion helpers.

use std::io;

use libaprs_engine::{
    oversized_input_error, LineTransport, DEFAULT_TRANSPORT_READ_LIMIT, MAX_PACKET_LEN,
};

/// Splits an HTTP request/response body into APRS packet byte lines.
#[must_use]
pub fn read_packet_lines_from_body(body: &[u8]) -> Vec<Vec<u8>> {
    LineTransport::new(body)
        .packets()
        .into_iter()
        .map(<[u8]>::to_vec)
        .collect()
}

/// Splits an HTTP body into packet lines with an explicit byte limit.
pub fn read_packet_lines_from_body_with_limit(
    body: &[u8],
    max_bytes: usize,
) -> io::Result<Vec<Vec<u8>>> {
    if body.len() > max_bytes {
        return Err(oversized_input_error());
    }
    read_packet_lines_from_body_with_packet_limit(body, MAX_PACKET_LEN)
}

/// Splits an HTTP body using the default transport byte limit.
pub fn read_bounded_packet_lines_from_body(body: &[u8]) -> io::Result<Vec<Vec<u8>>> {
    read_packet_lines_from_body_with_limit(body, DEFAULT_TRANSPORT_READ_LIMIT)
}

/// Splits an HTTP body while enforcing a per-packet byte limit.
pub fn read_packet_lines_from_body_with_packet_limit(
    body: &[u8],
    max_packet_len: usize,
) -> io::Result<Vec<Vec<u8>>> {
    Ok(LineTransport::new(body)
        .packets_with_limit(max_packet_len)?
        .into_iter()
        .map(<[u8]>::to_vec)
        .collect())
}
