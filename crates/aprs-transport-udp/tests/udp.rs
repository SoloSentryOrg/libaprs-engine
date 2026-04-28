use std::net::UdpSocket;
use std::time::Duration;

use aprs_transport_udp::recv_packet_datagrams;

#[test]
fn udp_datagram_reader_preserves_packet_bytes() {
    let receiver = UdpSocket::bind("127.0.0.1:0").expect("receiver bind");
    receiver
        .set_read_timeout(Some(Duration::from_secs(1)))
        .expect("timeout");
    let sender = UdpSocket::bind("127.0.0.1:0").expect("sender bind");

    sender
        .send_to(b"N0CALL>APRS:>\xff", receiver.local_addr().expect("addr"))
        .expect("send");

    let packets = recv_packet_datagrams(&receiver, 1, 128).expect("recv");

    assert_eq!(packets, vec![b"N0CALL>APRS:>\xff".to_vec()]);
}

#[test]
fn udp_datagram_reader_rejects_oversized_datagrams() {
    let receiver = UdpSocket::bind("127.0.0.1:0").expect("receiver bind");
    receiver
        .set_read_timeout(Some(Duration::from_secs(1)))
        .expect("timeout");
    let sender = UdpSocket::bind("127.0.0.1:0").expect("sender bind");

    sender
        .send_to(
            b"N0CALL>APRS:>too-long",
            receiver.local_addr().expect("addr"),
        )
        .expect("send");

    let error =
        recv_packet_datagrams(&receiver, 1, 4).expect_err("oversized datagram must fail closed");

    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    assert_eq!(error.to_string(), "transport.oversized_input");
}
