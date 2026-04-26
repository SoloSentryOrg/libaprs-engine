#![forbid(unsafe_code)]

//! In-process channel helpers for APRS packet bytes.

use std::sync::mpsc::{Receiver, TryRecvError};

/// Drains up to `max_packets` owned packet byte vectors from a channel.
#[must_use]
pub fn drain_packet_channel(receiver: &Receiver<Vec<u8>>, max_packets: usize) -> Vec<Vec<u8>> {
    let mut packets = Vec::new();
    for _ in 0..max_packets {
        match receiver.try_recv() {
            Ok(packet) => packets.push(packet),
            Err(TryRecvError::Empty | TryRecvError::Disconnected) => break,
        }
    }
    packets
}
