use libaprs_engine::{
    encoder::{
        encode_status, encode_telemetry, encode_uncompressed_position, UncompressedPositionEncoding,
    },
    parse_packet,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = [b"APRS".as_slice(), b"WIDE1-1".as_slice()];

    let status = encode_status(b"N0CALL", &path, b"encoder example")?;
    assert_eq!(
        parse_packet(&status)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error.code()))?
            .raw()
            .as_bytes(),
        b"N0CALL>APRS,WIDE1-1:>encoder example"
    );

    let position = encode_uncompressed_position(
        b"N0CALL",
        &path,
        UncompressedPositionEncoding {
            messaging: false,
            latitude: b"4903.50N",
            symbol_table: b'/',
            longitude: b"07201.75W",
            symbol_code: b'-',
            comment: b"encoded",
        },
    )?;
    assert_eq!(
        parse_packet(&position)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error.code()))?
            .summary()
            .semantic,
        "position"
    );

    let telemetry = encode_telemetry(b"N0CALL", &path, 1, [111, 222, 33, 44, 55], None)?;
    assert_eq!(
        parse_packet(&telemetry)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error.code()))?
            .summary()
            .semantic,
        "telemetry"
    );

    Ok(())
}
