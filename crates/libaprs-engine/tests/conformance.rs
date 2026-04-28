use std::collections::BTreeSet;

use libaprs_engine::{
    parse_packet, AprsData, Engine, EngineResult, MessageKind, Policy, PolicyRejection,
    TelemetryMetadataKind,
};

#[test]
fn valid_fixture_corpus_parses_without_losing_raw_bytes() {
    for (index, line) in include_str!("fixtures/valid.aprs").lines().enumerate() {
        let packet = line.as_bytes();
        let parsed = parse_packet(packet).unwrap_or_else(|err| {
            panic!("valid fixture line {} failed: {err:?}", index + 1);
        });

        assert_eq!(parsed.raw().as_bytes(), packet);
        assert!(!matches!(parsed.aprs_data(), AprsData::Malformed { .. }));
    }
}

#[test]
fn aprs101_fixture_corpus_parses_without_losing_raw_bytes() {
    for fixture in aprs101_valid_fixtures() {
        let parsed = parse_packet(fixture.packet).unwrap_or_else(|err| {
            panic!("APRS101 fixture {} failed: {err:?}", fixture.id);
        });

        assert_eq!(parsed.raw().as_bytes(), fixture.packet);
        assert!(
            !matches!(parsed.aprs_data(), AprsData::Malformed { .. }),
            "APRS101 fixture {} produced malformed semantics",
            fixture.id
        );
    }
}

#[test]
fn aprs101_fixture_corpus_has_source_references() {
    let referenced_ids = aprs101_source_reference_ids();
    let mut seen_ids = BTreeSet::new();

    for fixture in aprs101_valid_fixtures() {
        assert!(
            seen_ids.insert(fixture.id),
            "APRS101 fixture {} is duplicated",
            fixture.id
        );
        assert!(
            referenced_ids.contains(fixture.id),
            "APRS101 fixture {} is missing a source reference",
            fixture.id
        );
    }
}

#[test]
fn aprs101_fixture_corpus_keeps_minimum_family_coverage() {
    let fixtures = aprs101_valid_fixtures();
    let semantic_families = fixtures
        .iter()
        .map(|fixture| {
            parse_packet(fixture.packet)
                .expect("fixture should parse")
                .aprs_data()
                .kind_name()
        })
        .collect::<BTreeSet<_>>();

    assert!(
        fixtures.len() >= 30,
        "APRS101 fixture count regressed below expected coverage"
    );
    assert!(
        semantic_families.len() >= 18,
        "APRS101 semantic family coverage regressed: {semantic_families:?}"
    );
}

#[test]
fn malformed_fixture_corpus_fails_closed_at_codec_boundary() {
    for (index, line) in include_str!("fixtures/malformed.aprs").lines().enumerate() {
        let packet = line.as_bytes();
        assert!(
            parse_packet(packet).is_err(),
            "malformed fixture line {} unexpectedly parsed",
            index + 1
        );
    }
}

#[test]
fn malformed_semantic_fixtures_stay_visible_after_codec_parse() {
    for fixture in aprs101_malformed_semantic_fixtures() {
        let parsed = parse_packet(fixture.packet).unwrap_or_else(|err| {
            panic!(
                "malformed semantic fixture {} failed codec: {err:?}",
                fixture.id
            );
        });
        assert_eq!(parsed.raw().as_bytes(), fixture.packet);
        assert!(
            matches!(parsed.aprs_data(), AprsData::Malformed { .. }),
            "expected malformed semantic payload for {}, got {:?}",
            fixture.id,
            parsed.aprs_data()
        );
    }
}

#[test]
fn strict_policy_rejects_all_malformed_semantic_families_by_default() {
    let mut engine = Engine::new(Policy::strict());

    for fixture in aprs101_malformed_semantic_fixtures() {
        match engine.process(fixture.packet) {
            EngineResult::Rejected { reason, packet } => {
                assert_eq!(reason, PolicyRejection::MalformedSemantics);
                assert_eq!(packet.raw().as_bytes(), fixture.packet);
            }
            other => panic!(
                "expected strict malformed semantic rejection for {}, got {other:?}",
                fixture.id
            ),
        }
    }
}

#[test]
fn malformed_semantic_fixture_identifiers_match_payloads() {
    for fixture in aprs101_malformed_semantic_fixtures() {
        let parsed = parse_packet(fixture.packet).expect("codec framing should parse");
        assert!(
            matches!(parsed.aprs_data(), AprsData::Malformed { identifier, .. } if identifier == fixture.identifier),
            "expected malformed identifier {:?} for {}, got {:?}",
            fixture.identifier,
            fixture.id,
            parsed.aprs_data()
        );
    }
}

#[test]
fn byte_fuzz_inputs_never_panic_and_preserve_successful_raw_bytes() {
    let mut state = 0x5eed_u32;
    for len in 0..=128 {
        for _ in 0..32 {
            let mut input = Vec::with_capacity(len);
            for _ in 0..len {
                state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                input.push((state >> 24) as u8);
            }

            if let Ok(parsed) = parse_packet(&input) {
                assert_eq!(parsed.raw().as_bytes(), input.as_slice());
            }
        }
    }
}

#[test]
fn deterministic_mutation_corpus_never_panics_or_loses_successful_raw_bytes() {
    let seeds: [&[u8]; 6] = [
        b"N0CALL>APRS:>status",
        b"N0CALL-15>APRS,WIDE1-1*:!4903.50N/07201.75W-ok",
        b"N0CALL>APRS:T#001,111,222,033,044,055,10101010",
        b"N0CALL>APRS:$GPGLL,4916.45,N,12311.12,W,225444,A,*00",
        b"N0CALL>APRS:}SRC>APRS:>nested",
        b"N0CALL>APRS:!\xff\xfe\xfd",
    ];
    let mutations = [0x00, b'>', b':', b',', b'*', b'-', b'\r', b'\n', 0x7f, 0xff];

    for seed in seeds {
        for index in 0..seed.len() {
            for mutation in mutations {
                let mut input = seed.to_vec();
                input[index] = mutation;

                if let Ok(parsed) = parse_packet(&input) {
                    assert_eq!(parsed.raw().as_bytes(), input.as_slice());
                }
            }
        }
    }
}

#[test]
fn structured_fuzz_preserves_payload_and_never_panics() {
    let payloads: [&[u8]; 8] = [
        b">status",
        b"!4903.50N/07201.75W-ok",
        b"T#001,111,222,033,044,055,10101010",
        b"\xff\xfe\xfd",
        b"",
        b"~opaque",
        b"$GPGGA,1",
        b"}SRC>APRS:>nested",
    ];

    for payload in payloads {
        let mut packet = b"N0CALL>APRS:".to_vec();
        packet.extend_from_slice(payload);
        let parsed = parse_packet(&packet);
        if payload.is_empty() {
            assert!(parsed.is_err());
        } else {
            let parsed = parsed.expect("structured packet should parse");
            assert_eq!(parsed.payload(), payload);
        }
    }
}

#[test]
fn aprs101_fixture_semantics_match_expected_families() {
    let expectations = [
        ("APRS101_STATUS", "status"),
        ("APRS101_POSITION_NO_MSG", "position"),
        ("APRS101_POSITION_MSG", "position"),
        ("APRS101_POSITION_TIMESTAMP_UTC", "timestamped_position"),
        ("APRS101_POSITION_TIMESTAMP_LOCAL", "timestamped_position"),
        ("APRS101_COMPRESSED_POSITION", "compressed_position"),
        ("APRS101_MESSAGE", "message"),
        ("APRS101_MESSAGE_ACK", "message"),
        ("APRS101_MESSAGE_REJECT", "message"),
        ("APRS101_BULLETIN", "message"),
        ("APRS101_ANNOUNCEMENT", "message"),
        ("APRS101_OBJECT_LIVE", "object"),
        ("APRS101_OBJECT_KILLED", "object"),
        ("APRS101_ITEM_LIVE", "item"),
        ("APRS101_ITEM_KILLED", "item"),
        ("APRS101_WEATHER", "weather"),
        ("APRS101_TELEMETRY_VALUES", "telemetry"),
        ("APRS101_TELEMETRY_PARAMETER_NAMES", "telemetry_metadata"),
        ("APRS101_TELEMETRY_UNITS", "telemetry_metadata"),
        ("APRS101_TELEMETRY_EQUATIONS", "telemetry_metadata"),
        ("APRS101_TELEMETRY_BITS", "telemetry_metadata"),
        ("APRS101_QUERY", "query"),
        ("APRS101_CAPABILITY", "capability"),
        ("APRS101_NMEA_GPGLL", "nmea"),
        ("APRS101_MICE_CURRENT", "mic_e"),
        ("APRS101_MICE_OLD", "mic_e"),
        ("APRS101_MAIDENHEAD", "maidenhead"),
        ("APRS101_USER_DEFINED", "user_defined"),
        ("APRS101_THIRD_PARTY", "third_party"),
        ("APRS101_UNSUPPORTED_IDENTIFIER", "unsupported"),
    ];

    for (id, semantic) in expectations {
        let fixture = aprs101_fixture(id);
        let parsed = parse_packet(fixture.packet).expect("fixture should parse");
        assert_eq!(
            parsed.aprs_data().kind_name(),
            semantic,
            "{id} semantic family changed"
        );
    }
}

#[test]
fn aprs101_message_subtypes_are_classified() {
    let cases = [
        ("APRS101_MESSAGE", MessageKind::Message),
        ("APRS101_MESSAGE_ACK", MessageKind::Ack),
        ("APRS101_MESSAGE_REJECT", MessageKind::Reject),
        ("APRS101_BULLETIN", MessageKind::Bulletin),
        ("APRS101_ANNOUNCEMENT", MessageKind::Announcement),
    ];

    for (id, kind) in cases {
        let fixture = aprs101_fixture(id);
        let parsed = parse_packet(fixture.packet).expect("fixture should parse");
        let AprsData::Message(message) = parsed.aprs_data() else {
            panic!("{id} should be a message variant");
        };
        assert_eq!(message.kind, kind, "{id} message subtype changed");
    }
}

#[test]
fn aprs101_telemetry_metadata_subtypes_are_classified() {
    let cases = [
        (
            "APRS101_TELEMETRY_PARAMETER_NAMES",
            TelemetryMetadataKind::ParameterNames,
        ),
        ("APRS101_TELEMETRY_UNITS", TelemetryMetadataKind::Units),
        (
            "APRS101_TELEMETRY_EQUATIONS",
            TelemetryMetadataKind::Equations,
        ),
        ("APRS101_TELEMETRY_BITS", TelemetryMetadataKind::BitSense),
    ];

    for (id, kind) in cases {
        let fixture = aprs101_fixture(id);
        let parsed = parse_packet(fixture.packet).expect("fixture should parse");
        let AprsData::TelemetryMetadata(metadata) = parsed.aprs_data() else {
            panic!("{id} should be telemetry metadata");
        };
        assert_eq!(metadata.kind, kind, "{id} metadata subtype changed");
        assert!(!metadata.fields().is_empty());
    }
}

#[test]
fn strict_policy_rejects_invalid_nmea_checksum_when_enabled() {
    let mut policy = Policy::strict();
    policy.reject_invalid_nmea_checksum = true;
    let mut engine = Engine::new(policy);

    match engine.process(b"N0CALL>APRS:$GPGLL,4916.45,N,12311.12,W,225444,A,*00") {
        EngineResult::Rejected { reason, packet } => {
            assert_eq!(reason, PolicyRejection::InvalidNmeaChecksum);
            assert_eq!(
                packet.raw().as_bytes(),
                b"N0CALL>APRS:$GPGLL,4916.45,N,12311.12,W,225444,A,*00"
            );
        }
        other => panic!("expected invalid checksum rejection, got {other:?}"),
    }
}

#[test]
fn fuzz_seed_corpora_do_not_panic_and_preserve_accepted_raw_bytes() {
    let seeds: &[&[u8]] = &[
        include_bytes!("../../../fuzz/corpus/parse_packet/status"),
        include_bytes!("../../../fuzz/corpus/parse_packet/message"),
        include_bytes!("../../../fuzz/corpus/parse_packet/telemetry"),
        include_bytes!("../../../fuzz/corpus/parse_packet/weather"),
        include_bytes!("../../../fuzz/corpus/parse_packet/third_party"),
    ];

    for seed in seeds {
        if let Ok(parsed) = parse_packet(seed) {
            assert_eq!(parsed.raw().as_bytes(), *seed);
            let _ = parsed.aprs_data();
            let _ = parsed.summary();
        }
    }
}

#[derive(Debug)]
struct Fixture<'a> {
    id: &'a str,
    packet: &'a [u8],
}

#[derive(Debug)]
struct MalformedSemanticFixture<'a> {
    id: &'a str,
    packet: &'a [u8],
    identifier: u8,
}

fn aprs101_valid_fixtures() -> Vec<Fixture<'static>> {
    include_bytes!("fixtures/aprs101_valid.aprs")
        .split(|byte| *byte == b'\n')
        .filter_map(|line| {
            let line = line.strip_suffix(b"\r").unwrap_or(line);
            if line.is_empty() || line.starts_with(b"#") {
                return None;
            }

            let separator = line
                .iter()
                .position(|byte| *byte == b'\t')
                .expect("APRS101 fixtures use '<id>\\t<packet>' records");
            let (id, packet_with_separator) = line.split_at(separator);
            let packet = &packet_with_separator[1..];
            let id = std::str::from_utf8(id).expect("fixture IDs are ASCII");
            Some(Fixture { id, packet })
        })
        .collect()
}

fn aprs101_fixture(id: &str) -> Fixture<'static> {
    aprs101_valid_fixtures()
        .into_iter()
        .find(|fixture| fixture.id == id)
        .unwrap_or_else(|| panic!("missing APRS101 fixture {id}"))
}

fn aprs101_malformed_semantic_fixtures() -> Vec<MalformedSemanticFixture<'static>> {
    include_bytes!("fixtures/aprs101_malformed_semantics.aprs")
        .split(|byte| *byte == b'\n')
        .filter_map(|line| {
            let line = line.strip_suffix(b"\r").unwrap_or(line);
            if line.is_empty() || line.starts_with(b"#") {
                return None;
            }

            let mut fields = line.split(|byte| *byte == b'\t');
            let id = fields.next().expect("malformed semantic fixture has an ID");
            let packet = fields
                .next()
                .expect("malformed semantic fixture has packet bytes");
            let identifier = fields
                .next()
                .expect("malformed semantic fixture has an expected identifier");
            assert!(
                fields.next().is_none(),
                "malformed semantic fixture has too many fields"
            );
            let id = std::str::from_utf8(id).expect("fixture IDs are ASCII");
            let [identifier] = identifier else {
                panic!("fixture {id} expected identifier must be one byte");
            };
            Some(MalformedSemanticFixture {
                id,
                packet,
                identifier: *identifier,
            })
        })
        .collect()
}

fn aprs101_source_reference_ids() -> BTreeSet<&'static str> {
    include_str!("fixtures/aprs101_sources.tsv")
        .lines()
        .filter_map(|line| {
            if line.is_empty() || line.starts_with('#') {
                return None;
            }

            let (id, _reference) = line
                .split_once('\t')
                .expect("APRS101 source references use '<id>\\t<reference>' records");
            Some(id)
        })
        .collect()
}
