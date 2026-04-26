#![no_main]

use libaprs_engine::AprsData;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let mut packet = b"N0CALL>APRS::TARGET   :".to_vec();
    packet.extend_from_slice(&data[..data.len().min(512)]);

    if let Ok(parsed) = libaprs_engine::parse_packet(&packet) {
        assert_eq!(parsed.raw().as_bytes(), packet.as_slice());
        if let AprsData::Message(message) = parsed.aprs_data() {
            let _ = message.addressee;
            let _ = message.kind;
            let _ = message.text;
            let _ = message.id;
        }
    }
});
