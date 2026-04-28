#![forbid(unsafe_code)]

//! APRS-IS transport helpers.
//!
//! This crate keeps APRS-IS connection framing outside the core parser crate.
//! It provides login-line construction and reader-backed packet splitting while
//! preserving packet bytes.

use std::io::{self, Read};

use libaprs_engine::{read_all_with_limit, LineTransport, MAX_PACKET_LEN};

/// Default maximum APRS-IS reader batch size.
pub const DEFAULT_MAX_APRS_IS_READ_BYTES: usize = 1024 * 1024;

/// APRS-IS login settings.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AprsIsLogin<'a> {
    /// Login callsign.
    pub callsign: &'a str,
    /// APRS-IS passcode, or `-1` for receive-only/no-auth use.
    pub passcode: i32,
    /// Client software/version identifier.
    pub software: &'a str,
    /// Optional APRS-IS filter expression.
    pub filter: Option<&'a str>,
}

impl AprsIsLogin<'_> {
    /// Builds the APRS-IS login line terminated with CRLF.
    ///
    /// Values containing CR or LF are rejected to prevent line injection into
    /// the APRS-IS control stream.
    pub fn line(&self) -> Result<String, AprsIsLoginError> {
        validate_login_field("callsign", self.callsign)?;
        validate_login_field("software", self.software)?;
        if let Some(filter) = self.filter {
            validate_login_field("filter", filter)?;
        }

        let mut line = format!(
            "user {} pass {} vers {}",
            self.callsign, self.passcode, self.software
        );
        if let Some(filter) = self.filter {
            line.push_str(" filter ");
            line.push_str(filter);
        }
        line.push_str("\r\n");
        Ok(line)
    }
}

/// APRS-IS login line validation error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AprsIsLoginError {
    /// A login field contains CR or LF and would inject another line.
    LineInjection { field: &'static str },
}

impl AprsIsLoginError {
    /// Stable machine-readable error code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::LineInjection { .. } => "aprs_is_login_line_injection",
        }
    }
}

impl std::fmt::Display for AprsIsLoginError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LineInjection { field } => {
                write!(formatter, "APRS-IS login field contains CR or LF: {field}")
            }
        }
    }
}

impl std::error::Error for AprsIsLoginError {}

/// Reads newline-separated APRS-IS packet bytes from a generic reader.
pub fn read_packet_lines_from_reader(reader: impl Read) -> io::Result<Vec<Vec<u8>>> {
    read_packet_lines_from_reader_with_limit(reader, DEFAULT_MAX_APRS_IS_READ_BYTES)
}

/// Reads APRS-IS packet bytes from a reader with an explicit byte limit.
pub fn read_packet_lines_from_reader_with_limit(
    reader: impl Read,
    max_bytes: usize,
) -> io::Result<Vec<Vec<u8>>> {
    let input = read_all(reader, max_bytes)?;
    try_read_packet_lines(&input)
}

/// Splits newline-separated APRS-IS packet bytes into owned packet lines.
#[must_use]
pub fn read_packet_lines(input: &[u8]) -> Vec<Vec<u8>> {
    LineTransport::new(input)
        .packets()
        .into_iter()
        .filter(|line| !line.starts_with(b"#"))
        .map(<[u8]>::to_vec)
        .collect()
}

/// Splits APRS-IS packet bytes while enforcing the APRS packet limit.
pub fn try_read_packet_lines(input: &[u8]) -> io::Result<Vec<Vec<u8>>> {
    Ok(LineTransport::new(input)
        .packets_with_limit(MAX_PACKET_LEN)?
        .into_iter()
        .filter(|line| !line.starts_with(b"#"))
        .map(<[u8]>::to_vec)
        .collect())
}

fn read_all(reader: impl Read, max_bytes: usize) -> io::Result<Vec<u8>> {
    read_all_with_limit(reader, max_bytes)
}

fn validate_login_field(field: &'static str, value: &str) -> Result<(), AprsIsLoginError> {
    if value.as_bytes().contains(&b'\r') || value.as_bytes().contains(&b'\n') {
        return Err(AprsIsLoginError::LineInjection { field });
    }
    Ok(())
}
