# API Guide

This guide shows how to use `libaprs-engine` from another Rust project.

## Dependency

Use the package name `libaprs-engine` in `Cargo.toml` and the crate name
`libaprs_engine` in Rust code.

```toml
[dependencies]
libaprs-engine = { git = "https://github.com/elodiejmirza/libaprs-engine", package = "libaprs-engine" }
```

For a local checkout:

```toml
[dependencies]
libaprs-engine = { path = "../libaprs-engine/crates/libaprs-engine" }
```

## Parser Boundary

The primary codec API is:

```rust
pub fn parse_packet(input: &[u8]) -> Result<ParsedPacket, ParseError>
```

Use byte slices. Do not convert packet input to `String` before parsing.

```rust
use libaprs_engine::parse_packet;

fn main() -> Result<(), libaprs_engine::ParseError> {
    let packet = parse_packet(b"N0CALL>APRS:>status text")?;

    assert_eq!(packet.raw().as_bytes(), b"N0CALL>APRS:>status text");
    assert_eq!(packet.source(), b"N0CALL");
    assert_eq!(packet.destination(), b"APRS");
    assert_eq!(packet.payload(), b">status text");
    assert_eq!(packet.information(), b"status text");

    Ok(())
}
```

## Parse Errors

`ParseError` is fail-closed. Malformed input never returns a partial packet.

Common error families include:

- empty input
- oversized packet
- missing `>` or `:` separators
- empty address or payload segments
- invalid source, destination, or path components
- non-AX.25-like source/path metadata

## Raw Packet Preservation

`ParsedPacket` owns a `RawPacket`. Field accessors return byte slices into the
preserved original bytes.

```rust
fn main() -> Result<(), libaprs_engine::ParseError> {
    let bytes = b"N0CALL>APRS,WIDE1-1:>\xff";
    let packet = libaprs_engine::parse_packet(bytes)?;

    assert_eq!(packet.raw().as_bytes(), bytes);
    assert_eq!(packet.information(), b"\xff");

    Ok(())
}
```

## Semantic Views

Use `ParsedPacket::aprs_data()` for APRS information-field semantics. Semantic
views preserve byte slices and only decode typed values when safe.

```rust
use libaprs_engine::{parse_packet, AprsData};

fn main() -> Result<(), libaprs_engine::ParseError> {
    let packet = parse_packet(b"N0CALL>APRS:=4903.50N/07201.75W-comment")?;

    match packet.aprs_data() {
        AprsData::Position(position) => {
            if let Some(coordinates) = position.coordinates() {
                println!("{}, {}", coordinates.latitude, coordinates.longitude);
            }
            println!("comment={}", String::from_utf8_lossy(position.comment));
        }
        other => println!("semantic={}", other.kind_name()),
    }

    Ok(())
}
```

Current semantic families include:

- `Status`
- `Position`
- `TimestampedPosition`
- `CompressedPosition`
- `Message`
- `Object`
- `Item`
- `Weather`
- `Telemetry`
- `Query`
- `Capability`
- `Nmea`
- `MicE`
- `Maidenhead`
- `UserDefined`
- `ThirdParty`
- `Malformed`
- `Unsupported`

## Engine And Policy

`Engine` combines codec parsing, semantic classification, policy decisions, and
counters.

```rust
use libaprs_engine::{Engine, EngineResult, Policy, PolicyRejection};

let mut engine = Engine::new(Policy::strict());

match engine.process(b"N0CALL>APRS:>ok") {
    EngineResult::Accepted { packet } => {
        println!("accepted {}", packet.aprs_data().kind_name());
    }
    EngineResult::Rejected { reason, .. } => {
        eprintln!("rejected by policy: {reason:?}");
    }
    EngineResult::ParseError(error) => {
        eprintln!("malformed packet: {error:?}");
    }
}

let counters = engine.counters();
println!(
    "accepted={} rejected={} malformed={}",
    counters.accepted, counters.rejected, counters.malformed
);
```

Strict policy rejects unsupported semantics, malformed semantics, and excessive
path component counts. Permissive policy allows unsupported and malformed
semantic variants while keeping codec validation fail-closed.

```rust
let strict = libaprs_engine::Policy::strict();
let permissive = libaprs_engine::Policy::permissive();

assert!(!strict.allow_unsupported);
assert!(permissive.allow_unsupported);
```

## Line Transport

Use `LineTransport` for newline-separated files, stdin, or stream buffers. It
splits on LF, strips a trailing CR, ignores empty lines, and returns packet byte
slices.

```rust
use libaprs_engine::{LineTransport, parse_packet};

fn main() -> Result<(), libaprs_engine::ParseError> {
    let input = b"N0CALL>APRS:>one\r\nN0CALL>APRS:>two\n";

    for packet_bytes in LineTransport::new(input).packets() {
        let packet = parse_packet(packet_bytes)?;
        println!("{}", packet.aprs_data().kind_name());
    }

    Ok(())
}
```

## JSON Diagnostics

`ParsedPacket::to_json()` returns compact diagnostic JSON. It is intended for
inspection and CLI output, not as a stable serialization contract.

```rust
fn main() -> Result<(), libaprs_engine::ParseError> {
    let packet = libaprs_engine::parse_packet(b"N0CALL>APRS:>hello")?;
    println!("{}", packet.to_json());

    Ok(())
}
```
