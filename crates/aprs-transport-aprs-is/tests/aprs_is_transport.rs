use std::io::Cursor;

use aprs_transport_aprs_is::{
    q_construct_from_tnc2, read_packet_lines_from_reader, read_packet_lines_from_reader_with_limit,
    validate_aprs_is_callsign, AprsIsFilter, AprsIsLogin, AprsIsProfileError, AprsIsQConstructKind,
};

#[test]
fn aprs_is_login_line_includes_filter_and_crlf() {
    let login = AprsIsLogin {
        callsign: "N0CALL",
        passcode: -1,
        software: "libaprs-engine 1.1.0",
        filter: Some("r/49/-72/50"),
    };

    assert_eq!(
        login.line().expect("valid login line"),
        "user N0CALL pass -1 vers libaprs-engine 1.1.0 filter r/49/-72/50\r\n"
    );
}

#[test]
fn aprs_is_login_line_rejects_line_injection() {
    let cases = [
        AprsIsLogin {
            callsign: "N0CALL\r\nbad",
            passcode: -1,
            software: "libaprs-engine 1.1.0",
            filter: None,
        },
        AprsIsLogin {
            callsign: "N0CALL",
            passcode: -1,
            software: "libaprs-engine\r\nbad",
            filter: None,
        },
        AprsIsLogin {
            callsign: "N0CALL",
            passcode: -1,
            software: "libaprs-engine 1.1.0",
            filter: Some("r/49/-72/50\r\nbad"),
        },
        AprsIsLogin {
            callsign: "N0CALL",
            passcode: -1,
            software: "libaprs-engine\t1.1.0",
            filter: None,
        },
    ];

    for login in cases {
        let error = login.line().expect_err("CRLF must fail closed");

        assert_eq!(error.code(), "aprs_is_login_line_injection");
    }
}

#[test]
fn aprs_is_profile_login_requires_uppercase_callsign_and_valid_filter() {
    let login = AprsIsLogin {
        callsign: "N0CALL-7",
        passcode: -1,
        software: "libaprs-engine 2.5.0",
        filter: Some("r/49/-72/50 t/poimq"),
    };

    assert_eq!(
        login.profile_line().expect("profile login should encode"),
        "user N0CALL-7 pass -1 vers libaprs-engine 2.5.0 filter r/49/-72/50 t/poimq\r\n"
    );

    let lowercase = AprsIsLogin {
        callsign: "n0call",
        passcode: -1,
        software: "libaprs-engine 2.5.0",
        filter: None,
    };

    assert_eq!(
        lowercase.profile_line().expect_err("lowercase callsign"),
        AprsIsProfileError::LowercaseCallsign
    );

    let control_byte = AprsIsLogin {
        callsign: "N0CALL-7",
        passcode: -1,
        software: "libaprs-engine\u{1b}",
        filter: None,
    };

    assert_eq!(
        control_byte
            .profile_line()
            .expect_err("control bytes fail closed"),
        AprsIsProfileError::LineInjection { field: "software" }
    );
}

#[test]
fn aprs_is_filter_validation_rejects_empty_tokens_and_line_injection() {
    assert_eq!(
        AprsIsFilter::new("r/49/-72/50 t/poimq")
            .expect("valid filter")
            .as_str(),
        "r/49/-72/50 t/poimq"
    );
    assert_eq!(validate_aprs_is_callsign("N0CALL-15"), Ok(()));
    assert_eq!(
        validate_aprs_is_callsign("N0CALL-16"),
        Err(AprsIsProfileError::InvalidCallsign)
    );
    assert_eq!(
        AprsIsFilter::new("r/49/-72/50  t/poimq").expect_err("empty filter token"),
        AprsIsProfileError::InvalidFilter
    );
    assert_eq!(
        AprsIsFilter::new("r/49/-72/50\r\nbad").expect_err("line injection"),
        AprsIsProfileError::LineInjection { field: "filter" }
    );
}

#[test]
fn aprs_is_q_construct_diagnostics_parse_raw_tnc2_paths() {
    let packet = b"N0CALL>APRS,TCPIP*,qAC,T2SERVER:>status";
    let construct = q_construct_from_tnc2(packet).expect("q construct should be found");

    assert_eq!(construct.component, b"qAC");
    assert_eq!(construct.next_component, Some(b"T2SERVER".as_slice()));
    assert_eq!(construct.kind, AprsIsQConstructKind::VerifiedLogin);
    assert_eq!(construct.kind.code(), "qac");

    let gated =
        q_construct_from_tnc2(b"N0CALL>APRS,WIDE1-1,qAR,IGATE:>rf").expect("qAR should be found");
    assert_eq!(gated.kind, AprsIsQConstructKind::VerifiedIgate);

    assert!(q_construct_from_tnc2(b"N0CALL>APRS,WIDE1-1:>rf").is_none());
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
