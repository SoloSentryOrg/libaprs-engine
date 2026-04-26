# Examples

These examples are intentionally small and copyable.

## Parse One Packet

```rust
use libaprs_engine::{parse_packet, AprsData};

fn main() -> Result<(), libaprs_engine::ParseError> {
    let packet = parse_packet(b"N0CALL>APRS:>hello")?;

    println!("source={}", String::from_utf8_lossy(packet.source()));
    println!("destination={}", String::from_utf8_lossy(packet.destination()));

    if let AprsData::Status(status) = packet.aprs_data() {
        println!("status={}", String::from_utf8_lossy(status.text));
    }

    Ok(())
}
```

## Process A Packet File

```rust
use std::fs::File;

use libaprs_engine::{
    read_all_with_limit, Engine, EngineResult, LineTransport, Policy,
    DEFAULT_TRANSPORT_READ_LIMIT,
};

fn main() -> std::io::Result<()> {
    let input = read_all_with_limit(File::open("packets.aprs")?, DEFAULT_TRANSPORT_READ_LIMIT)?;
    let mut engine = Engine::new(Policy::strict());

    for packet_bytes in LineTransport::new(&input).packets() {
        match engine.process(packet_bytes) {
            EngineResult::Accepted { packet } => {
                println!("{}", packet.to_json());
            }
            EngineResult::Rejected { reason, .. } => {
                eprintln!("rejected: {reason:?}");
            }
            EngineResult::ParseError(error) => {
                eprintln!("malformed: {error:?}");
            }
        }
    }

    Ok(())
}
```

## Use The File Transport Crate

```rust
use aprs_transport_file::read_packet_lines_from_path_with_limit;
use libaprs_engine::parse_packet;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    for packet_bytes in read_packet_lines_from_path_with_limit("packets.aprs", 256 * 1024)? {
        let packet = parse_packet(&packet_bytes).map_err(|error| error.code())?;
        println!("{}", packet.aprs_data().kind_name());
    }

    Ok(())
}
```

## Use The Shared Transport Contract

```rust
use libaprs_engine::{LineTransport, PacketSink, PacketSource};

fn main() -> std::io::Result<()> {
    let mut source = LineTransport::new(b"N0CALL>APRS:>one\nN1CALL>APRS:>two\n");
    let mut packets = Vec::new();

    for packet in source.recv_packets()? {
        packets.send_packet(&packet)?;
    }

    Ok(())
}
```

## Extract Position Coordinates

```rust
use libaprs_engine::{parse_packet, AprsData};

fn main() -> Result<(), libaprs_engine::ParseError> {
    let packet = parse_packet(b"N0CALL>APRS:=4903.50N/07201.75W-comment")?;

    match packet.aprs_data() {
        AprsData::Position(position) => {
            if let Some(coordinates) = position.coordinates() {
                println!("lat={}", coordinates.latitude);
                println!("lon={}", coordinates.longitude);
            }
        }
        other => eprintln!("not a position packet: {}", other.kind_name()),
    }

    Ok(())
}
```

## Read Telemetry

```rust
use libaprs_engine::{parse_packet, AprsData};

fn main() -> Result<(), libaprs_engine::ParseError> {
    let packet = parse_packet(b"N0CALL>APRS:T#123,001,002,003,004,005,10101010")?;

    if let AprsData::Telemetry(telemetry) = packet.aprs_data() {
        println!("sequence={:?}", telemetry.sequence_number());
        println!("analog={:?}", telemetry.analog_values());
        println!("bits={:?}", telemetry.digital_bits());
    }

    Ok(())
}
```

## Read Telemetry Metadata

```rust
use libaprs_engine::{parse_packet, AprsData};

fn main() -> Result<(), libaprs_engine::ParseError> {
    let packet = parse_packet(b"N0CALL>APRS::PARM.    :Vbat,Temp,Pressure")?;

    if let AprsData::TelemetryMetadata(metadata) = packet.aprs_data() {
        println!("kind={:?}", metadata.kind);
        println!("fields={:?}", metadata.fields());
    }

    Ok(())
}
```

## Inspect NMEA And Third-Party Data

```rust
use libaprs_engine::{parse_packet, AprsData};

fn main() -> Result<(), libaprs_engine::ParseError> {
    let nmea = parse_packet(b"N0CALL>APRS:$GPGLL,4916.45,N,12311.12,W,225444,A,*1D")?;
    if let AprsData::Nmea(sentence) = nmea.aprs_data() {
        println!("checksum={:?}", sentence.checksum());
    }

    let third_party = parse_packet(b"N0CALL>APRS:}SRC>APRS:>nested")?;
    if let AprsData::ThirdParty(wrapper) = third_party.aprs_data() {
        let nested = wrapper.nested_packet()?;
        println!("nested={}", nested.aprs_data().kind_name());
    }

    Ok(())
}
```

## Handle Invalid UTF-8 Payloads

```rust
use libaprs_engine::parse_packet;

fn main() -> Result<(), libaprs_engine::ParseError> {
    let raw = b"N0CALL>APRS:>\xff";
    let packet = parse_packet(raw)?;

    assert_eq!(packet.raw().as_bytes(), raw);
    println!("{}", packet.to_json());

    Ok(())
}
```
