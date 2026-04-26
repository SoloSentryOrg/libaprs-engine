use aprs_transport_aprs_is::{read_packet_lines as read_aprs_is_packet_lines, AprsIsLogin};
use aprs_transport_file::read_packet_lines;
use aprs_transport_tcp::read_packet_lines_from_reader;
use libaprs_engine::{parse_packet, AprsData};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let packet = parse_packet(b"N0CALL>APRS:>hello").map_err(|error| error.code())?;
    assert!(matches!(packet.aprs_data(), AprsData::Status { .. }));

    let file_packets = read_packet_lines(b"N0CALL>APRS:>file\n");
    assert_eq!(file_packets.len(), 1);

    let tcp_packets = read_packet_lines_from_reader(std::io::Cursor::new(b"N0CALL>APRS:>tcp\n"))?;
    assert_eq!(tcp_packets.len(), 1);

    let aprs_is_login = AprsIsLogin {
        callsign: "N0CALL",
        passcode: -1,
        software: "libaprs-engine-downstream-smoke 0.1.3",
        filter: Some("r/49/-72/50"),
    };
    assert!(aprs_is_login.line()?.ends_with("\r\n"));

    let aprs_is_packets = read_aprs_is_packet_lines(b"# banner\nN0CALL>APRS:>aprs-is\n");
    assert_eq!(aprs_is_packets.len(), 1);

    Ok(())
}
