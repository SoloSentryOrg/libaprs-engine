use aprs_transport_kiss::{
    decode_frames, decode_frames_with_limit, encode_data_frame, KissError, KissFrame,
};

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

#[test]
fn kiss_rejects_payload_over_configured_frame_limit() {
    let encoded = encode_data_frame(0, b"N0CALL>APRS:>toolong").expect("frame");

    let error = decode_frames_with_limit(&encoded, 4).expect_err("oversized frame must fail");

    assert_eq!(error, KissError::OversizedFrame);
    assert_eq!(error.code(), "kiss_oversized_frame");
}

#[test]
fn kiss_rejects_encoded_frame_that_exceeds_configured_limit_before_close() {
    let error =
        decode_frames_with_limit(&[0xc0, 0x00, b'a', b'b', b'c'], 1).expect_err("oversized frame");

    assert_eq!(error, KissError::OversizedFrame);
}
