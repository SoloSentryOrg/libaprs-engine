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

#[test]
fn channel_transport_respects_batch_limit_for_backpressure() {
    let (sender, receiver) = mpsc::channel();
    sender.send(b"N0CALL>APRS:>one".to_vec()).expect("send");
    sender.send(b"N1CALL>APRS:>two".to_vec()).expect("send");
    sender.send(b"N2CALL>APRS:>three".to_vec()).expect("send");

    let first_batch = drain_packet_channel(&receiver, 2);
    let second_batch = drain_packet_channel(&receiver, 2);

    assert_eq!(
        first_batch,
        vec![b"N0CALL>APRS:>one".to_vec(), b"N1CALL>APRS:>two".to_vec()]
    );
    assert_eq!(second_batch, vec![b"N2CALL>APRS:>three".to_vec()]);
}
