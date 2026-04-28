use std::io::Cursor;

use aprs_transport_aprs_is::{
    read_packet_lines_from_reader, read_packet_lines_from_reader_with_limit, AprsIsLogin,
};

#[test]
fn aprs_is_login_line_includes_filter_and_crlf() {
    let login = AprsIsLogin {
        callsign: "N0CALL",
        passcode: -1,
        software: "libaprs-engine 0.6.0",
        filter: Some("r/49/-72/50"),
    };

    assert_eq!(
        login.line().expect("valid login line"),
        "user N0CALL pass -1 vers libaprs-engine 0.6.0 filter r/49/-72/50\r\n"
    );
}

#[test]
fn aprs_is_login_line_rejects_line_injection() {
    let login = AprsIsLogin {
        callsign: "N0CALL\r\nbad",
        passcode: -1,
        software: "libaprs-engine 0.6.0",
        filter: None,
    };

    let error = login.line().expect_err("CRLF must fail closed");

    assert_eq!(error.code(), "aprs_is_login_line_injection");
}

#[test]
fn aprs_is_reader_ignores_server_comments_and_preserves_bytes() {
    let input = Cursor::new(b"# server banner\r\nN0CALL>APRS:>\xff\n");

    let packets = read_packet_lines_from_reader(input).expect("reader should parse");

    assert_eq!(packets, vec![b"N0CALL>APRS:>\xff".to_vec()]);
}

#[test]
fn aprs_is_reader_rejects_inputs_over_configured_limit() {
    let input = Cursor::new(b"N0CALL>APRS:>oversized\n");

    let error =
        read_packet_lines_from_reader_with_limit(input, 4).expect_err("oversized input must fail");

    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
}

#[test]
fn aprs_is_reader_rejects_packet_line_over_protocol_limit() {
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
