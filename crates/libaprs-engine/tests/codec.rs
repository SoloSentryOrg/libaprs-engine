use libaprs_engine::{parse_packet, ParseError, MAX_PACKET_LEN};

#[test]
fn valid_packet_preserves_exact_raw_bytes() {
    let input = b"N0CALL>APRS,TCPIP*:hello world";

    let parsed = parse_packet(input).expect("valid packet should parse");

    assert_eq!(parsed.raw().as_bytes(), input);
    assert_eq!(parsed.source(), b"N0CALL");
    assert_eq!(parsed.path(), b"APRS,TCPIP*");
    assert_eq!(parsed.payload(), b"hello world");
}

#[test]
fn empty_input_fails_closed() {
    let err = parse_packet(b"").expect_err("empty input must be rejected");

    assert_eq!(err, ParseError::Empty);
}

#[test]
fn packet_without_required_separator_fails_closed() {
    let err = parse_packet(b"N0CALL APRS hello").expect_err("missing separators must be rejected");

    assert_eq!(err, ParseError::MissingSeparator);
}

#[test]
fn invalid_utf8_payload_preserves_raw_bytes_and_does_not_panic() {
    let input = b"N0CALL>APRS:\xff\xfe\xfd";

    let parsed = parse_packet(input).expect("payload bytes are opaque");

    assert_eq!(parsed.raw().as_bytes(), input);
    assert_eq!(parsed.payload(), b"\xff\xfe\xfd");
}

#[test]
fn oversized_packet_is_rejected() {
    let input = vec![b'A'; MAX_PACKET_LEN + 1];

    let err = parse_packet(&input).expect_err("oversized packets must be rejected");

    assert_eq!(err, ParseError::Oversized);
}
