use aprs_transport_kiss::{decode_frames, encode_data_frame};
use libaprs_engine::parse_packet;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let packet = b"N0CALL>APRS:>kiss profile";
    let tcp_or_serial_bytes = encode_data_frame(0, packet).map_err(|error| error.code())?;

    for frame in decode_frames(&tcp_or_serial_bytes).map_err(|error| error.code())? {
        if frame.command != 0 {
            continue;
        }

        let parsed = parse_packet(&frame.payload)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error.code()))?;
        assert_eq!(parsed.raw().as_bytes(), packet);
    }

    Ok(())
}
