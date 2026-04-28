use aprs_transport_http::{read_packet_lines_from_body, read_packet_lines_from_body_with_limit};

#[test]
fn http_body_reader_splits_lines_and_preserves_bytes() {
    let packets = read_packet_lines_from_body(b"N0CALL>APRS:>one\nN1CALL>APRS:>\xff\n");

    assert_eq!(
        packets,
        vec![b"N0CALL>APRS:>one".to_vec(), b"N1CALL>APRS:>\xff".to_vec()]
    );
}

#[test]
fn http_body_reader_rejects_input_over_configured_limit() {
    let error = read_packet_lines_from_body_with_limit(b"N0CALL>APRS:>oversized\n", 4)
        .expect_err("oversized input must fail");

    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    assert_eq!(error.to_string(), "transport.oversized_input");
}

#[test]
fn http_body_reader_rejects_packet_line_over_protocol_limit() {
    let mut input = b"N0CALL>APRS:>".to_vec();
    input.resize(
        libaprs_engine::MAX_PACKET_LEN + b"N0CALL>APRS:>".len() + 1,
        b'A',
    );
    input.push(b'\n');

    let error = read_packet_lines_from_body_with_limit(&input, 4096)
        .expect_err("oversized packet line must fail");

    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    assert_eq!(error.to_string(), "transport.oversized_input");
}
