#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let midpoint = data.len() / 2;
    let topic = String::from_utf8_lossy(&data[..midpoint]);
    let filter = String::from_utf8_lossy(&data[midpoint..]);
    let _ = aprs_transport_mqtt::topic_matches(&filter, &topic);
});
