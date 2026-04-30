# API Guide

This guide shows how to use `libaprs-engine` from another Rust project.

## Dependency

Use the package name `libaprs-engine` in `Cargo.toml` and the crate name
`libaprs_engine` in Rust code.

```toml
[dependencies]
libaprs-engine = { git = "https://github.com/elodiejmirza/libaprs-engine", package = "libaprs-engine", tag = "v1.1.0" }
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

This API is part of the `1.0.0` public boundary. See
[Public API Boundary](public-api.md) for the full stable API list, internal API
boundaries, and semver guidance.

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

Use `parse_packet_with_options` when a consumer needs a stricter packet length
limit than the default `MAX_PACKET_LEN`.

```rust
use libaprs_engine::{ParseOptions, parse_packet_with_options};

fn main() -> Result<(), libaprs_engine::ParseError> {
    let options = ParseOptions::new(80);
    let packet = parse_packet_with_options(b"N0CALL>APRS:>short", options)?;
    println!("{}", packet.aprs_data().kind_name());
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

`ParseError::code()` returns stable strings such as `parse.empty` and
`parse.invalid_address` for external logs and metrics.

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
- `TelemetryMetadata`
- `Query`
- `Capability`
- `Nmea`
- `MicE`
- `Maidenhead`
- `UserDefined`
- `ThirdParty`
- `Malformed`
- `Unsupported`

Semantic helper methods include:

- `Position::coordinates()` and `CompressedPosition::coordinates()`
- `Object::coordinates()` and `Item::coordinates()` when their bodies start
  with supported APRS position encodings
- `Weather::fields()` for wind, temperature, rain, humidity, pressure,
  luminosity, snow, and raw rain counter values
- `Telemetry::sequence_number()`, `Telemetry::analog_values()`, and
  `Telemetry::digital_bits()`
- `TelemetryMetadata::fields()`
- `Nmea::talker_id()`, `Nmea::sentence_id()`, `Nmea::data_fields()`, and
  `Nmea::checksum()`
- `MicE::coordinates()`, `MicE::speed_course()`, and `MicE::message_code()`
- `ThirdParty::nested_packet()`

Semantic malformed handling is explicit and byte-preserving:

- empty `_` weather reports are `AprsData::Malformed`
- `}` third-party bodies must pass the nested `source>path:payload` codec
  envelope before they are exposed as `AprsData::ThirdParty`
- unsupported identifiers remain `AprsData::Unsupported` instead of being
  guessed as another packet family

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

An engine can also process a packet source directly:

```rust
use libaprs_engine::{Engine, LineTransport, Policy};

fn main() -> std::io::Result<()> {
    let mut engine = Engine::new(Policy::permissive());
    let mut source = LineTransport::new(b"N0CALL>APRS:>one\n");
    let results = engine.process_source(&mut source)?;

    assert_eq!(results.len(), 1);
    Ok(())
}
```

Strict policy rejects unsupported semantics, malformed semantics, and excessive
path component counts. It also exposes an opt-in semantic check for rejecting
NMEA sentences when a present checksum does not match. Permissive policy allows
unsupported and malformed semantic variants while keeping codec validation
fail-closed.

```rust
let mut strict = libaprs_engine::Policy::strict();
strict.reject_invalid_nmea_checksum = true;
let permissive = libaprs_engine::Policy::permissive();

assert!(!strict.allow_unsupported);
assert!(strict.reject_invalid_nmea_checksum);
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

    for packet_bytes in LineTransport::new(input)
        .packets_with_limit(libaprs_engine::MAX_PACKET_LEN)?
    {
        let packet = parse_packet(packet_bytes)?;
        println!("{}", packet.aprs_data().kind_name());
    }

    Ok(())
}
```

Use `packets()` only for already bounded trusted byte slices. Use
`packets_with_limit()` for external input; it returns
`transport.oversized_input` before owned packet copies are allocated.

`LineTransport` also implements the shared `PacketSource` trait with the
default APRS packet limit. `Vec<Vec<u8>>` implements `PacketSink` for tests,
examples, and in-process adapters.

```rust
use libaprs_engine::{LineTransport, PacketSink, PacketSource};

fn main() -> std::io::Result<()> {
    let mut source = LineTransport::new(b"N0CALL>APRS:>one\n");
    let mut sink = Vec::new();

    for packet in source.recv_packets()? {
        sink.send_packet(&packet)?;
    }

    Ok(())
}
```

Transport helpers use byte limits rather than unbounded reads. The shared
default is `DEFAULT_TRANSPORT_READ_LIMIT`, and oversized transport input returns
an `InvalidData` I/O error whose message is the stable code
`transport.oversized_input`.

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

## Structured Diagnostics

`ParsedPacket::summary()` returns a stable structured summary for observability.
It includes address bytes, semantic names, and decoded helper values when they
are available.

```rust
fn main() -> Result<(), libaprs_engine::ParseError> {
    let packet = libaprs_engine::parse_packet(
        b"N0CALL>APRS:$GPGLL,4916.45,N,12311.12,W,225444,A,*1D",
    )?;
    let summary = packet.summary();

    assert_eq!(summary.semantic, "nmea");
    assert!(summary.nmea_checksum.expect("checksum").valid);

    Ok(())
}
```

Parser, policy, and transport failures also expose structured diagnostic
metadata. Use the stable `code` field for alerts and machine processing.

```rust
use libaprs_engine::{ParseError, PolicyRejection, TransportErrorCode};

fn main() {
    let parse = ParseError::MissingSeparator.diagnostic();
    assert_eq!(parse.code, "parse.missing_separator");
    assert_eq!(parse.layer.code(), "parse");

    let policy = PolicyRejection::UnsupportedSemantics.diagnostic();
    assert_eq!(policy.code, "policy.unsupported_semantics");

    let transport = TransportErrorCode::OversizedInput.diagnostic();
    assert_eq!(transport.code, "transport.oversized_input");
}
```

`support_matrix()` returns the same capability inventory exposed by the CLI
`support-matrix --json` command. It is intended for deployment checks and
documentation tooling.

```rust
fn main() {
    let matrix = libaprs_engine::support_matrix();
    assert_eq!(matrix.schema_version, 1);
    assert!(matrix
        .semantic_families
        .iter()
        .any(|item| item.kind == "status"));
}
```

## Optional Serde Diagnostics

Enable the `serde` feature to use an owned diagnostic structure that serializes
raw bytes as byte arrays rather than assuming UTF-8.

```toml
[dependencies]
libaprs-engine = {
  git = "https://github.com/elodiejmirza/libaprs-engine",
  package = "libaprs-engine",
  tag = "v1.1.0",
  features = ["serde"]
}
```

## APRS-IS Transport Adapter

Use `aprs-transport-aprs-is` for APRS-IS login framing and reader-backed packet
splitting. APRS-IS server comment lines beginning with `#` are ignored, packet
bytes are preserved exactly, and reader-backed input is capped by default to
avoid unbounded memory growth. Network connection management remains
application-owned.

```toml
[dependencies]
aprs-transport-aprs-is = "1.1.0"
libaprs-engine = "1.1.0"
```

```rust
use aprs_transport_aprs_is::{read_packet_lines_from_reader, AprsIsLogin};
use libaprs_engine::parse_packet;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let login = AprsIsLogin {
        callsign: "N0CALL",
        passcode: -1,
        software: "libaprs-engine 1.1.0",
        filter: Some("r/49/-72/50"),
    };
    assert!(login.line()?.ends_with("\r\n"));

    let input = std::io::Cursor::new(b"# banner\r\nN0CALL>APRS:>hello\n");
    for line in read_packet_lines_from_reader(input)? {
        let packet = parse_packet(&line).map_err(|error| error.code())?;
        println!("{}", packet.aprs_data().kind_name());
    }

    Ok(())
}
```

```rust
use libaprs_engine::{parse_packet, serde_support::PacketDiagnostic};

fn main() -> Result<(), libaprs_engine::ParseError> {
    let packet = parse_packet(b"N0CALL>APRS:>\xff")?;
    let diagnostic = PacketDiagnostic::from_packet(&packet);
    assert_eq!(diagnostic.semantic, "status");
    Ok(())
}
```

## File Transport Adapter

Use `aprs-transport-file` if you want a separate crate to read packet files as
bytes before handing packets to the core engine.

```toml
[dependencies]
aprs-transport-file = { git = "https://github.com/elodiejmirza/libaprs-engine", package = "aprs-transport-file", tag = "v1.1.0" }
libaprs-engine = { git = "https://github.com/elodiejmirza/libaprs-engine", package = "libaprs-engine", tag = "v1.1.0" }
```

## TCP Transport Adapter

Use `aprs-transport-tcp` when packet bytes come from a blocking TCP stream or
another `Read` implementation. This keeps network I/O outside the parser core.

```toml
[dependencies]
aprs-transport-tcp = { git = "https://github.com/elodiejmirza/libaprs-engine", package = "aprs-transport-tcp", tag = "v1.1.0" }
libaprs-engine = { git = "https://github.com/elodiejmirza/libaprs-engine", package = "libaprs-engine", tag = "v1.1.0" }
```

```rust
use aprs_transport_tcp::read_packet_lines_from_reader;
use libaprs_engine::parse_packet;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input = std::io::Cursor::new(b"N0CALL>APRS:>hello\n");

    for line in read_packet_lines_from_reader(input)? {
        let packet = parse_packet(&line).map_err(|error| error.code())?;
        println!("{}", packet.aprs_data().kind_name());
    }

    Ok(())
}
```
