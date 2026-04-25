#![forbid(unsafe_code)]

//! File-oriented APRS transport helpers.
//!
//! This crate keeps file I/O outside the core parser crate. It reads packet
//! files as bytes, splits them with `LineTransport`, and returns owned packet
//! byte vectors for callers that want a simple adapter.

use std::fs;
use std::io;
use std::path::Path;

use libaprs_engine::LineTransport;

/// Reads newline-separated packet bytes from a file path.
pub fn read_packet_lines_from_path(path: impl AsRef<Path>) -> io::Result<Vec<Vec<u8>>> {
    let input = fs::read(path)?;
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
