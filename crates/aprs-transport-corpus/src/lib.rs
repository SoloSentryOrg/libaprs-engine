#![forbid(unsafe_code)]

//! Corpus replay helpers for APRS packet fixtures.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use libaprs_engine::LineTransport;

/// Reads all regular files in a directory in stable path order.
pub fn read_corpus_packet_lines(dir: impl AsRef<Path>) -> io::Result<Vec<Vec<u8>>> {
    let mut files = fs::read_dir(dir)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<io::Result<Vec<PathBuf>>>()?;
    files.sort();

    let mut packets = Vec::new();
    for path in files {
        if path.is_file() {
            let input = fs::read(path)?;
            packets.extend(
                LineTransport::new(&input)
                    .packets()
                    .into_iter()
                    .map(<[u8]>::to_vec),
            );
        }
    }
    Ok(packets)
}
