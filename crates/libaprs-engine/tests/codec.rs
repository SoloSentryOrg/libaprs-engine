use libaprs_engine::{
    parse_packet, AprsData, Capability, CompressedPosition, DataTypeIdentifier, Item, Maidenhead,
    Message, MessageKind, MicE, Nmea, Object, ParseError, Position, Query, Telemetry,
    ThirdParty, TimestampedPosition, UserDefined, Weather, MAX_PACKET_LEN,
};

#[test]
fn valid_packet_preserves_exact_raw_bytes() {
    let input = b"N0CALL>APRS,TCPIP*:>hello world";

    let parsed = parse_packet(input).expect("valid packet should parse");

    assert_eq!(parsed.raw().as_bytes(), input);
    assert_eq!(parsed.source(), b"N0CALL");
    assert_eq!(parsed.destination(), b"APRS");
    assert_eq!(parsed.digipeaters(), vec![b"TCPIP*".as_slice()]);
    assert_eq!(parsed.path_components(), vec![b"APRS".as_slice(), b"TCPIP*".as_slice()]);
    assert_eq!(parsed.path(), b"APRS,TCPIP*");
    assert_eq!(parsed.payload(), b">hello world");
    assert_eq!(parsed.data_type_identifier(), DataTypeIdentifier::Status);
    assert_eq!(parsed.information(), b"hello world");
}

#[test]
fn packet_without_digipeaters_returns_empty_digipeater_list() {
    let input = b"N0CALL>APRS:hello";

    let parsed = parse_packet(input).expect("valid direct packet should parse");

    assert_eq!(parsed.destination(), b"APRS");
    assert!(parsed.digipeaters().is_empty());
    assert_eq!(parsed.path_components(), vec![b"APRS".as_slice()]);
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
fn packet_with_non_ax25_like_source_fails_closed() {
    let err = parse_packet(b"N0 CALL>APRS:hello").expect_err("invalid source must be rejected");

    assert_eq!(err, ParseError::InvalidAddress);
}

#[test]
fn packet_with_non_ax25_like_path_fails_closed() {
    let err = parse_packet(b"N0CALL>APRS,\nTCPIP:hello").expect_err("invalid path must be rejected");

    assert_eq!(err, ParseError::InvalidAddress);
}

#[test]
fn address_callsign_longer_than_six_bytes_fails_closed() {
    let err = parse_packet(b"TOOLONG>APRS:hello").expect_err("long callsign must be rejected");

    assert_eq!(err, ParseError::InvalidAddress);
}

#[test]
fn address_with_out_of_range_ssid_fails_closed() {
    let err = parse_packet(b"N0CALL-16>APRS:hello").expect_err("SSID above 15 must be rejected");

    assert_eq!(err, ParseError::InvalidAddress);
}

#[test]
fn lowercase_address_metadata_fails_closed() {
    let err = parse_packet(b"n0call>APRS:hello").expect_err("lowercase source must be rejected");

    assert_eq!(err, ParseError::InvalidAddress);
}

#[test]
fn repeated_marker_inside_address_fails_closed() {
    let err = parse_packet(b"N0CALL>AP*RS:hello").expect_err("misplaced repeated marker must be rejected");

    assert_eq!(err, ParseError::InvalidAddress);
}

#[test]
fn valid_ssid_and_repeated_path_marker_parse() {
    let input = b"N0CALL-7>APRS,WIDE1-1*:hello";

    let parsed = parse_packet(input).expect("valid SSID and path marker should parse");

    assert_eq!(parsed.source(), b"N0CALL-7");
    assert_eq!(parsed.destination(), b"APRS");
    assert_eq!(parsed.digipeaters(), vec![b"WIDE1-1*".as_slice()]);
    assert_eq!(parsed.path(), b"APRS,WIDE1-1*");
}

#[test]
fn invalid_utf8_payload_preserves_raw_bytes_and_does_not_panic() {
    let input = b"N0CALL>APRS:!\xff\xfe\xfd";

    let parsed = parse_packet(input).expect("payload bytes are opaque");

    assert_eq!(parsed.raw().as_bytes(), input);
    assert_eq!(parsed.payload(), b"!\xff\xfe\xfd");
    assert_eq!(parsed.data_type_identifier(), DataTypeIdentifier::PositionNoTimestamp);
    assert_eq!(parsed.information(), b"\xff\xfe\xfd");
}

#[test]
fn unknown_data_type_identifier_is_preserved_as_byte() {
    let input = b"N0CALL>APRS:~opaque";

    let parsed = parse_packet(input).expect("unknown data type byte is still structured");

    assert_eq!(parsed.data_type_identifier(), DataTypeIdentifier::Unknown(b'~'));
    assert_eq!(parsed.information(), b"opaque");
}

#[test]
fn status_semantics_preserve_status_text_bytes() {
    let parsed = parse_packet(b"N0CALL>APRS:>Running semantic parser").expect("status should parse");

    assert_eq!(
        parsed.aprs_data(),
        AprsData::Status {
            text: b"Running semantic parser".as_slice()
        }
    );
}

#[test]
fn uncompressed_position_semantics_parse_coordinates_and_comment() {
    let parsed =
        parse_packet(b"N0CALL>APRS:!4903.50N/07201.75W-Test comment").expect("position should parse");

    assert_eq!(
        parsed.aprs_data(),
        AprsData::Position(Position {
            messaging: false,
            latitude: b"4903.50N".as_slice(),
            symbol_table: b'/',
            longitude: b"07201.75W".as_slice(),
            symbol_code: b'-',
            comment: b"Test comment".as_slice(),
        })
    );
}

#[test]
fn message_semantics_parse_addressee_text_and_message_id() {
    let parsed =
        parse_packet(b"N0CALL>APRS::TARGET   :hello world{42").expect("message should parse");

    assert_eq!(
        parsed.aprs_data(),
        AprsData::Message(Message {
            addressee: b"TARGET   ".as_slice(),
            kind: MessageKind::Message,
            text: b"hello world".as_slice(),
            id: Some(b"42".as_slice()),
        })
    );
}

#[test]
fn message_semantics_classify_ack_reject_bulletin_and_announcement() {
    let ack = parse_packet(b"N0CALL>APRS::TARGET   :ack42").expect("ack should parse");
    let reject = parse_packet(b"N0CALL>APRS::TARGET   :rej42").expect("reject should parse");
    let bulletin = parse_packet(b"N0CALL>APRS::BLN1     :bulletin").expect("bulletin should parse");
    let announcement = parse_packet(b"N0CALL>APRS::BLNA     :announcement").expect("announcement should parse");

    assert_eq!(message_kind(ack.aprs_data()), MessageKind::Ack);
    assert_eq!(message_kind(reject.aprs_data()), MessageKind::Reject);
    assert_eq!(message_kind(bulletin.aprs_data()), MessageKind::Bulletin);
    assert_eq!(message_kind(announcement.aprs_data()), MessageKind::Announcement);
}

#[test]
fn unknown_semantics_preserve_identifier_and_information_bytes() {
    let parsed = parse_packet(b"N0CALL>APRS:~opaque").expect("unknown payload should parse");

    assert_eq!(
        parsed.aprs_data(),
        AprsData::Unsupported {
            identifier: b'~',
            information: b"opaque".as_slice(),
        }
    );
}

#[test]
fn object_semantics_parse_name_liveness_timestamp_and_body() {
    let parsed = parse_packet(b"N0CALL>APRS:;LEADER   *092345z4903.50N/07201.75W-")
        .expect("object should parse");

    assert_eq!(
        parsed.aprs_data(),
        AprsData::Object(Object {
            name: b"LEADER   ".as_slice(),
            live: true,
            timestamp: b"092345z".as_slice(),
            body: b"4903.50N/07201.75W-".as_slice(),
        })
    );
}

#[test]
fn item_semantics_parse_name_liveness_and_body() {
    let parsed = parse_packet(b"N0CALL>APRS:)BIKE!4903.50N/07201.75W-")
        .expect("item should parse");

    assert_eq!(
        parsed.aprs_data(),
        AprsData::Item(Item {
            name: b"BIKE".as_slice(),
            live: true,
            body: b"4903.50N/07201.75W-".as_slice(),
        })
    );
}

#[test]
fn timestamped_position_semantics_parse_timestamp_coordinates_and_comment() {
    let parsed =
        parse_packet(b"N0CALL>APRS:/092345z4903.50N/07201.75W-Test comment").expect("position should parse");

    assert_eq!(
        parsed.aprs_data(),
        AprsData::TimestampedPosition(TimestampedPosition {
            messaging: false,
            timestamp: b"092345z".as_slice(),
            position: Position {
                messaging: false,
                latitude: b"4903.50N".as_slice(),
                symbol_table: b'/',
                longitude: b"07201.75W".as_slice(),
                symbol_code: b'-',
                comment: b"Test comment".as_slice(),
            },
        })
    );
}

#[test]
fn compressed_position_semantics_preserve_compressed_fields() {
    let parsed = parse_packet(b"N0CALL>APRS:!/5L!!<*e7>7P[comment").expect("compressed position should parse");

    assert_eq!(
        parsed.aprs_data(),
        AprsData::CompressedPosition(CompressedPosition {
            messaging: false,
            symbol_table: b'/',
            compressed_latitude: b"5L!!".as_slice(),
            compressed_longitude: b"<*e7".as_slice(),
            symbol_code: b'>',
            extension: b"7P".as_slice(),
            compression_type: b'[',
            comment: b"comment".as_slice(),
        })
    );
}

#[test]
fn weather_semantics_preserve_weather_bytes() {
    let parsed = parse_packet(b"N0CALL>APRS:_092345c220s004g010t077r000p000P000h50b10150")
        .expect("weather should parse");

    assert_eq!(
        parsed.aprs_data(),
        AprsData::Weather(Weather {
            report: b"092345c220s004g010t077r000p000P000h50b10150".as_slice(),
        })
    );
}

#[test]
fn telemetry_semantics_parse_sequence_values_and_bits() {
    let parsed = parse_packet(b"N0CALL>APRS:T#001,111,222,033,044,055,10101010")
        .expect("telemetry should parse");

    assert_eq!(
        parsed.aprs_data(),
        AprsData::Telemetry(Telemetry {
            sequence: b"001".as_slice(),
            analog: [
                b"111".as_slice(),
                b"222".as_slice(),
                b"033".as_slice(),
                b"044".as_slice(),
                b"055".as_slice(),
            ],
            digital: Some(b"10101010".as_slice()),
        })
    );
}

#[test]
fn query_and_capability_semantics_preserve_query_bytes() {
    let query = parse_packet(b"N0CALL>APRS:?APRS?").expect("query should parse");
    let capability = parse_packet(b"N0CALL>APRS:<IGATE,MSG_CNT=1").expect("capability should parse");

    assert_eq!(
        query.aprs_data(),
        AprsData::Query(Query {
            query: b"APRS?".as_slice(),
        })
    );
    assert_eq!(
        capability.aprs_data(),
        AprsData::Capability(Capability {
            body: b"IGATE,MSG_CNT=1".as_slice(),
        })
    );
}

#[test]
fn nmea_mice_maidenhead_user_defined_and_third_party_semantics_parse() {
    let nmea = parse_packet(b"N0CALL>APRS:$GPGGA,123519,4807.038,N,01131.000,E,1")
        .expect("NMEA should parse");
    let mic_e = parse_packet(b"N0CALL>APRS:`abcde").expect("Mic-E should parse");
    let maidenhead = parse_packet(b"N0CALL>APRS:[IO91wm]").expect("Maidenhead should parse");
    let user_defined = parse_packet(b"N0CALL>APRS:{Q1payload").expect("user-defined should parse");
    let third_party = parse_packet(b"N0CALL>APRS:}SRC>APRS:>nested").expect("third-party should parse");

    assert_eq!(
        nmea.aprs_data(),
        AprsData::Nmea(Nmea {
            sentence: b"GPGGA,123519,4807.038,N,01131.000,E,1".as_slice(),
        })
    );
    assert_eq!(
        mic_e.aprs_data(),
        AprsData::MicE(MicE {
            identifier: b'`',
            body: b"abcde".as_slice(),
        })
    );
    assert_eq!(
        maidenhead.aprs_data(),
        AprsData::Maidenhead(Maidenhead {
            locator: b"IO91wm".as_slice(),
            comment: b"]".as_slice(),
        })
    );
    assert_eq!(
        user_defined.aprs_data(),
        AprsData::UserDefined(UserDefined {
            user_id: b'Q',
            packet_type: b'1',
            body: b"payload".as_slice(),
        })
    );
    assert_eq!(
        third_party.aprs_data(),
        AprsData::ThirdParty(ThirdParty {
            body: b"SRC>APRS:>nested".as_slice(),
        })
    );
}

fn message_kind(data: AprsData<'_>) -> MessageKind {
    match data {
        AprsData::Message(message) => message.kind,
        other => panic!("expected message, got {other:?}"),
    }
}

#[test]
fn oversized_packet_is_rejected() {
    let input = vec![b'A'; MAX_PACKET_LEN + 1];

    let err = parse_packet(&input).expect_err("oversized packets must be rejected");

    assert_eq!(err, ParseError::Oversized);
}
