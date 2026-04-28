use std::net::UdpSocket;
use std::time::Duration;

use aprs_transport_udp::recv_packet_datagrams;
use libaprs_engine::{parse_packet, AprsData};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let receiver = UdpSocket::bind("127.0.0.1:0")?;
    receiver.set_read_timeout(Some(Duration::from_secs(1)))?;

    let sender = UdpSocket::bind("127.0.0.1:0")?;
    sender.send_to(b"N0CALL>APRS:>udp", receiver.local_addr()?)?;

    for packet_bytes in recv_packet_datagrams(&receiver, 1, 128)? {
        let packet = parse_packet(&packet_bytes).map_err(|error| error.code())?;
        assert!(matches!(packet.aprs_data(), AprsData::Status { .. }));
    }

    Ok(())
}
