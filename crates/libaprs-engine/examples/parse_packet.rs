use libaprs_engine::{parse_packet, AprsData};

fn main() -> Result<(), libaprs_engine::ParseError> {
    let packet = parse_packet(b"N0CALL>APRS:>hello")?;

    println!("source={}", String::from_utf8_lossy(packet.source()));
    println!(
        "destination={}",
        String::from_utf8_lossy(packet.destination())
    );

    if let AprsData::Status { text } = packet.aprs_data() {
        println!("status={}", String::from_utf8_lossy(text));
    }

    Ok(())
}
