use std::io::Cursor;

use aprs_transport_serial::read_packet_lines_from_reader_with_limit;

#[test]
fn serial_reader_preserves_non_utf8_packet_bytes() {
    let packets =
        read_packet_lines_from_reader_with_limit(Cursor::new(b"N0CALL>APRS:>\xff\n"), 1024)
            .expect("reader should parse");

    assert_eq!(packets, vec![b"N0CALL>APRS:>\xff".to_vec()]);
}

#[test]
fn serial_reader_rejects_oversized_batches() {
    let error = read_packet_lines_from_reader_with_limit(Cursor::new(b"abcdef"), 4)
        .expect_err("oversized input must fail");

    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
}
