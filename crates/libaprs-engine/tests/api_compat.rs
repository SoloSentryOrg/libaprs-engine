use libaprs_engine::{
    parse_packet, parse_packet_with_options, AprsData, Engine, EngineResult, LineTransport,
    ParseError, ParseOptions, Policy, PolicyRejection,
};

#[test]
fn stable_intent_parser_api_remains_usable() {
    let input = b"N0CALL>APRS:>hello";
    let parsed = parse_packet(input).expect("packet should parse");

    assert_eq!(parsed.raw().as_bytes(), input);
    assert_eq!(parsed.source(), b"N0CALL");
    assert_eq!(parsed.destination(), b"APRS");
    assert_eq!(parsed.payload(), b">hello");
    assert_eq!(parsed.information(), b"hello");
    assert_eq!(parsed.aprs_data().kind_name(), "status");
    assert!(parsed.to_json().contains("\"semantic\":\"status\""));
}

#[test]
fn stable_intent_options_and_errors_remain_usable() {
    let err = parse_packet_with_options(b"N0CALL>APRS:>hello", ParseOptions::new(4))
        .expect_err("tight size limit should reject packet");

    assert_eq!(err, ParseError::Oversized);
    assert_eq!(err.code(), "parse.oversized");
}

#[test]
fn stable_intent_engine_policy_and_transport_api_remain_usable() {
    let mut engine = Engine::new(Policy::strict());

    for packet in LineTransport::new(b"N0CALL>APRS:>ok\nN0CALL>APRS:~opaque\n").packets() {
        let _ = engine.process(packet);
    }

    let counters = engine.counters();
    assert_eq!(counters.accepted, 1);
    assert_eq!(counters.rejected, 1);
    assert_eq!(counters.malformed, 0);

    match engine.process(b"N0CALL>APRS:~opaque") {
        EngineResult::Rejected { reason, .. } => {
            assert_eq!(reason, PolicyRejection::UnsupportedSemantics);
            assert_eq!(reason.code(), "policy.unsupported_semantics");
        }
        other => panic!("expected rejection, got {other:?}"),
    }
}

#[test]
fn documented_semantic_helpers_remain_usable() {
    let telemetry = parse_packet(b"N0CALL>APRS:T#001,111,222,033,044,055,10101010")
        .expect("telemetry should parse");
    let AprsData::Telemetry(telemetry) = telemetry.aprs_data() else {
        panic!("expected telemetry");
    };

    assert_eq!(telemetry.sequence_number(), Some(1));
    assert_eq!(telemetry.analog_values(), Some([111, 222, 33, 44, 55]));
    assert_eq!(
        telemetry.digital_bits(),
        Some([true, false, true, false, true, false, true, false])
    );

    let nmea = parse_packet(b"N0CALL>APRS:$GPGLL,4916.45,N,12311.12,W,225444,A,*1D")
        .expect("NMEA should parse");
    let AprsData::Nmea(nmea) = nmea.aprs_data() else {
        panic!("expected NMEA");
    };
    assert!(nmea.checksum().expect("checksum").valid);
}
