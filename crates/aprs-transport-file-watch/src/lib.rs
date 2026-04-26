#![forbid(unsafe_code)]

//! Append-only file watch helpers for APRS packet logs.

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;

use libaprs_engine::LineTransport;

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
    let mut file = File::open(path)?;
    file.seek(SeekFrom::Start(offset))?;
    let mut input = Vec::new();
    file.read_to_end(&mut input)?;
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
