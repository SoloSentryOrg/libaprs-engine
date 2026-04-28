use aprs_transport_ax25::{decode_ax25_ui_frame, decode_ax25_ui_frame_with_limit, Ax25Error};

#[test]
fn ax25_ui_frame_decoder_extracts_aprs_payload() {
    let mut frame = Vec::new();
    frame.extend_from_slice(&encode_addr("APRS", 0, false));
    frame.extend_from_slice(&encode_addr("N0CALL", 0, true));
    frame.push(0x03);
    frame.push(0xf0);
    frame.extend_from_slice(b">hello");

    let decoded = decode_ax25_ui_frame(&frame).expect("valid UI frame");

    assert_eq!(decoded.source, b"N0CALL".to_vec());
    assert_eq!(decoded.destination, b"APRS".to_vec());
    assert_eq!(decoded.information, b">hello".to_vec());
}

#[test]
fn ax25_decoder_rejects_frames_over_configured_limit() {
    let frame = vec![0; 32];

    let error = decode_ax25_ui_frame_with_limit(&frame, 16).expect_err("oversized frame fails");

    assert_eq!(error, Ax25Error::OversizedFrame);
    assert_eq!(error.code(), "ax25_oversized_frame");
}

fn encode_addr(callsign: &str, ssid: u8, last: bool) -> [u8; 7] {
    let mut out = [b' ' << 1; 7];
    for (index, byte) in callsign.bytes().take(6).enumerate() {
        out[index] = byte << 1;
    }
    out[6] = 0x60 | ((ssid & 0x0f) << 1) | u8::from(last);
    out
}
