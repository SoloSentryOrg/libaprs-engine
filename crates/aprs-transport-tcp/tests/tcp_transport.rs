use std::io::{self, Cursor};
use std::net::TcpListener;
use std::thread;
use std::time::Duration;

use aprs_transport_tcp::{
    read_packet_lines_from_reader, read_packet_lines_from_reader_with_limit,
    read_packet_lines_from_tcp_addr_with_options, TcpReadOptions, DEFAULT_TCP_CONNECT_TIMEOUT,
    DEFAULT_TCP_READ_TIMEOUT,
};

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

#[test]
fn tcp_addr_helper_applies_caller_owned_read_timeout() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback listener");
    let addr = listener.local_addr().expect("listener address");
    let server = thread::spawn(move || {
        let (_stream, _) = listener.accept().expect("accept one connection");
        thread::sleep(Duration::from_millis(250));
    });

    let error = read_packet_lines_from_tcp_addr_with_options(
        addr,
        TcpReadOptions::default()
            .with_connect_timeout(Some(Duration::from_secs(1)))
            .with_read_timeout(Some(Duration::from_millis(50))),
    )
    .expect_err("idle peer should hit caller-owned read timeout");

    assert!(matches!(
        error.kind(),
        io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
    ));
    server.join().expect("server thread");
}

#[test]
fn tcp_options_default_uses_finite_timeouts() {
    let options = TcpReadOptions::default();

    assert_eq!(options.connect_timeout, Some(DEFAULT_TCP_CONNECT_TIMEOUT));
    assert_eq!(options.read_timeout, Some(DEFAULT_TCP_READ_TIMEOUT));
}

#[test]
fn tcp_options_builder_api_remains_source_compatible() {
    let options = TcpReadOptions::default()
        .with_connect_timeout(Some(Duration::from_secs(2)))
        .with_read_timeout(Some(Duration::from_secs(3)))
        .with_max_bytes(128);

    assert_eq!(options.connect_timeout, Some(Duration::from_secs(2)));
    assert_eq!(options.read_timeout, Some(Duration::from_secs(3)));
    assert_eq!(options.max_bytes, 128);
}
