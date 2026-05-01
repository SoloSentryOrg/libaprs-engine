use libaprs_engine::{
    parse_packet, parse_packet_with_options, AprsData, Capability, CompressedPosition,
    DataTypeIdentifier, Item, Maidenhead, Message, MessageKind, MicE, MicEMessageCode,
    MicEStandardMessage, MicEStatus, Nmea, NmeaChecksum, Object, ParseError, ParseOptions,
    Position, Query, Telemetry, TelemetryMetadata, TelemetryMetadataKind, ThirdParty,
    TimestampedPosition, UserDefined, Weather, WeatherFields, MAX_PACKET_LEN,
};

#[test]
fn valid_packet_preserves_exact_raw_bytes() {
    let input = b"N0CALL>APRS,TCPIP*:>hello world";

    let parsed = parse_packet(input).expect("valid packet should parse");

    assert_eq!(parsed.raw().as_bytes(), input);
    assert_eq!(parsed.source(), b"N0CALL");
    assert_eq!(parsed.destination(), b"APRS");
    assert_eq!(parsed.digipeaters(), vec![b"TCPIP*".as_slice()]);
    assert_eq!(
        parsed.path_components(),
        vec![b"APRS".as_slice(), b"TCPIP*".as_slice()]
    );
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
fn separator_and_path_mutations_fail_closed() {
    let cases = [
        (
            b">APRS:>missing-source".as_slice(),
            ParseError::EmptySegment,
        ),
        (
            b"N0CALL>:>missing-path".as_slice(),
            ParseError::EmptySegment,
        ),
        (b"N0CALL>APRS:".as_slice(), ParseError::EmptySegment),
        (
            b"N0CALL>APRS,,WIDE1-1:>empty-path-component".as_slice(),
            ParseError::InvalidAddress,
        ),
        (
            b"N0CALL>APRS,WIDE1-16:>bad-ssid".as_slice(),
            ParseError::InvalidAddress,
        ),
        (
            b"N0CALL>APRS,WIDE1-1**,TCPIP:>bad-repeat-marker".as_slice(),
            ParseError::InvalidAddress,
        ),
    ];

    for (input, expected) in cases {
        assert_eq!(
            parse_packet(input).expect_err("mutated packet should fail closed"),
            expected,
            "mutation unexpectedly changed error for {input:?}"
        );
    }
}

#[test]
fn path_component_flood_fails_closed_without_partial_packet() {
    let mut input = b"N0CALL>APRS".to_vec();
    for _ in 0..128 {
        input.extend_from_slice(b",");
    }
    input.extend_from_slice(b":>path flood");

    let err = parse_packet(&input).expect_err("empty path components must fail closed");

    assert_eq!(err, ParseError::InvalidAddress);
}

#[test]
fn packet_with_non_ax25_like_source_fails_closed() {
    let err = parse_packet(b"N0 CALL>APRS:hello").expect_err("invalid source must be rejected");

    assert_eq!(err, ParseError::InvalidAddress);
}

#[test]
fn packet_with_non_ax25_like_path_fails_closed() {
    let err =
        parse_packet(b"N0CALL>APRS,\nTCPIP:hello").expect_err("invalid path must be rejected");

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
    let err = parse_packet(b"N0CALL>AP*RS:hello")
        .expect_err("misplaced repeated marker must be rejected");

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
    assert_eq!(
        parsed.data_type_identifier(),
        DataTypeIdentifier::PositionNoTimestamp
    );
    assert_eq!(parsed.information(), b"\xff\xfe\xfd");
}

#[test]
fn invalid_utf8_address_metadata_fails_closed() {
    let cases = [
        b"N0\xffCALL>APRS:>status".as_slice(),
        b"N0CALL>AP\xffRS:>status".as_slice(),
        b"N0CALL>APRS,WIDE\xff:>status".as_slice(),
    ];

    for input in cases {
        let err = parse_packet(input).expect_err("address bytes must be conservative ASCII");
        assert_eq!(err, ParseError::InvalidAddress, "{input:?}");
    }
}

#[test]
fn unknown_data_type_identifier_is_preserved_as_byte() {
    let input = b"N0CALL>APRS:~opaque";

    let parsed = parse_packet(input).expect("unknown data type byte is still structured");

    assert_eq!(
        parsed.data_type_identifier(),
        DataTypeIdentifier::Unknown(b'~')
    );
    assert_eq!(parsed.information(), b"opaque");
}

#[test]
fn status_semantics_preserve_status_text_bytes() {
    let parsed =
        parse_packet(b"N0CALL>APRS:>Running semantic parser").expect("status should parse");

    assert_eq!(
        parsed.aprs_data(),
        AprsData::Status {
            text: b"Running semantic parser".as_slice()
        }
    );
}

#[test]
fn uncompressed_position_semantics_parse_coordinates_and_comment() {
    let parsed = parse_packet(b"N0CALL>APRS:!4903.50N/07201.75W-Test comment")
        .expect("position should parse");

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
fn uncompressed_position_interprets_decimal_coordinates() {
    let parsed = parse_packet(b"N0CALL>APRS:!4903.50N/07201.75W-Test comment")
        .expect("position should parse");
    let AprsData::Position(position) = parsed.aprs_data() else {
        panic!("expected position");
    };

    let coordinates = position.coordinates().expect("coordinates should decode");

    assert_approx_eq(coordinates.latitude, 49.0583333333);
    assert_approx_eq(coordinates.longitude, -72.0291666667);
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
    let announcement =
        parse_packet(b"N0CALL>APRS::BLNA     :announcement").expect("announcement should parse");

    assert_eq!(message_kind(ack.aprs_data()), MessageKind::Ack);
    assert_eq!(message_kind(reject.aprs_data()), MessageKind::Reject);
    assert_eq!(message_kind(bulletin.aprs_data()), MessageKind::Bulletin);
    assert_eq!(
        message_kind(announcement.aprs_data()),
        MessageKind::Announcement
    );
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
fn object_and_item_coordinates_decode_from_position_body() {
    let object = parse_packet(b"N0CALL>APRS:;LEADER   *092345z4903.50N/07201.75W-object")
        .expect("object should parse");
    let item =
        parse_packet(b"N0CALL>APRS:)BIKE!4903.50N/07201.75W-item").expect("item should parse");

    let AprsData::Object(object) = object.aprs_data() else {
        panic!("expected object");
    };
    let AprsData::Item(item) = item.aprs_data() else {
        panic!("expected item");
    };

    let object_coordinates = object.coordinates().expect("object coordinates");
    let item_coordinates = item.coordinates().expect("item coordinates");

    assert_approx_eq(object_coordinates.latitude, 49.0583333333);
    assert_approx_eq(object_coordinates.longitude, -72.0291666667);
    assert_eq!(object_coordinates, item_coordinates);
}

#[test]
fn item_semantics_parse_name_liveness_and_body() {
    let parsed = parse_packet(b"N0CALL>APRS:)BIKE!4903.50N/07201.75W-").expect("item should parse");

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
    let parsed = parse_packet(b"N0CALL>APRS:/092345z4903.50N/07201.75W-Test comment")
        .expect("position should parse");

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
    let parsed = parse_packet(b"N0CALL>APRS:!/5L!!<*e7>7P[comment")
        .expect("compressed position should parse");

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
fn compressed_position_interprets_decimal_coordinates() {
    let parsed = parse_packet(b"N0CALL>APRS:!/5L!!<*e7>7P[comment")
        .expect("compressed position should parse");
    let AprsData::CompressedPosition(position) = parsed.aprs_data() else {
        panic!("expected compressed position");
    };

    let coordinates = position
        .coordinates()
        .expect("compressed coordinates should decode");

    assert_approx_eq(coordinates.latitude, 49.5);
    assert_approx_eq(coordinates.longitude, -72.75000394);
}

#[test]
fn compressed_position_weather_symbol_exposes_weather_report_from_comment() {
    let parsed = parse_packet(b"N0CALL>APRS:!/5L!!<*e7_7P[c220s004g010t077r001p002P003h50")
        .expect("compressed weather position should parse");
    let AprsData::CompressedPosition(position) = parsed.aprs_data() else {
        panic!("expected compressed position");
    };

    let weather = position
        .weather()
        .expect("compressed weather symbol position should expose weather report");

    assert_eq!(
        weather.report,
        b"c220s004g010t077r001p002P003h50".as_slice()
    );
    assert_eq!(weather.fields().temperature_fahrenheit, Some(77));
    assert_eq!(weather.fields().wind_direction_degrees, Some(220));
}

#[test]
fn short_uncompressed_position_is_malformed_not_panic() {
    let parsed = parse_packet(b"N0CALL>APRS:!4903.50N/07201.75W")
        .expect("framed packet should parse at codec boundary");

    assert_eq!(
        parsed.aprs_data(),
        AprsData::Malformed {
            identifier: b'!',
            information: b"4903.50N/07201.75W".as_slice(),
        }
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
fn weather_semantics_extract_numeric_fields() {
    let parsed =
        parse_packet(b"N0CALL>APRS:_092345c220s004g010t-05r001p002P003h50b10150L123l045S006#789")
            .expect("weather should parse");
    let AprsData::Weather(weather) = parsed.aprs_data() else {
        panic!("expected weather");
    };

    assert_eq!(
        weather.fields(),
        WeatherFields {
            timestamp: Some(b"092345".as_slice()),
            wind_direction_degrees: Some(220),
            wind_speed_mph: Some(4),
            wind_gust_mph: Some(10),
            temperature_fahrenheit: Some(-5),
            rain_last_hour_hundredths_inch: Some(1),
            rain_last_24_hours_hundredths_inch: Some(2),
            rain_since_midnight_hundredths_inch: Some(3),
            humidity_percent: Some(50),
            pressure_tenths_hpa: Some(10150),
            luminosity_watts_per_square_meter: Some(123),
            luminosity_1000_plus_watts_per_square_meter: Some(1045),
            snow_last_24_hours_inches: Some(6),
            raw_rain_counter: Some(789),
        }
    );
}

#[test]
fn weather_semantics_ignore_malformed_optional_fields() {
    let parsed = parse_packet(b"N0CALL>APRS:_abcdefcxxxs004g010t---r001p002P003h00b1015L12lxxS0#7")
        .expect("weather should parse");
    let AprsData::Weather(weather) = parsed.aprs_data() else {
        panic!("expected weather");
    };

    assert_eq!(
        weather.fields(),
        WeatherFields {
            timestamp: None,
            wind_direction_degrees: None,
            wind_speed_mph: Some(4),
            wind_gust_mph: Some(10),
            temperature_fahrenheit: None,
            rain_last_hour_hundredths_inch: Some(1),
            rain_last_24_hours_hundredths_inch: Some(2),
            rain_since_midnight_hundredths_inch: Some(3),
            humidity_percent: Some(100),
            pressure_tenths_hpa: None,
            luminosity_watts_per_square_meter: None,
            luminosity_1000_plus_watts_per_square_meter: None,
            snow_last_24_hours_inches: None,
            raw_rain_counter: None,
        }
    );
}

#[test]
fn position_weather_symbol_exposes_weather_report_from_comment() {
    let parsed =
        parse_packet(b"N0CALL>APRS:!4903.50N/07201.75W_c220s004g010t077r001p002P003h50b10150")
            .expect("weather position should parse");
    let AprsData::Position(position) = parsed.aprs_data() else {
        panic!("expected position");
    };

    let weather = position
        .weather()
        .expect("weather symbol position should expose weather report");

    assert_eq!(
        weather.report,
        b"c220s004g010t077r001p002P003h50b10150".as_slice()
    );
    assert_eq!(weather.fields().temperature_fahrenheit, Some(77));
    assert_eq!(weather.fields().wind_direction_degrees, Some(220));
}

#[test]
fn timestamped_position_object_and_item_expose_embedded_weather_reports() {
    let timestamped = parse_packet(b"N0CALL>APRS:/092345z4903.50N/07201.75W_c220s004g010t077")
        .expect("timestamped weather position should parse");
    let object = parse_packet(b"N0CALL>APRS:;WXOBJ    *092345z4903.50N/07201.75W_c180s002t055")
        .expect("weather object should parse");
    let item = parse_packet(b"N0CALL>APRS:)WXITEM!4903.50N/07201.75W_c090s001t060")
        .expect("weather item should parse");

    let AprsData::TimestampedPosition(timestamped) = timestamped.aprs_data() else {
        panic!("expected timestamped position");
    };
    let AprsData::Object(object) = object.aprs_data() else {
        panic!("expected object");
    };
    let AprsData::Item(item) = item.aprs_data() else {
        panic!("expected item");
    };

    assert_eq!(
        timestamped
            .weather()
            .expect("timestamped weather")
            .fields()
            .temperature_fahrenheit,
        Some(77)
    );
    assert_eq!(
        object
            .weather()
            .expect("object weather")
            .fields()
            .temperature_fahrenheit,
        Some(55)
    );
    assert_eq!(
        item.weather()
            .expect("item weather")
            .fields()
            .temperature_fahrenheit,
        Some(60)
    );
}

#[test]
fn object_and_item_expose_compressed_embedded_weather_reports() {
    let object = parse_packet(b"N0CALL>APRS:;CWXOBJ   *092345z/5L!!<*e7_7P[c180s002t055")
        .expect("compressed weather object should parse");
    let item = parse_packet(b"N0CALL>APRS:)CWXITEM!/5L!!<*e7_7P[c090s001t060")
        .expect("compressed weather item should parse");

    let AprsData::Object(object) = object.aprs_data() else {
        panic!("expected object");
    };
    let AprsData::Item(item) = item.aprs_data() else {
        panic!("expected item");
    };

    assert_eq!(
        object
            .weather()
            .expect("compressed object weather")
            .fields()
            .temperature_fahrenheit,
        Some(55)
    );
    assert_eq!(
        item.weather()
            .expect("compressed item weather")
            .fields()
            .temperature_fahrenheit,
        Some(60)
    );
}

#[test]
fn non_weather_position_symbol_does_not_expose_weather_report() {
    let parsed = parse_packet(b"N0CALL>APRS:!4903.50N/07201.75W-plain comment")
        .expect("position should parse");
    let AprsData::Position(position) = parsed.aprs_data() else {
        panic!("expected position");
    };

    assert!(position.weather().is_none());
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
fn telemetry_semantics_extract_numeric_values() {
    let parsed = parse_packet(b"N0CALL>APRS:T#001,111,222,033,044,055,10101010")
        .expect("telemetry should parse");
    let AprsData::Telemetry(telemetry) = parsed.aprs_data() else {
        panic!("expected telemetry");
    };

    assert_eq!(telemetry.sequence_number(), Some(1));
    assert_eq!(telemetry.analog_values(), Some([111, 222, 33, 44, 55]));
    assert_eq!(
        telemetry.digital_bits(),
        Some([true, false, true, false, true, false, true, false])
    );
}

#[test]
fn telemetry_metadata_packets_are_modeled_by_kind_and_fields() {
    let parameters =
        parse_packet(b"N0CALL>APRS::PARM.    :Vbat,Temp,Pressure").expect("PARM should parse");
    let units = parse_packet(b"N0CALL>APRS::UNIT.    :V,C,hPa").expect("UNIT should parse");
    let equations = parse_packet(b"N0CALL>APRS::EQNS.    :0,1,0,0,1,0").expect("EQNS should parse");
    let bits = parse_packet(b"N0CALL>APRS::BITS.    :10101010,Flags").expect("BITS should parse");

    assert_eq!(
        parameters.aprs_data(),
        AprsData::TelemetryMetadata(TelemetryMetadata {
            addressee: b"PARM.    ".as_slice(),
            kind: TelemetryMetadataKind::ParameterNames,
            body: b"Vbat,Temp,Pressure".as_slice(),
        })
    );
    assert_eq!(
        telemetry_metadata_kind(units.aprs_data()),
        TelemetryMetadataKind::Units
    );
    assert_eq!(
        telemetry_metadata_kind(equations.aprs_data()),
        TelemetryMetadataKind::Equations
    );
    assert_eq!(
        telemetry_metadata_kind(bits.aprs_data()),
        TelemetryMetadataKind::BitSense
    );

    let AprsData::TelemetryMetadata(metadata) = parameters.aprs_data() else {
        panic!("expected telemetry metadata");
    };
    assert_eq!(
        metadata.fields(),
        vec![
            b"Vbat".as_slice(),
            b"Temp".as_slice(),
            b"Pressure".as_slice()
        ]
    );
}

#[test]
fn query_and_capability_semantics_preserve_query_bytes() {
    let query = parse_packet(b"N0CALL>APRS:?APRS?").expect("query should parse");
    let capability =
        parse_packet(b"N0CALL>APRS:<IGATE,MSG_CNT=1").expect("capability should parse");

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
    let third_party =
        parse_packet(b"N0CALL>APRS:}SRC>APRS:>nested").expect("third-party should parse");

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
            destination: b"APRS".as_slice(),
            body: b"abcde".as_slice(),
            status: None,
            latitude_digits: None,
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

#[test]
fn nmea_checksum_validation_reports_expected_and_calculated_values() {
    let parsed = parse_packet(b"N0CALL>APRS:$GPGLL,4916.45,N,12311.12,W,225444,A,*1D")
        .expect("NMEA should parse");
    let AprsData::Nmea(nmea) = parsed.aprs_data() else {
        panic!("expected NMEA");
    };

    assert_eq!(
        nmea.checksum(),
        Some(NmeaChecksum {
            expected: 0x1d,
            calculated: 0x1d,
            valid: true,
        })
    );

    let invalid = parse_packet(b"N0CALL>APRS:$GPGLL,4916.45,N,12311.12,W,225444,A,*00")
        .expect("NMEA should parse even with invalid checksum");
    let AprsData::Nmea(nmea) = invalid.aprs_data() else {
        panic!("expected NMEA");
    };
    assert_eq!(
        nmea.checksum(),
        Some(NmeaChecksum {
            expected: 0,
            calculated: 0x1d,
            valid: false,
        })
    );
}

#[test]
fn nmea_semantics_expose_sentence_identifiers_and_fields() {
    let parsed = parse_packet(b"N0CALL>APRS:$GPGLL,4916.45,N,12311.12,W,225444,A,*1D")
        .expect("NMEA should parse");
    let AprsData::Nmea(nmea) = parsed.aprs_data() else {
        panic!("expected NMEA");
    };

    assert_eq!(nmea.talker_id(), Some(b"GP".as_slice()));
    assert_eq!(nmea.sentence_id(), Some(b"GLL".as_slice()));
    assert_eq!(
        nmea.data_fields(),
        vec![
            b"4916.45".as_slice(),
            b"N".as_slice(),
            b"12311.12".as_slice(),
            b"W".as_slice(),
            b"225444".as_slice(),
            b"A".as_slice(),
            b"".as_slice(),
        ]
    );
}

#[test]
fn mic_e_semantics_extract_destination_derived_status_and_latitude_digits() {
    let mic_e = parse_packet(b"N0CALL>ABC123:`abcde").expect("Mic-E should parse");

    assert_eq!(
        mic_e.aprs_data(),
        AprsData::MicE(MicE {
            identifier: b'`',
            destination: b"ABC123".as_slice(),
            body: b"abcde".as_slice(),
            status: Some(MicEStatus::Custom([true, true, true])),
            latitude_digits: Some([0, 1, 2, 1, 2, 3]),
        })
    );

    let AprsData::MicE(mic_e) = mic_e.aprs_data() else {
        panic!("expected Mic-E");
    };
    assert_eq!(
        mic_e.message_code(),
        Some(MicEMessageCode::Standard(MicEStandardMessage::OffDuty))
    );
}

#[test]
fn mic_e_semantics_decode_custom_and_emergency_message_codes() {
    let custom = parse_packet(b"N0CALL>PQR123:`abcde").expect("custom Mic-E should parse");
    let emergency = parse_packet(b"N0CALL>123123:`abcde").expect("emergency Mic-E should parse");

    let AprsData::MicE(custom) = custom.aprs_data() else {
        panic!("expected custom Mic-E");
    };
    let AprsData::MicE(emergency) = emergency.aprs_data() else {
        panic!("expected emergency Mic-E");
    };

    assert_eq!(custom.message_code(), Some(MicEMessageCode::Custom(0)));
    assert_eq!(emergency.message_code(), Some(MicEMessageCode::Emergency));
}

#[test]
fn mic_e_semantics_decode_position_speed_and_course_when_available() {
    let mic_e = parse_packet(b"N0CALL>ABC123:`a\x1d\x1dDEF").expect("Mic-E should parse");
    let AprsData::MicE(mic_e) = mic_e.aprs_data() else {
        panic!("expected Mic-E");
    };

    let coordinates = mic_e
        .coordinates()
        .expect("Mic-E coordinates should decode");
    assert_approx_eq(coordinates.latitude, -1.3538333333);
    assert_approx_eq(coordinates.longitude, 69.0168333333);
    assert_eq!(
        mic_e.speed_course(),
        Some(libaprs_engine::MicESpeedCourse {
            speed_knots: 404,
            course_degrees: 142,
        })
    );
}

#[test]
fn third_party_semantics_can_parse_nested_packet_explicitly() {
    let parsed = parse_packet(b"N0CALL>APRS:}SRC>APRS:>nested").expect("third-party should parse");
    let AprsData::ThirdParty(third_party) = parsed.aprs_data() else {
        panic!("expected third-party");
    };

    let nested = third_party
        .nested_packet()
        .expect("nested packet should parse explicitly");
    assert_eq!(nested.source(), b"SRC");
    assert_eq!(nested.destination(), b"APRS");
    assert_eq!(
        nested.aprs_data(),
        AprsData::Status {
            text: b"nested".as_slice()
        }
    );
}

#[test]
fn malformed_semantic_payloads_remain_policy_visible_without_partial_success() {
    let cases = [
        (b"N0CALL>APRS:!9903.50N/07201.75W-".as_slice(), b'!'),
        (b"N0CALL>APRS:!9001.00N/07201.75W-".as_slice(), b'!'),
        (b"N0CALL>APRS:!4903.50N/18001.00W-".as_slice(), b'!'),
        (b"N0CALL>APRS:/badtime4903.50N/07201.75W-".as_slice(), b'/'),
        (b"N0CALL>APRS:!/5L!!<*e7>7P\x7fcomment".as_slice(), b'!'),
        (b"N0CALL>APRS::TOOSHORT".as_slice(), b':'),
        (
            b"N0CALL>APRS:;LEADER   *badtime4903.50N/07201.75W-".as_slice(),
            b';',
        ),
        (
            b"N0CALL>APRS:)TOO-LONG-NAME!4903.50N/07201.75W-".as_slice(),
            b')',
        ),
        (b"N0CALL>APRS:T#001,111".as_slice(), b'T'),
        (b"N0CALL>ABC123:`ab".as_slice(), b'`'),
        (b"N0CALL>APRS:[IO91".as_slice(), b'['),
        (b"N0CALL>APRS:[9911!!bad locator".as_slice(), b'['),
        (b"N0CALL>APRS:{Q".as_slice(), b'{'),
    ];

    for (input, identifier) in cases {
        let parsed = parse_packet(input).expect("codec framing should parse");
        assert!(
            matches!(parsed.aprs_data(), AprsData::Malformed { identifier: actual, .. } if actual == identifier),
            "expected malformed semantic identifier {identifier:?} for {input:?}, got {:?}",
            parsed.aprs_data()
        );
        assert_eq!(parsed.raw().as_bytes(), input);
    }
}

fn message_kind(data: AprsData<'_>) -> MessageKind {
    match data {
        AprsData::Message(message) => message.kind,
        other => panic!("expected message, got {other:?}"),
    }
}

fn telemetry_metadata_kind(data: AprsData<'_>) -> TelemetryMetadataKind {
    match data {
        AprsData::TelemetryMetadata(metadata) => metadata.kind,
        other => panic!("expected telemetry metadata, got {other:?}"),
    }
}

fn assert_approx_eq(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 0.000_001,
        "expected {expected}, got {actual}"
    );
}

#[test]
fn oversized_packet_is_rejected() {
    let input = vec![b'A'; MAX_PACKET_LEN + 1];

    let err = parse_packet(&input).expect_err("oversized packets must be rejected");

    assert_eq!(err, ParseError::Oversized);
}

#[test]
fn custom_parse_options_can_tighten_packet_size_limit() {
    let input = b"N0CALL>APRS:>ok";
    let err = parse_packet_with_options(input, ParseOptions::new(input.len() - 1))
        .expect_err("custom max length should reject packet");

    assert_eq!(err, ParseError::Oversized);
    assert_eq!(err.code(), "parse.oversized");
}

#[test]
fn packet_length_boundary_is_exact_and_fail_closed() {
    let mut exact = b"N0CALL>APRS:".to_vec();
    exact.resize(MAX_PACKET_LEN, b'A');
    let parsed = parse_packet(&exact).expect("exact maximum packet size should parse");
    assert_eq!(parsed.raw().as_bytes(), exact.as_slice());

    let mut oversized = exact;
    oversized.push(b'A');
    assert_eq!(
        parse_packet(&oversized).expect_err("one byte over maximum must fail"),
        ParseError::Oversized
    );

    assert_eq!(
        parse_packet_with_options(b"N0CALL>APRS:>x", ParseOptions::new(0))
            .expect_err("zero max length rejects non-empty input"),
        ParseError::Oversized
    );
}
