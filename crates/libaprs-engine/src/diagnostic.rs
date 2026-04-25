use crate::ParsedPacket;

pub(crate) fn packet_to_json(packet: &ParsedPacket) -> String {
    format!(
        "{{\"raw\":\"{}\",\"source\":\"{}\",\"destination\":\"{}\",\"path\":\"{}\",\"payload\":\"{}\",\"data_type\":\"{}\",\"semantic\":\"{}\"}}",
        escape_json_bytes(packet.raw().as_bytes()),
        escape_json_bytes(packet.source()),
        escape_json_bytes(packet.destination()),
        escape_json_bytes(packet.path()),
        escape_json_bytes(packet.payload()),
        packet.data_type_identifier().name(),
        packet.aprs_data().kind_name(),
    )
}

fn escape_json_bytes(bytes: &[u8]) -> String {
    let mut escaped = String::new();
    for byte in bytes {
        match byte {
            b'"' => escaped.push_str("\\\""),
            b'\\' => escaped.push_str("\\\\"),
            b'\n' => escaped.push_str("\\n"),
            b'\r' => escaped.push_str("\\r"),
            b'\t' => escaped.push_str("\\t"),
            0x20..=0x7e => escaped.push(char::from(*byte)),
            _ => {
                escaped.push_str("\\u00");
                escaped.push(hex_digit(byte >> 4));
                escaped.push(hex_digit(byte & 0x0f));
            }
        }
    }
    escaped
}

fn hex_digit(value: u8) -> char {
    match value {
        0..=9 => char::from(b'0' + value),
        10..=15 => char::from(b'a' + value - 10),
        _ => '0',
    }
}
