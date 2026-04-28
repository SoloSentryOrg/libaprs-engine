use aprs_transport_kiss::{decode_frames, encode_data_frame};
use libaprs_engine::{parse_packet, AprsData};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let encoded = encode_data_frame(0, b"N0CALL>APRS:>kiss").map_err(|error| error.code())?;
    let frames = decode_frames(&encoded).map_err(|error| error.code())?;

    for frame in frames {
        if frame.command == 0 {
            let packet = parse_packet(&frame.payload).map_err(|error| error.code())?;
            assert!(matches!(packet.aprs_data(), AprsData::Status { .. }));
        }
    }

    Ok(())
}
