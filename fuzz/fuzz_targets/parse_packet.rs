#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(packet) = libaprs_engine::parse_packet(data) {
        assert_eq!(packet.raw().as_bytes(), data);
        let _ = packet.aprs_data();
        let _ = packet.summary();
    }
});
