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
use libaprs_engine::{Engine, EngineResult, LineTransport, Policy};

fn main() -> std::io::Result<()> {
    let input = std::fs::read("packets.aprs")?;
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
