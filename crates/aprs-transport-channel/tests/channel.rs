use std::sync::mpsc;

use aprs_transport_channel::drain_packet_channel;

#[test]
fn channel_transport_drains_owned_packet_bytes() {
    let (sender, receiver) = mpsc::channel();
    sender.send(b"N0CALL>APRS:>\xff".to_vec()).expect("send");
    drop(sender);

    let packets = drain_packet_channel(&receiver, 8);

    assert_eq!(packets, vec![b"N0CALL>APRS:>\xff".to_vec()]);
}
