#![forbid(unsafe_code)]

//! TCP-oriented APRS transport helpers.
//!
//! This crate keeps network I/O outside the core parser crate. It reads bytes
//! from a caller-provided reader or TCP address, then splits packets with
//! `LineTransport` without converting packet bytes to UTF-8.

use std::io::{self, Read};
use std::net::{TcpStream, ToSocketAddrs};

use libaprs_engine::LineTransport;

/// Reads newline-separated packet bytes from a generic reader.
pub fn read_packet_lines_from_reader(reader: impl Read) -> io::Result<Vec<Vec<u8>>> {
    let input = read_all(reader)?;
    Ok(read_packet_lines(&input))
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

fn read_all(mut reader: impl Read) -> io::Result<Vec<u8>> {
    let mut input = Vec::new();
    reader.read_to_end(&mut input)?;
    Ok(input)
}
