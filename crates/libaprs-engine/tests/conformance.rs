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
