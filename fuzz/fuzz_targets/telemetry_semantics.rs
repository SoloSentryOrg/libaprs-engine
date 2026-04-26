#![no_main]

use libaprs_engine::AprsData;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let mut packet = b"N0CALL>APRS:T#".to_vec();
    packet.extend_from_slice(&data[..data.len().min(512)]);

    if let Ok(parsed) = libaprs_engine::parse_packet(&packet) {
        assert_eq!(parsed.raw().as_bytes(), packet.as_slice());
        if let AprsData::Telemetry(telemetry) = parsed.aprs_data() {
            let _ = telemetry.sequence_number();
            let _ = telemetry.analog_values();
            let _ = telemetry.digital_bits();
        }
    }
});
