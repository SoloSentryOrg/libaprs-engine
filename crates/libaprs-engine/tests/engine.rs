use libaprs_engine::{Engine, EngineResult, LineTransport, Policy, PolicyRejection};

#[test]
fn line_transport_splits_lf_and_crlf_packets() {
    let transport = LineTransport::new(b"N0CALL>APRS:>one\r\nN0CALL>APRS:>two\n\n");

    assert_eq!(
        transport.packets(),
        vec![
            b"N0CALL>APRS:>one".as_slice(),
            b"N0CALL>APRS:>two".as_slice()
        ]
    );
}

#[test]
fn engine_counts_accepted_rejected_and_malformed_packets() {
    let mut engine = Engine::default();

    assert!(matches!(
        engine.process(b"N0CALL>APRS:>ok"),
        EngineResult::Accepted { .. }
    ));
    assert!(matches!(
        engine.process(b"N0CALL>APRS:~unsupported"),
        EngineResult::Rejected {
            reason: PolicyRejection::UnsupportedSemantics,
            ..
        }
    ));
    assert!(matches!(
        engine.process(b"bad packet"),
        EngineResult::ParseError(_)
    ));

    let counters = engine.counters();
    assert_eq!(counters.accepted, 1);
    assert_eq!(counters.rejected, 1);
    assert_eq!(counters.malformed, 1);
}

#[test]
fn permissive_policy_accepts_unsupported_semantics() {
    let mut engine = Engine::new(Policy::permissive());

    assert!(matches!(
        engine.process(b"N0CALL>APRS:~unsupported"),
        EngineResult::Accepted { .. }
    ));
}

#[test]
fn policy_rejections_have_stable_codes() {
    assert_eq!(PolicyRejection::PathTooLong.code(), "policy.path_too_long");
    assert_eq!(
        PolicyRejection::MalformedSemantics.code(),
        "policy.malformed_semantics"
    );
    assert_eq!(
        PolicyRejection::UnsupportedSemantics.code(),
        "policy.unsupported_semantics"
    );
}

#[test]
fn json_diagnostic_escapes_raw_bytes_and_identifies_semantics() {
    let packet =
        libaprs_engine::parse_packet(b"N0CALL>APRS:>hello \"json\"").expect("packet should parse");
    let json = packet.to_json();

    assert!(json.contains("\"source\":\"N0CALL\""));
    assert!(json.contains("\"semantic\":\"status\""));
    assert!(json.contains("\\\"json\\\""));
}
