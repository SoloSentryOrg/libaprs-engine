#![forbid(unsafe_code)]

//! UDP datagram APRS transport helpers.

use std::io;
use std::net::UdpSocket;

/// Receives up to `max_datagrams` UDP datagrams as owned APRS packet bytes.
pub fn recv_packet_datagrams(
    socket: &UdpSocket,
    max_datagrams: usize,
    max_datagram_len: usize,
) -> io::Result<Vec<Vec<u8>>> {
    let mut packets = Vec::new();
    for _ in 0..max_datagrams {
        let mut buffer = vec![0; max_datagram_len.saturating_add(1)];
        let len = socket.recv(&mut buffer)?;
        if len > max_datagram_len {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "UDP datagram exceeds configured byte limit",
            ));
        }
        buffer.truncate(len);
        packets.push(buffer);
    }
    Ok(packets)
}
