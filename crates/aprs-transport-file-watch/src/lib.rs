#![forbid(unsafe_code)]

//! Append-only file watch helpers for APRS packet logs.

use std::fs::File;
use std::io::{self, Seek, SeekFrom};
use std::path::Path;

use libaprs_engine::{read_all_with_limit, LineTransport, DEFAULT_TRANSPORT_READ_LIMIT};

/// Result of reading appended bytes from a packet file.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppendedPackets {
    /// Packet lines found in the appended byte range.
    pub packets: Vec<Vec<u8>>,
    /// Offset to use on the next read.
    pub next_offset: u64,
}

/// Reads bytes appended after `offset` and splits them into packet lines.
pub fn read_appended_packet_lines(
    path: impl AsRef<Path>,
    offset: u64,
) -> io::Result<AppendedPackets> {
    read_appended_packet_lines_with_limit(path, offset, DEFAULT_TRANSPORT_READ_LIMIT)
}

/// Reads appended bytes after `offset` with an explicit byte limit.
pub fn read_appended_packet_lines_with_limit(
    path: impl AsRef<Path>,
    offset: u64,
    max_bytes: usize,
) -> io::Result<AppendedPackets> {
    let mut file = File::open(path)?;
    file.seek(SeekFrom::Start(offset))?;
    let input = read_all_with_limit(file, max_bytes)?;
    let next_offset = offset.saturating_add(input.len() as u64);
    let packets = LineTransport::new(&input)
        .packets()
        .into_iter()
        .map(<[u8]>::to_vec)
        .collect();

    Ok(AppendedPackets {
        packets,
        next_offset,
    })
}
