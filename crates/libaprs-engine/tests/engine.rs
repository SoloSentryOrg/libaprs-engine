use std::io::Cursor;

use libaprs_engine::{
    read_all_with_limit, Engine, EngineEvent, EngineEventKind, EngineResult, LineTransport, Policy,
    PolicyRejection, TransportErrorCode, TransportFailureEvent, EVENT_RAW_BYTE_LIMIT,
};

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
fn transport_boundaries_reject_oversized_batches_before_parsing() {
    let error = read_all_with_limit(Cursor::new(b"abcdef"), 5)
        .expect_err("bounded read must reject oversized input");

    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    assert_eq!(error.to_string(), TransportErrorCode::OversizedInput.code());
}

#[test]
fn line_transport_rejects_oversized_packet_before_owned_copies() {
    let input = b"N0CALL>APRS:>short\nN0CALL>APRS:>this-packet-is-too-long\n";
    let error = LineTransport::new(input)
        .packets_with_limit(b"N0CALL>APRS:>short".len())
        .expect_err("per-packet limit must reject oversized line");

    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    assert_eq!(error.to_string(), TransportErrorCode::OversizedInput.code());
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
fn engine_process_event_emits_stable_observability_events() {
    let mut engine = Engine::default();

    let EngineEvent::Accepted(accepted) = engine.process_event(b"N0CALL>APRS:>ok") else {
        panic!("accepted packet should emit an accepted event");
    };
    assert_eq!(accepted.kind(), EngineEventKind::Accepted);
    assert_eq!(accepted.kind().code(), "accepted");
    assert_eq!(accepted.packet.raw().as_bytes(), b"N0CALL>APRS:>ok");
    assert_eq!(accepted.packet.summary().semantic, "status");

    let EngineEvent::Rejected(rejected) = engine.process_event(b"N0CALL>APRS:~unsupported") else {
        panic!("unsupported packet should emit a policy rejection event");
    };
    assert_eq!(rejected.kind(), EngineEventKind::PolicyRejected);
    assert_eq!(rejected.reason, PolicyRejection::UnsupportedSemantics);
    assert_eq!(rejected.diagnostic.code, "policy.unsupported_semantics");
    assert_eq!(
        rejected.packet.raw().as_bytes(),
        b"N0CALL>APRS:~unsupported"
    );

    let malformed_input = b"not a packet \xff";
    let EngineEvent::Malformed(malformed) = engine.process_event(malformed_input) else {
        panic!("malformed packet should emit a malformed event");
    };
    assert_eq!(malformed.kind(), EngineEventKind::Malformed);
    assert_eq!(malformed.diagnostic.code, "parse.missing_separator");
    assert_eq!(malformed.raw, malformed_input);
    assert!(!malformed.raw_truncated);

    assert_eq!(engine.counters().accepted, 1);
    assert_eq!(engine.counters().rejected, 1);
    assert_eq!(engine.counters().malformed, 1);
}

#[test]
fn malformed_event_raw_bytes_are_bounded_for_oversized_input() {
    let mut engine = Engine::default();
    let oversized = vec![b'X'; EVENT_RAW_BYTE_LIMIT + 64];

    let EngineEvent::Malformed(malformed) = engine.process_event(&oversized) else {
        panic!("oversized packet should emit a malformed event");
    };

    assert_eq!(malformed.diagnostic.code, "parse.oversized");
    assert_eq!(malformed.raw.len(), EVENT_RAW_BYTE_LIMIT);
    assert!(malformed.raw_truncated);
}

#[test]
fn transport_failure_event_exposes_stable_diagnostic_metadata() {
    let event = TransportFailureEvent::from_code(TransportErrorCode::OversizedInput);

    assert_eq!(event.kind(), EngineEventKind::TransportFailure);
    assert_eq!(event.kind().code(), "transport_failure");
    assert_eq!(event.code, TransportErrorCode::OversizedInput);
    assert_eq!(event.diagnostic.code, "transport.oversized_input");
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
fn default_policy_reports_but_does_not_reject_invalid_nmea_checksum() {
    let mut engine = Engine::default();

    let EngineResult::Accepted { packet } =
        engine.process(b"N0CALL>APRS:$GPGLL,4916.45,N,12311.12,W,225444,A,*00")
    else {
        panic!("invalid NMEA checksum should remain accepted unless policy enables rejection");
    };

    let checksum = packet
        .summary()
        .nmea_checksum
        .expect("checksum details should be reported");
    assert_eq!(checksum.expected, 0x00);
    assert_eq!(checksum.calculated, 0x1d);
    assert!(!checksum.valid);
}

#[test]
fn policy_can_reject_invalid_nmea_checksum() {
    let mut policy = Policy::strict();
    policy.reject_invalid_nmea_checksum = true;
    let mut engine = Engine::new(policy);

    assert!(matches!(
        engine.process(b"N0CALL>APRS:$GPGLL,4916.45,N,12311.12,W,225444,A,*00"),
        EngineResult::Rejected {
            reason: PolicyRejection::InvalidNmeaChecksum,
            ..
        }
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
    assert_eq!(
        PolicyRejection::InvalidNmeaChecksum.code(),
        "policy.nmea_checksum_mismatch"
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
