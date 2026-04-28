#![forbid(unsafe_code)]

//! TCP-oriented APRS transport helpers.
//!
//! This crate keeps network I/O outside the core parser crate. It reads bytes
//! from a caller-provided reader or TCP address, then splits packets with
//! `LineTransport` without converting packet bytes to UTF-8.

use std::io::{self, Read};
use std::net::{TcpStream, ToSocketAddrs};

use libaprs_engine::{
    read_all_with_limit, LineTransport, DEFAULT_TRANSPORT_READ_LIMIT, MAX_PACKET_LEN,
};

/// Reads newline-separated packet bytes from a generic reader.
pub fn read_packet_lines_from_reader(reader: impl Read) -> io::Result<Vec<Vec<u8>>> {
    read_packet_lines_from_reader_with_limit(reader, DEFAULT_TRANSPORT_READ_LIMIT)
}

/// Reads newline-separated packet bytes from a generic reader with an explicit byte limit.
pub fn read_packet_lines_from_reader_with_limit(
    reader: impl Read,
    max_bytes: usize,
) -> io::Result<Vec<Vec<u8>>> {
    let input = read_all_with_limit(reader, max_bytes)?;
    try_read_packet_lines(&input)
}

/// Connects to a TCP address and reads newline-separated packet bytes.
pub fn read_packet_lines_from_tcp_addr(addr: impl ToSocketAddrs) -> io::Result<Vec<Vec<u8>>> {
    read_packet_lines_from_reader(TcpStream::connect(addr)?)
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

/// Splits newline-separated packet bytes while enforcing the APRS packet limit.
pub fn try_read_packet_lines(input: &[u8]) -> io::Result<Vec<Vec<u8>>> {
    Ok(LineTransport::new(input)
        .packets_with_limit(MAX_PACKET_LEN)?
        .into_iter()
        .map(<[u8]>::to_vec)
        .collect())
}
