use libaprs_engine::{
    parse_packet, parse_packet_with_options, support_matrix, AprsData, Counters,
    DataTypeIdentifier, DiagnosticLayer, Engine, EngineEvent, EngineEventKind, EngineResult,
    LineTransport, MicEMessageCode, MicEStandardMessage, PacketSink, PacketSource, ParseError,
    ParseOptions, Policy, PolicyDecision, PolicyRejection, SupportStatus, TransportErrorCode,
    DEFAULT_PARSE_OPTIONS, DEFAULT_TRANSPORT_READ_LIMIT, EVENT_RAW_BYTE_LIMIT, MAX_PACKET_LEN,
};

const _: () = assert!(DEFAULT_TRANSPORT_READ_LIMIT >= MAX_PACKET_LEN);
const _: () = assert!(EVENT_RAW_BYTE_LIMIT == MAX_PACKET_LEN + 1);

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
    assert_eq!(parsed.summary().semantic, "status");
}

#[cfg(feature = "serde")]
#[test]
fn stable_diagnostic_alternative_to_json_remains_usable() {
    use libaprs_engine::serde_support::PacketDiagnostic;

    let input = b"N0CALL>APRS:>\xff";
    let parsed = parse_packet(input).expect("packet should parse");
    let diagnostic = PacketDiagnostic::from_packet(&parsed);
    let explicit_diagnostic = parsed.to_diagnostic();

    assert_eq!(diagnostic.schema_version, 1);
    assert_eq!(diagnostic.raw, input);
    assert_eq!(explicit_diagnostic, diagnostic);
    assert_eq!(diagnostic.source, b"N0CALL");
    assert_eq!(diagnostic.destination, b"APRS");
    assert_eq!(diagnostic.payload, b">\xff");
    assert_eq!(diagnostic.data_type, "status");
    assert_eq!(diagnostic.semantic, "status");
}

#[test]
fn stable_intent_options_and_errors_remain_usable() {
    assert_eq!(MAX_PACKET_LEN, 512);
    assert_eq!(DEFAULT_PARSE_OPTIONS, ParseOptions::default());
    assert_eq!(ParseOptions::default().max_packet_len, MAX_PACKET_LEN);

    let err = parse_packet_with_options(b"N0CALL>APRS:>hello", ParseOptions::new(4))
        .expect_err("tight size limit should reject packet");

    assert_eq!(err, ParseError::Oversized);
    assert_eq!(err.code(), "parse.oversized");

    assert_eq!(ParseError::Empty.code(), "parse.empty");
    assert_eq!(
        ParseError::MissingSeparator.code(),
        "parse.missing_separator"
    );
    assert_eq!(ParseError::EmptySegment.code(), "parse.empty_segment");
    assert_eq!(ParseError::InvalidAddress.code(), "parse.invalid_address");
}

#[test]
fn structured_error_diagnostics_remain_usable() {
    let parse_diagnostic = ParseError::MissingSeparator.diagnostic();
    assert_eq!(parse_diagnostic.layer, DiagnosticLayer::Parse);
    assert_eq!(parse_diagnostic.layer.code(), "parse");
    assert_eq!(parse_diagnostic.code, "parse.missing_separator");
    assert_eq!(parse_diagnostic.name, "missing_separator");
    assert!(parse_diagnostic.description.contains("separator"));
    assert!(parse_diagnostic.remediation.contains("source>path:payload"));

    let policy_diagnostic = PolicyRejection::UnsupportedSemantics.diagnostic();
    assert_eq!(policy_diagnostic.layer, DiagnosticLayer::Policy);
    assert_eq!(policy_diagnostic.layer.code(), "policy");
    assert_eq!(policy_diagnostic.code, "policy.unsupported_semantics");
    assert_eq!(policy_diagnostic.name, "unsupported_semantics");
    assert!(policy_diagnostic.description.contains("unsupported"));

    let transport_diagnostic = TransportErrorCode::OversizedInput.diagnostic();
    assert_eq!(transport_diagnostic.layer, DiagnosticLayer::Transport);
    assert_eq!(transport_diagnostic.layer.code(), "transport");
    assert_eq!(transport_diagnostic.code, "transport.oversized_input");
    assert_eq!(transport_diagnostic.name, "oversized_input");
    assert!(transport_diagnostic.remediation.contains("bounded"));
}

#[test]
fn stable_support_matrix_api_remains_usable() {
    let matrix = support_matrix();

    assert_eq!(matrix.schema_version, 1);
    assert!(matrix.semantic_families.iter().any(|item| {
        item.kind == "status"
            && item.status == SupportStatus::Supported
            && item.notes.contains("preserved")
    }));
    assert!(matrix
        .transport_adapters
        .iter()
        .any(|item| item.crate_name == "aprs-transport-kiss"
            && item.status == SupportStatus::Supported
            && item.boundary.contains("KISS")));
    assert_eq!(
        matrix
            .diagnostic_layers
            .iter()
            .map(|layer| layer.code())
            .collect::<Vec<_>>(),
        vec!["parse", "policy", "transport"]
    );
}

#[test]
fn stable_intent_engine_policy_and_transport_api_remain_usable() {
    let mut engine = Engine::new(Policy::strict());
    assert_eq!(engine.counters(), Counters::default());

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
fn stable_intent_policy_decision_api_remains_usable() {
    let accepted = parse_packet(b"N0CALL>APRS:>ok").expect("status should parse");
    assert_eq!(
        Policy::strict().evaluate(&accepted, &accepted.aprs_data()),
        PolicyDecision::Accept
    );

    let unsupported = parse_packet(b"N0CALL>APRS:~opaque").expect("unsupported should parse");
    assert_eq!(
        Policy::strict().evaluate(&unsupported, &unsupported.aprs_data()),
        PolicyDecision::Reject(PolicyRejection::UnsupportedSemantics)
    );

    let malformed = parse_packet(b"N0CALL>APRS:!bad").expect("malformed semantic should parse");
    assert_eq!(
        Policy::strict().evaluate(&malformed, &malformed.aprs_data()),
        PolicyDecision::Reject(PolicyRejection::MalformedSemantics)
    );

    assert_eq!(PolicyRejection::PathTooLong.code(), "policy.path_too_long");
    assert_eq!(
        PolicyRejection::MalformedSemantics.code(),
        "policy.malformed_semantics"
    );
    assert_eq!(
        PolicyRejection::InvalidNmeaChecksum.code(),
        "policy.nmea_checksum_mismatch"
    );
}

#[test]
fn stable_intent_data_type_identifier_names_remain_usable() {
    let identifiers = [
        (
            b"!",
            DataTypeIdentifier::PositionNoTimestamp,
            "position_no_timestamp",
        ),
        (
            b"=",
            DataTypeIdentifier::PositionNoTimestampMessaging,
            "position_no_timestamp_messaging",
        ),
        (
            b"/",
            DataTypeIdentifier::PositionWithTimestamp,
            "position_with_timestamp",
        ),
        (
            b"@",
            DataTypeIdentifier::PositionWithTimestampMessaging,
            "position_with_timestamp_messaging",
        ),
        (b">", DataTypeIdentifier::Status, "status"),
        (b"?", DataTypeIdentifier::Query, "query"),
        (b"<", DataTypeIdentifier::Capability, "capability"),
        (b":", DataTypeIdentifier::Message, "message"),
        (b";", DataTypeIdentifier::Object, "object"),
        (b")", DataTypeIdentifier::Item, "item"),
        (b"_", DataTypeIdentifier::Weather, "weather"),
        (b"T", DataTypeIdentifier::Telemetry, "telemetry"),
        (b"$", DataTypeIdentifier::Nmea, "nmea"),
        (b"`", DataTypeIdentifier::MicECurrent, "mic_e_current"),
        (b"'", DataTypeIdentifier::MicEOld, "mic_e_old"),
        (b"[", DataTypeIdentifier::Maidenhead, "maidenhead"),
        (b"{", DataTypeIdentifier::UserDefined, "user_defined"),
        (b"}", DataTypeIdentifier::ThirdParty, "third_party"),
    ];

    for (identifier, expected, name) in identifiers {
        let mut packet = b"N0CALL>APRS:".to_vec();
        packet.extend_from_slice(identifier);
        packet.extend_from_slice(match identifier {
            b"!" | b"=" => b"4903.50N/07201.75W-comment".as_slice(),
            b"/" | b"@" => b"092345z4903.50N/07201.75W-comment".as_slice(),
            b":" => b"TARGET   :hello".as_slice(),
            b";" => b"OBJECT   *092345z4903.50N/07201.75W-comment".as_slice(),
            b")" => b"ITEM!4903.50N/07201.75W-comment".as_slice(),
            b"T" => b"#001,111,222,033,044,055,10101010".as_slice(),
            b"[" => b"AA00aa comment".as_slice(),
            b"{" => b"ABbody".as_slice(),
            _ => b"body".as_slice(),
        });

        let parsed = parse_packet(&packet).expect("identifier packet should parse");
        assert_eq!(parsed.data_type_identifier(), expected);
        assert_eq!(parsed.data_type_identifier().name(), name);
    }

    let unknown = parse_packet(b"N0CALL>APRS:~body").expect("unknown should parse");
    assert_eq!(
        unknown.data_type_identifier(),
        DataTypeIdentifier::Unknown(b'~')
    );
    assert_eq!(unknown.data_type_identifier().name(), "unknown");
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
    assert_eq!(nmea.talker_id(), Some(b"GP".as_slice()));
    assert_eq!(nmea.sentence_id(), Some(b"GLL".as_slice()));

    let object = parse_packet(b"N0CALL>APRS:;LEADER   *092345z4903.50N/07201.75W-object")
        .expect("object should parse");
    let AprsData::Object(object) = object.aprs_data() else {
        panic!("expected object");
    };
    assert!(object.coordinates().is_some());

    let weather_position = parse_packet(b"N0CALL>APRS:!4903.50N/07201.75W_c220s004t077")
        .expect("weather position should parse");
    let AprsData::Position(position) = weather_position.aprs_data() else {
        panic!("expected position");
    };
    assert_eq!(
        position
            .weather()
            .expect("weather report")
            .fields()
            .temperature_fahrenheit,
        Some(77)
    );

    let compressed_weather = parse_packet(b"N0CALL>APRS:!/5L!!<*e7_7P[c220s004t077")
        .expect("compressed weather position should parse");
    let AprsData::CompressedPosition(compressed) = compressed_weather.aprs_data() else {
        panic!("expected compressed position");
    };
    assert_eq!(
        compressed
            .weather()
            .expect("compressed weather report")
            .fields()
            .temperature_fahrenheit,
        Some(77)
    );

    let mic_e = parse_packet(b"N0CALL>ABC123:`abcde").expect("Mic-E should parse");
    let AprsData::MicE(mic_e) = mic_e.aprs_data() else {
        panic!("expected Mic-E");
    };
    assert_eq!(
        mic_e.message_code(),
        Some(MicEMessageCode::Standard(MicEStandardMessage::OffDuty))
    );
}

#[test]
fn structured_packet_summary_exposes_decoded_details() {
    let nmea = parse_packet(b"N0CALL>APRS:$GPGLL,4916.45,N,12311.12,W,225444,A,*1D")
        .expect("NMEA should parse");
    let summary = nmea.summary();

    assert_eq!(summary.source, b"N0CALL");
    assert_eq!(summary.destination, b"APRS");
    assert_eq!(summary.semantic, "nmea");
    assert!(summary.nmea_checksum.expect("checksum").valid);
    assert!(summary.coordinates.is_none());

    let position =
        parse_packet(b"N0CALL>APRS:!4903.50N/07201.75W-Test").expect("position should parse");
    let summary = position.summary();
    assert_eq!(summary.semantic, "position");
    assert!(summary.coordinates.is_some());
}

#[test]
fn transport_contracts_and_codes_remain_usable() {
    let mut source = LineTransport::new(b"N0CALL>APRS:>one\nN1CALL>APRS:>two\n");
    let packets = source.recv_packets().expect("line transport source");

    let mut sink = Vec::new();
    for packet in &packets {
        sink.send_packet(packet).expect("vec sink");
    }

    assert_eq!(sink, packets);
    assert_eq!(
        TransportErrorCode::OversizedInput.code(),
        "transport.oversized_input"
    );
}

#[test]
fn line_transport_can_fail_closed_on_oversized_packet_lines() {
    let transport = LineTransport::new(b"N0CALL>APRS:>one\nN1CALL>APRS:>two\n");
    let error = transport
        .packets_with_limit(b"N0CALL>APRS:>one".len() - 1)
        .expect_err("oversized packet line must fail closed");

    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    assert_eq!(error.to_string(), TransportErrorCode::OversizedInput.code());
}

#[test]
fn line_transport_packet_limit_preserves_non_utf8_bytes() {
    let packets = LineTransport::new(b"N0CALL>APRS:>\xff\r\n")
        .packets_with_limit(MAX_PACKET_LEN)
        .expect("valid packet should split");

    assert_eq!(packets, vec![b"N0CALL>APRS:>\xff".as_slice()]);
}

#[test]
fn engine_can_process_packet_sources() {
    let mut engine = Engine::new(Policy::permissive());
    let mut source = LineTransport::new(b"N0CALL>APRS:>one\nN1CALL>APRS:~two\n");

    let results = engine.process_source(&mut source).expect("source batch");

    assert_eq!(results.len(), 2);
    assert_eq!(engine.counters().accepted, 2);
}

#[test]
fn stable_observability_event_api_remains_usable() {
    assert_eq!(EngineEventKind::Accepted.code(), "accepted");
    assert_eq!(EngineEventKind::PolicyRejected.code(), "policy_rejected");
    assert_eq!(EngineEventKind::Malformed.code(), "malformed");
    assert_eq!(
        EngineEventKind::TransportFailure.code(),
        "transport_failure"
    );

    let mut engine = Engine::default();
    let event = engine.process_event(b"N0CALL>APRS:>ok");

    assert_eq!(event.kind(), EngineEventKind::Accepted);
    let EngineEvent::Accepted(accepted) = event else {
        panic!("expected accepted event");
    };
    assert_eq!(accepted.packet.summary().semantic, "status");
}

#[cfg(feature = "metrics")]
#[test]
fn metrics_feature_exports_counter_metrics_without_runtime_dependencies() {
    use libaprs_engine::metrics_support::{
        counter_metrics, record_counters, CounterMetric, MetricsRecorder, ACCEPTED_PACKETS_TOTAL,
        MALFORMED_PACKETS_TOTAL, REJECTED_PACKETS_TOTAL,
    };

    let counters = Counters {
        accepted: 1,
        rejected: 2,
        malformed: 3,
    };
    let metrics = counter_metrics(counters);

    assert_eq!(
        metrics,
        [
            CounterMetric {
                name: ACCEPTED_PACKETS_TOTAL,
                value: 1
            },
            CounterMetric {
                name: REJECTED_PACKETS_TOTAL,
                value: 2
            },
            CounterMetric {
                name: MALFORMED_PACKETS_TOTAL,
                value: 3
            },
        ]
    );

    #[derive(Default)]
    struct Recorder {
        metrics: Vec<CounterMetric>,
    }

    impl MetricsRecorder for Recorder {
        fn record_counter(&mut self, metric: CounterMetric) {
            self.metrics.push(metric);
        }
    }

    let mut recorder = Recorder::default();
    record_counters(&mut recorder, counters);

    assert_eq!(recorder.metrics, metrics);
}
