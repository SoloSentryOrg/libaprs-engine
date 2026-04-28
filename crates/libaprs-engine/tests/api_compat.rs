use libaprs_engine::{
    parse_packet, parse_packet_with_options, AprsData, Counters, DataTypeIdentifier, Engine,
    EngineResult, LineTransport, MicEMessageCode, MicEStandardMessage, PacketSink, PacketSource,
    ParseError, ParseOptions, Policy, PolicyDecision, PolicyRejection, TransportErrorCode,
    DEFAULT_PARSE_OPTIONS, DEFAULT_TRANSPORT_READ_LIMIT, MAX_PACKET_LEN,
};

const _: () = assert!(DEFAULT_TRANSPORT_READ_LIMIT >= MAX_PACKET_LEN);

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
fn engine_can_process_packet_sources() {
    let mut engine = Engine::new(Policy::permissive());
    let mut source = LineTransport::new(b"N0CALL>APRS:>one\nN1CALL>APRS:~two\n");

    let results = engine.process_source(&mut source).expect("source batch");

    assert_eq!(results.len(), 2);
    assert_eq!(engine.counters().accepted, 2);
}
