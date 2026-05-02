# Examples

![libaprs-engine documentation header](assets/brand/docs-header.svg)

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
                let summary = packet.summary();
                println!("accepted semantic={}", summary.semantic);
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

## Extract Object And Item Coordinates

```rust
use libaprs_engine::{parse_packet, AprsData};

fn main() -> Result<(), libaprs_engine::ParseError> {
    let packet = parse_packet(b"N0CALL>APRS:;LEADER   *092345z4903.50N/07201.75W-object")?;

    if let AprsData::Object(object) = packet.aprs_data() {
        if let Some(coordinates) = object.coordinates() {
            println!("lat={}", coordinates.latitude);
            println!("lon={}", coordinates.longitude);
        }
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

## Encode Packets

```rust
use libaprs_engine::{
    encoder::{
        encode_ack, encode_status, encode_telemetry, encode_telemetry_metadata,
        encode_uncompressed_position, TelemetryMetadataEncodingKind, UncompressedPositionEncoding,
    },
    parse_packet,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = [b"APRS".as_slice(), b"WIDE1-1".as_slice()];
    let status = encode_status(b"N0CALL", &path, b"hello")?;
    assert_eq!(parse_packet(&status)?.summary().semantic, "status");

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
    assert_eq!(parse_packet(&position)?.summary().semantic, "position");

    let telemetry = encode_telemetry(b"N0CALL", &path, 1, [111, 222, 33, 44, 55], None)?;
    assert_eq!(parse_packet(&telemetry)?.summary().semantic, "telemetry");

    let ack = encode_ack(b"N0CALL", &path, b"TARGET   ", b"42")?;
    assert_eq!(parse_packet(&ack)?.summary().semantic, "message");

    let metadata = encode_telemetry_metadata(
        b"N0CALL",
        &path,
        TelemetryMetadataEncodingKind::Parameters,
        b"Vbat,Temp",
    )?;
    assert_eq!(parse_packet(&metadata)?.summary().semantic, "telemetry_metadata");

    Ok(())
}
```

## Inspect NMEA And Third-Party Data

```rust
use libaprs_engine::{parse_packet, AprsData, MicEMessageCode};

fn main() -> Result<(), libaprs_engine::ParseError> {
    let nmea = parse_packet(b"N0CALL>APRS:$GPGLL,4916.45,N,12311.12,W,225444,A,*1D")?;
    if let AprsData::Nmea(sentence) = nmea.aprs_data() {
        println!("talker={:?}", sentence.talker_id());
        println!("sentence={:?}", sentence.sentence_id());
        println!("fields={:?}", sentence.data_fields());
        println!("checksum={:?}", sentence.checksum());
    }

    let mic_e = parse_packet(b"N0CALL>ABC123:`abcde")?;
    if let AprsData::MicE(mic_e) = mic_e.aprs_data() {
        if let Some(MicEMessageCode::Standard(message)) = mic_e.message_code() {
            println!("mic-e={message:?}");
        }
    }

    let third_party = parse_packet(b"N0CALL>APRS:}SRC>APRS:>nested")?;
    if let AprsData::ThirdParty(wrapper) = third_party.aprs_data() {
        let nested = wrapper.nested_packet()?;
        println!("nested={}", nested.aprs_data().kind_name());
    }

    Ok(())
}
```

## Transport Cookbook Examples

Compile-tested transport examples live with their crates:

- `crates/libaprs-engine/examples/encode_packets.rs`
- `crates/libaprs-engine/examples/service_ingest.rs`
- `crates/libaprs-engine/examples/service_toolkit.rs`
- `crates/aprs-transport-aprs-is/examples/profile.rs`
- `crates/aprs-transport-aprs-is/examples/reader.rs`
- `crates/aprs-transport-kiss/examples/frame_pipeline.rs`
- `crates/aprs-transport-kiss/examples/tcp_serial_profile.rs`
- `crates/aprs-transport-udp/examples/datagram_ingest.rs`
- `crates/aprs-transport-corpus/examples/replay.rs`

## Service Ingestion Pattern

Use `Engine` as the service boundary after transport-specific framing and
bounded reads. Keep policy strict, log stable diagnostic codes, and leave
timeouts, reconnects, and worker queues application-owned.

```rust
use libaprs_engine::{Engine, EngineResult, LineTransport, Policy, MAX_PACKET_LEN};

fn main() -> Result<(), std::io::Error> {
    let input = b"N0CALL>APRS:>service online\nN1CALL>APRS:~opaque\n";
    let mut engine = Engine::new(Policy::strict());

    for packet in LineTransport::new(input).packets_with_limit(MAX_PACKET_LEN)? {
        match engine.process(packet) {
            EngineResult::Accepted { packet } => {
                println!("accepted semantic={}", packet.summary().semantic);
            }
            EngineResult::Rejected { reason, .. } => {
                println!("rejected code={}", reason.diagnostic().code);
            }
            EngineResult::ParseError(error) => {
                println!("malformed code={}", error.diagnostic().code);
            }
        }
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
    assert_eq!(packet.summary().semantic, "status");

    Ok(())
}
```
