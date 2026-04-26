use aprs_transport_kiss::{decode_frames, encode_data_frame, KissFrame};

#[test]
fn kiss_round_trip_preserves_escaped_packet_bytes() {
    let packet = b"N0CALL>APRS:>\xc0\xdb";

    let encoded = encode_data_frame(2, packet).expect("valid frame");
    let frames = decode_frames(&encoded).expect("valid KISS frame");

    assert_eq!(
        frames,
        vec![KissFrame {
            port: 2,
            command: 0,
            payload: packet.to_vec(),
        }]
    );
}

#[test]
fn kiss_rejects_unclosed_or_bad_escape_frames() {
    assert!(decode_frames(&[0xc0, 0x00, b'a']).is_err());
    assert!(decode_frames(&[0xc0, 0x00, 0xdb, 0x01, 0xc0]).is_err());
}
