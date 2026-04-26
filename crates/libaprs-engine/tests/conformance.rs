use std::collections::BTreeSet;

use libaprs_engine::{parse_packet, AprsData};

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

    for fixture in aprs101_valid_fixtures() {
        assert!(
            referenced_ids.contains(fixture.id),
            "APRS101 fixture {} is missing a source reference",
            fixture.id
        );
    }
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

#[derive(Debug)]
struct Fixture<'a> {
    id: &'a str,
    packet: &'a [u8],
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
