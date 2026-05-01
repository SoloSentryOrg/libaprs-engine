use aprs_transport_aprs_is::{read_packet_lines as read_aprs_is_packet_lines, AprsIsLogin};
use aprs_transport_ax25::decode_ax25_ui_frame;
use aprs_transport_channel::drain_packet_channel;
use aprs_transport_file::read_packet_lines;
use aprs_transport_http::read_packet_lines_from_body;
use aprs_transport_kiss::{decode_frames, encode_data_frame};
use aprs_transport_mqtt::{packet_from_publish_payload, topic_matches};
use aprs_transport_serial::read_packet_lines as read_serial_packet_lines;
use aprs_transport_tcp::read_packet_lines_from_reader;
use aprs_transport_udp::recv_packet_datagrams;
use libaprs_engine::{parse_packet, AprsData};
use std::net::UdpSocket;
use std::sync::mpsc;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let packet = parse_packet(b"N0CALL>APRS:>hello").map_err(|error| error.code())?;
    assert!(matches!(packet.aprs_data(), AprsData::Status { .. }));
    assert_eq!(packet.to_diagnostic().semantic, "status");

    let file_packets = read_packet_lines(b"N0CALL>APRS:>file\n");
    assert_eq!(file_packets.len(), 1);

    let tcp_packets = read_packet_lines_from_reader(std::io::Cursor::new(b"N0CALL>APRS:>tcp\n"))?;
    assert_eq!(tcp_packets.len(), 1);

    let aprs_is_login = AprsIsLogin {
        callsign: "N0CALL",
        passcode: -1,
        software: "libaprs-engine-downstream-smoke 2.0.0",
        filter: Some("r/49/-72/50"),
    };
    assert!(aprs_is_login.line()?.ends_with("\r\n"));

    let aprs_is_packets = read_aprs_is_packet_lines(b"# banner\nN0CALL>APRS:>aprs-is\n");
    assert_eq!(aprs_is_packets.len(), 1);

    let kiss = encode_data_frame(0, b"N0CALL>APRS:>kiss").map_err(|error| error.code())?;
    assert_eq!(decode_frames(&kiss).map_err(|error| error.code())?.len(), 1);

    assert_eq!(
        read_serial_packet_lines(b"N0CALL>APRS:>serial\n"),
        vec![b"N0CALL>APRS:>serial".to_vec()]
    );
    assert_eq!(
        read_packet_lines_from_body(b"N0CALL>APRS:>http\n"),
        vec![b"N0CALL>APRS:>http".to_vec()]
    );
    assert_eq!(
        packet_from_publish_payload(b"N0CALL>APRS:>mqtt"),
        b"N0CALL>APRS:>mqtt".to_vec()
    );
    assert!(topic_matches("aprs/+/packet", "aprs/N0CALL/packet"));

    let mut ax25 = Vec::new();
    ax25.extend_from_slice(&encode_ax25_addr("APRS", true));
    ax25.extend_from_slice(&encode_ax25_addr("N0CALL", true));
    ax25.push(0x03);
    ax25.push(0xf0);
    ax25.extend_from_slice(b">ax25");
    assert_eq!(
        decode_ax25_ui_frame(&ax25)
            .map_err(|error| error.code())?
            .information,
        b">ax25"
    );

    let (sender, receiver) = mpsc::channel();
    sender.send(b"N0CALL>APRS:>channel".to_vec())?;
    assert_eq!(drain_packet_channel(&receiver, 1).len(), 1);

    let udp_receiver = UdpSocket::bind("127.0.0.1:0")?;
    udp_receiver.set_read_timeout(Some(Duration::from_secs(1)))?;
    let udp_sender = UdpSocket::bind("127.0.0.1:0")?;
    udp_sender.send_to(b"N0CALL>APRS:>udp", udp_receiver.local_addr()?)?;
    assert_eq!(recv_packet_datagrams(&udp_receiver, 1, 128)?.len(), 1);

    Ok(())
}

fn encode_ax25_addr(callsign: &str, last: bool) -> [u8; 7] {
    let mut out = [b' ' << 1; 7];
    for (index, byte) in callsign.bytes().take(6).enumerate() {
        out[index] = byte << 1;
    }
    out[6] = 0x60 | u8::from(last);
    out
}
