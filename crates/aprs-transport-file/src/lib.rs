#![forbid(unsafe_code)]

//! File-oriented APRS transport helpers.
//!
//! This crate keeps file I/O outside the core parser crate. It reads packet
//! files as bytes, splits them with `LineTransport`, and returns owned packet
//! byte vectors for callers that want a simple adapter.

use std::fs::{self, File};
use std::io;
use std::path::Path;

use libaprs_engine::{
    oversized_input_error, read_all_with_limit, LineTransport, DEFAULT_TRANSPORT_READ_LIMIT,
};

/// Reads newline-separated packet bytes from a file path.
pub fn read_packet_lines_from_path(path: impl AsRef<Path>) -> io::Result<Vec<Vec<u8>>> {
    read_packet_lines_from_path_with_limit(path, DEFAULT_TRANSPORT_READ_LIMIT)
}

/// Reads newline-separated packet bytes from a file path with an explicit byte limit.
pub fn read_packet_lines_from_path_with_limit(
    path: impl AsRef<Path>,
    max_bytes: usize,
) -> io::Result<Vec<Vec<u8>>> {
    let path = path.as_ref();
    if fs::metadata(path)?.len() > max_bytes as u64 {
        return Err(oversized_input_error());
    }
    let input = read_all_with_limit(File::open(path)?, max_bytes)?;
    Ok(read_packet_lines(&input))
}

/// Splits newline-separated packet bytes into owned packet lines.
#[must_use]
pub fn read_packet_lines(input: &[u8]) -> Vec<Vec<u8>> {
    LineTransport::new(input)
        .packets()
        .into_iter()
        .map(<[u8]>::to_vec)
        .collect()
}
