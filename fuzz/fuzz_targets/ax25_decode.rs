#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = aprs_transport_ax25::decode_ax25_ui_frame(data);
});
