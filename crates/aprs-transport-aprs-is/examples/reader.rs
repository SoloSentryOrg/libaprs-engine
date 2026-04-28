use aprs_transport_aprs_is::{read_packet_lines_from_reader, AprsIsLogin};
use libaprs_engine::{parse_packet, AprsData};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let login = AprsIsLogin {
        callsign: "N0CALL",
        passcode: -1,
        software: "libaprs-engine 1.0.0-rc.1",
        filter: Some("r/49/-72/50"),
    };
    assert!(login.line()?.ends_with("\r\n"));

    let input = std::io::Cursor::new(b"# aprs-is banner\r\nN0CALL>APRS:>aprs-is\n");
    for packet_bytes in read_packet_lines_from_reader(input)? {
        let packet = parse_packet(&packet_bytes).map_err(|error| error.code())?;
        assert!(matches!(packet.aprs_data(), AprsData::Status { .. }));
    }

    Ok(())
}
