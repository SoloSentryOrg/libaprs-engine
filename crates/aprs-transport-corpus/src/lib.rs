#![forbid(unsafe_code)]

//! Corpus replay helpers for APRS packet fixtures.

use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};

use libaprs_engine::{
    oversized_input_error, read_all_with_limit, LineTransport, DEFAULT_TRANSPORT_READ_LIMIT,
    MAX_PACKET_LEN,
};

/// Reads all regular files in a directory in stable path order.
pub fn read_corpus_packet_lines(dir: impl AsRef<Path>) -> io::Result<Vec<Vec<u8>>> {
    read_corpus_packet_lines_with_limit(dir, DEFAULT_TRANSPORT_READ_LIMIT)
}

/// Reads all regular files in a directory with a per-file byte limit.
pub fn read_corpus_packet_lines_with_limit(
    dir: impl AsRef<Path>,
    max_file_bytes: usize,
) -> io::Result<Vec<Vec<u8>>> {
    let mut files = fs::read_dir(dir)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<io::Result<Vec<PathBuf>>>()?;
    files.sort();

    let mut packets = Vec::new();
    for path in files {
        if path.is_file() {
            if fs::metadata(&path)?.len() > max_file_bytes as u64 {
                return Err(oversized_input_error());
            }
            let input = read_all_with_limit(File::open(path)?, max_file_bytes)?;
            packets.extend(
                LineTransport::new(&input)
                    .packets_with_limit(MAX_PACKET_LEN)?
                    .into_iter()
                    .map(<[u8]>::to_vec),
            );
        }
    }
    Ok(packets)
}
