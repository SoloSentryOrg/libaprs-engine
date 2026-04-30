#![forbid(unsafe_code)]

//! TCP-oriented APRS transport helpers.
//!
//! This crate keeps network I/O outside the core parser crate. It reads bytes
//! from a caller-provided reader or TCP address, then splits packets with
//! `LineTransport` without converting packet bytes to UTF-8.

use std::io::{self, Read};
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

use libaprs_engine::{
    read_all_with_limit, LineTransport, DEFAULT_TRANSPORT_READ_LIMIT, MAX_PACKET_LEN,
};

/// TCP read options owned by the caller.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TcpReadOptions {
    /// Optional timeout for establishing the TCP connection.
    pub connect_timeout: Option<Duration>,
    /// Optional timeout applied to reads after connecting.
    pub read_timeout: Option<Duration>,
    /// Maximum accepted byte batch read from the TCP stream.
    pub max_bytes: usize,
}

impl TcpReadOptions {
    /// Returns options with an explicit connection timeout.
    #[must_use]
    pub const fn with_connect_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.connect_timeout = timeout;
        self
    }

    /// Returns options with an explicit read timeout.
    #[must_use]
    pub const fn with_read_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.read_timeout = timeout;
        self
    }

    /// Returns options with an explicit maximum read size.
    #[must_use]
    pub const fn with_max_bytes(mut self, max_bytes: usize) -> Self {
        self.max_bytes = max_bytes;
        self
    }
}

impl Default for TcpReadOptions {
    fn default() -> Self {
        Self {
            connect_timeout: None,
            read_timeout: None,
            max_bytes: DEFAULT_TRANSPORT_READ_LIMIT,
        }
    }
}

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
    read_packet_lines_from_tcp_addr_with_options(addr, TcpReadOptions::default())
}

/// Connects to a TCP address with caller-owned timeout and read-limit options.
pub fn read_packet_lines_from_tcp_addr_with_options(
    addr: impl ToSocketAddrs,
    options: TcpReadOptions,
) -> io::Result<Vec<Vec<u8>>> {
    let stream = connect_with_options(addr, options)?;
    read_packet_lines_from_reader_with_limit(stream, options.max_bytes)
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

fn connect_with_options(
    addr: impl ToSocketAddrs,
    options: TcpReadOptions,
) -> io::Result<TcpStream> {
    let mut last_error = None;
    for socket_addr in addr.to_socket_addrs()? {
        let result = if let Some(timeout) = options.connect_timeout {
            TcpStream::connect_timeout(&socket_addr, timeout)
        } else {
            TcpStream::connect(socket_addr)
        };

        match result {
            Ok(stream) => {
                stream.set_read_timeout(options.read_timeout)?;
                return Ok(stream);
            }
            Err(error) => last_error = Some(error),
        }
    }

    Err(last_error.unwrap_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "TCP address resolved to no socket addresses",
        )
    }))
}
