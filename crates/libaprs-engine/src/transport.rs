use std::io::{self, Read};

use crate::{DiagnosticLayer, ErrorDiagnostic, MAX_PACKET_LEN};

/// Default maximum byte batch accepted by transport helper readers.
pub const DEFAULT_TRANSPORT_READ_LIMIT: usize = 1024 * 1024;

/// Stable transport diagnostic categories for logs and external systems.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransportErrorCode {
    /// Input exceeded the configured byte limit.
    OversizedInput,
    /// Input framing was invalid for the transport.
    InvalidFrame,
    /// Transport reached EOF before a complete packet/frame was available.
    UnexpectedEof,
    /// Transport operation timed out.
    Timeout,
    /// Underlying I/O failed.
    Io,
}

impl TransportErrorCode {
    /// Returns a stable machine-readable diagnostic code.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::OversizedInput => "transport.oversized_input",
            Self::InvalidFrame => "transport.invalid_frame",
            Self::UnexpectedEof => "transport.unexpected_eof",
            Self::Timeout => "transport.timeout",
            Self::Io => "transport.io",
        }
    }

    /// Returns structured transport error metadata for operator diagnostics.
    #[must_use]
    pub fn diagnostic(self) -> ErrorDiagnostic {
        match self {
            Self::OversizedInput => ErrorDiagnostic {
                layer: DiagnosticLayer::Transport,
                code: self.code(),
                name: "oversized_input",
                description: "transport input exceeded the configured byte limit",
                remediation: "keep bounded transport reads enabled and reject the oversized input",
            },
            Self::InvalidFrame => ErrorDiagnostic {
                layer: DiagnosticLayer::Transport,
                code: self.code(),
                name: "invalid_frame",
                description: "transport-specific framing was invalid before codec parsing",
                remediation: "drop the frame and inspect upstream framing or escaping",
            },
            Self::UnexpectedEof => ErrorDiagnostic {
                layer: DiagnosticLayer::Transport,
                code: self.code(),
                name: "unexpected_eof",
                description: "transport ended before a complete packet or frame was available",
                remediation: "discard the incomplete record and wait for a complete bounded frame",
            },
            Self::Timeout => ErrorDiagnostic {
                layer: DiagnosticLayer::Transport,
                code: self.code(),
                name: "timeout",
                description: "transport operation exceeded the caller-owned timeout",
                remediation: "apply reconnect or retry policy outside the core parser",
            },
            Self::Io => ErrorDiagnostic {
                layer: DiagnosticLayer::Transport,
                code: self.code(),
                name: "io",
                description: "underlying transport I/O failed",
                remediation:
                    "handle the I/O failure at the adapter boundary before parsing more bytes",
            },
        }
    }
}

/// Common packet-source contract for transport adapters.
pub trait PacketSource {
    /// Source-specific error type.
    type Error;

    /// Reads a bounded batch of packet byte vectors.
    fn recv_packets(&mut self) -> Result<Vec<Vec<u8>>, Self::Error>;
}

/// Common packet-sink contract for transport adapters.
pub trait PacketSink {
    /// Sink-specific error type.
    type Error;

    /// Sends one packet byte slice without mutating or normalizing it.
    fn send_packet(&mut self, packet: &[u8]) -> Result<(), Self::Error>;
}

/// Reads all available bytes from a reader while enforcing a hard limit.
pub fn read_all_with_limit(mut reader: impl Read, max_bytes: usize) -> io::Result<Vec<u8>> {
    let mut input = Vec::new();
    let mut limited = (&mut reader).take(max_bytes.saturating_add(1) as u64);
    limited.read_to_end(&mut input)?;
    if input.len() > max_bytes {
        return Err(oversized_input_error());
    }
    Ok(input)
}

/// Creates the standard oversized-input transport error.
#[must_use]
pub fn oversized_input_error() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        TransportErrorCode::OversizedInput.code(),
    )
}

/// Line-oriented packet source for file/stdin style transports.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LineTransport<'a> {
    input: &'a [u8],
}

impl<'a> LineTransport<'a> {
    /// Creates a transport over newline-separated packet bytes.
    #[must_use]
    pub fn new(input: &'a [u8]) -> Self {
        Self { input }
    }

    /// Iterates packet lines without trailing CR/LF bytes.
    #[must_use]
    pub fn packets(&self) -> Vec<&'a [u8]> {
        self.input
            .split(|byte| *byte == b'\n')
            .map(trim_trailing_carriage_return)
            .filter(|line| !line.is_empty())
            .collect()
    }

    /// Iterates packet lines while enforcing a per-packet byte limit.
    ///
    /// The limit is applied after removing one trailing carriage return from
    /// CRLF-framed lines and before allocating owned packet copies.
    pub fn packets_with_limit(&self, max_packet_len: usize) -> io::Result<Vec<&'a [u8]>> {
        let mut packets = Vec::new();
        for line in self
            .input
            .split(|byte| *byte == b'\n')
            .map(trim_trailing_carriage_return)
            .filter(|line| !line.is_empty())
        {
            if line.len() > max_packet_len {
                return Err(oversized_input_error());
            }
            packets.push(line);
        }
        Ok(packets)
    }
}

impl PacketSource for LineTransport<'_> {
    type Error = io::Error;

    fn recv_packets(&mut self) -> Result<Vec<Vec<u8>>, Self::Error> {
        Ok(self
            .packets_with_limit(MAX_PACKET_LEN)?
            .into_iter()
            .map(<[u8]>::to_vec)
            .collect())
    }
}

impl PacketSink for Vec<Vec<u8>> {
    type Error = io::Error;

    fn send_packet(&mut self, packet: &[u8]) -> Result<(), Self::Error> {
        self.push(packet.to_vec());
        Ok(())
    }
}

fn trim_trailing_carriage_return(line: &[u8]) -> &[u8] {
    line.strip_suffix(b"\r").unwrap_or(line)
}
