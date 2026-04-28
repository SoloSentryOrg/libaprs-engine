use std::io::Cursor;

use aprs_transport_tcp::{read_packet_lines_from_reader, read_packet_lines_from_reader_with_limit};

#[test]
fn tcp_reader_transport_preserves_non_utf8_packet_bytes() {
    let input = Cursor::new(b"N0CALL>APRS:>\xff\r\nN0CALL>APRS:>two\n\n");

    let packets = read_packet_lines_from_reader(input).expect("reader should parse");

    assert_eq!(
        packets,
        vec![b"N0CALL>APRS:>\xff".to_vec(), b"N0CALL>APRS:>two".to_vec()]
    );
}

#[test]
fn tcp_reader_rejects_input_over_configured_limit() {
    let input = Cursor::new(b"N0CALL>APRS:>oversized\n");

    let error =
        read_packet_lines_from_reader_with_limit(input, 4).expect_err("oversized input must fail");

    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    assert_eq!(error.to_string(), "transport.oversized_input");
}

#[test]
fn tcp_reader_rejects_packet_line_over_protocol_limit() {
    let mut input = b"N0CALL>APRS:>".to_vec();
    input.resize(
        libaprs_engine::MAX_PACKET_LEN + b"N0CALL>APRS:>".len() + 1,
        b'A',
    );
    input.push(b'\n');

    let error = read_packet_lines_from_reader_with_limit(Cursor::new(input), 4096)
        .expect_err("oversized packet line must fail");

    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    assert_eq!(error.to_string(), "transport.oversized_input");
}
