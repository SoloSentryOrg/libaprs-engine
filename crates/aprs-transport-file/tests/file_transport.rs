use aprs_transport_file::read_packet_lines;

#[test]
fn file_transport_preserves_non_utf8_packet_bytes() {
    let packets = read_packet_lines(b"N0CALL>APRS:>\xff\r\nN0CALL>APRS:>two\n\n");

    assert_eq!(
        packets,
        vec![b"N0CALL>APRS:>\xff".to_vec(), b"N0CALL>APRS:>two".to_vec()]
    );
}
