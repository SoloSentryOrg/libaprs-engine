#![forbid(unsafe_code)]

//! Serial-reader APRS transport helpers.

use std::io::{self, Read};

use libaprs_engine::{read_all_with_limit, LineTransport, MAX_PACKET_LEN};

/// Default maximum serial reader batch size.
pub const DEFAULT_MAX_SERIAL_READ_BYTES: usize = 1024 * 1024;

/// Reads newline-separated packet bytes from a generic serial-like reader.
pub fn read_packet_lines_from_reader(reader: impl Read) -> io::Result<Vec<Vec<u8>>> {
    read_packet_lines_from_reader_with_limit(reader, DEFAULT_MAX_SERIAL_READ_BYTES)
}

/// Reads newline-separated packet bytes with an explicit byte limit.
pub fn read_packet_lines_from_reader_with_limit(
    reader: impl Read,
    max_bytes: usize,
) -> io::Result<Vec<Vec<u8>>> {
    let input = read_all(reader, max_bytes)?;
    try_read_packet_lines(&input)
}

/// Splits serial packet bytes into owned packet lines.
#[must_use]
pub fn read_packet_lines(input: &[u8]) -> Vec<Vec<u8>> {
    LineTransport::new(input)
        .packets()
        .into_iter()
        .map(<[u8]>::to_vec)
        .collect()
}

/// Splits serial packet bytes while enforcing the APRS packet limit.
pub fn try_read_packet_lines(input: &[u8]) -> io::Result<Vec<Vec<u8>>> {
    Ok(LineTransport::new(input)
        .packets_with_limit(MAX_PACKET_LEN)?
        .into_iter()
        .map(<[u8]>::to_vec)
        .collect())
}

fn read_all(reader: impl Read, max_bytes: usize) -> io::Result<Vec<u8>> {
    read_all_with_limit(reader, max_bytes)
}
