use std::io::Cursor;

use aprs_transport_aprs_is::{
    q_construct_from_tnc2, read_packet_lines_from_reader, AprsIsFilter, AprsIsLogin,
};
use libaprs_engine::parse_packet;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filter = AprsIsFilter::new("r/49/-72/50 t/poimq")?;
    let login = AprsIsLogin {
        callsign: "N0CALL-7",
        passcode: -1,
        software: "libaprs-engine 3.0.0-rc.2",
        filter: Some(filter.as_str()),
    };
    assert!(login.profile_line()?.ends_with("\r\n"));

    let input = Cursor::new(b"# server banner\r\nN0CALL>APRS,TCPIP*,qAC,T2SERVER:>hello\n");
    for packet_bytes in read_packet_lines_from_reader(input)? {
        if let Some(q) = q_construct_from_tnc2(&packet_bytes) {
            assert_eq!(q.kind.code(), "qac");
        }

        let packet = parse_packet(b"N0CALL>APRS:>hello")
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error.code()))?;
        assert_eq!(packet.summary().semantic, "status");
    }

    Ok(())
}
