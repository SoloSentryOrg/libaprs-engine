use std::fs;

use aprs_transport_file::{
    read_packet_lines, read_packet_lines_from_path_with_limit, try_read_packet_lines,
};

#[test]
fn file_transport_preserves_non_utf8_packet_bytes() {
    let packets = read_packet_lines(b"N0CALL>APRS:>\xff\r\nN0CALL>APRS:>two\n\n");

    assert_eq!(
        packets,
        vec![b"N0CALL>APRS:>\xff".to_vec(), b"N0CALL>APRS:>two".to_vec()]
    );
}

#[test]
fn file_transport_rejects_file_over_configured_limit() {
    let path = std::env::temp_dir().join(format!("libaprs-file-{}", std::process::id()));
    fs::write(&path, b"N0CALL>APRS:>oversized\n").expect("write");

    let error =
        read_packet_lines_from_path_with_limit(&path, 4).expect_err("oversized input must fail");

    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    assert_eq!(error.to_string(), "transport.oversized_input");
    let _ = fs::remove_file(path);
}

#[test]
fn file_transport_can_reject_packet_line_over_protocol_limit() {
    let mut input = b"N0CALL>APRS:>".to_vec();
    input.resize(
        libaprs_engine::MAX_PACKET_LEN + b"N0CALL>APRS:>".len() + 1,
        b'A',
    );
    input.push(b'\n');

    let error = try_read_packet_lines(&input).expect_err("oversized packet line must fail");

    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    assert_eq!(error.to_string(), "transport.oversized_input");
}
