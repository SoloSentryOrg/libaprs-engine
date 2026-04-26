#![no_main]

use libaprs_engine::AprsData;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let mut packet = b"N0CALL>APRS:}".to_vec();
    packet.extend_from_slice(&data[..data.len().min(512)]);

    if let Ok(parsed) = libaprs_engine::parse_packet(&packet) {
        assert_eq!(parsed.raw().as_bytes(), packet.as_slice());
        if let AprsData::ThirdParty(third_party) = parsed.aprs_data() {
            if let Ok(nested) = third_party.nested_packet() {
                assert_eq!(nested.raw().as_bytes(), third_party.body);
                let _ = nested.aprs_data();
            }
        }
    }
});
