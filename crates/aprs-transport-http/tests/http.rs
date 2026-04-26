use aprs_transport_http::read_packet_lines_from_body;

#[test]
fn http_body_reader_splits_lines_and_preserves_bytes() {
    let packets = read_packet_lines_from_body(b"N0CALL>APRS:>one\nN1CALL>APRS:>\xff\n");

    assert_eq!(
        packets,
        vec![b"N0CALL>APRS:>one".to_vec(), b"N1CALL>APRS:>\xff".to_vec()]
    );
}
