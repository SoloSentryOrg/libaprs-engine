#![forbid(unsafe_code)]

//! Runtime-neutral async helpers for APRS packet bytes.

use std::io;

use libaprs_engine::{LineTransport, MAX_PACKET_LEN};

/// Splits packet bytes asynchronously without choosing a runtime dependency.
pub async fn split_packet_lines(input: &[u8]) -> Vec<Vec<u8>> {
    LineTransport::new(input)
        .packets()
        .into_iter()
        .map(<[u8]>::to_vec)
        .collect()
}

/// Splits packet bytes asynchronously while enforcing the APRS packet limit.
pub async fn try_split_packet_lines(input: &[u8]) -> io::Result<Vec<Vec<u8>>> {
    split_packet_lines_with_limit(input, MAX_PACKET_LEN).await
}

/// Splits packet bytes asynchronously with an explicit per-packet limit.
pub async fn split_packet_lines_with_limit(
    input: &[u8],
    max_packet_len: usize,
) -> io::Result<Vec<Vec<u8>>> {
    Ok(LineTransport::new(input)
        .packets_with_limit(max_packet_len)?
        .into_iter()
        .map(<[u8]>::to_vec)
        .collect())
}
