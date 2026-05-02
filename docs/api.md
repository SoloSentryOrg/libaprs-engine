# API Guide

![libaprs-engine documentation header](assets/brand/docs-header.svg)

This guide shows how to use `libaprs-engine` from another Rust project.

## Dependency

Use the package name `libaprs-engine` in `Cargo.toml` and the crate name
`libaprs_engine` in Rust code.

```toml
[dependencies]
libaprs-engine = "2.5.0"
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
- `Position::weather()`, `TimestampedPosition::weather()`, and
  `CompressedPosition::weather()` when the symbol code is `_` and the position
  comment carries weather bytes
- `Object::coordinates()` and `Item::coordinates()` when their bodies start
  with supported APRS position encodings
- `Object::weather()` and `Item::weather()` when their bodies start with an
  uncompressed or compressed weather-symbol position
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

Known unsupported semantic edge cases in the current `2.x` line include:

- Mic-E altitude, ambiguity, and telemetry extension decoding
- capability body field splitting
- semantic validation of third-party nested packet policy beyond the nested
  codec envelope

## Packet Encoding

Use `libaprs_engine::encoder` when an application needs to construct APRS packet
bytes before handing them to caller-owned transport code. Encoder helpers return
owned `Vec<u8>` values and validate the same conservative address envelope used
by the parser. They do not transmit, log, lowercase, trim, or otherwise
normalize packet bytes.

```rust
use libaprs_engine::{
    encoder::{
        encode_ack, encode_status, encode_telemetry_metadata,
        encode_uncompressed_position, TelemetryMetadataEncodingKind,
        UncompressedPositionEncoding,
    },
    parse_packet,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = [b"APRS".as_slice(), b"WIDE1-1".as_slice()];

    let status = encode_status(b"N0CALL", &path, b"hello")?;
    assert_eq!(status, b"N0CALL>APRS,WIDE1-1:>hello");
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

Current encoder helpers cover generic payloads, status, uncompressed position,
message, acknowledgement, rejection, bulletin, announcement, telemetry,
telemetry metadata, object, and item packets. Use `aprs-transport-kiss` for KISS
frame encoding because that framing boundary lives in the transport crate.

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

## Service Toolkit

Use `libaprs_engine::service` for small runtime-neutral helpers in long-running
ingestion services. These helpers do not own clocks, storage, threads, sockets,
or queues.

```rust
use libaprs_engine::{
    parse_packet,
    service::{
        DuplicateDecision, DuplicateWindow, PacketRateBudget, RateLimitDecision,
        SemanticBlocklist, SemanticFamily,
    },
};

fn main() -> Result<(), libaprs_engine::ParseError> {
    let packet_bytes = b"N0CALL>APRS:>service";
    let mut duplicates = DuplicateWindow::new(128);
    let mut budget = PacketRateBudget::new(100);
    let blocked = SemanticBlocklist::new(&[SemanticFamily::Unsupported]);

    if budget.try_consume() == RateLimitDecision::Allowed
        && duplicates.observe(packet_bytes) == DuplicateDecision::New
    {
        let packet = parse_packet(packet_bytes)?;
        assert!(!blocked.rejects(&packet.aprs_data()));
    }

    Ok(())
}
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

The library no longer exposes `ParsedPacket::to_json()` starting in
`v2.0.0-rc.1`.
Rust integrations should use structured diagnostics, event structs, or an
application-owned schema. The CLI still provides `aprs-cli --json` as
diagnostic output for operators.

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

`Engine::process_event()` returns stable event structs for long-running
ingestion services that need packet, policy, malformed, and counter telemetry
without scraping text output.

```rust
use libaprs_engine::{Engine, EngineEvent};

fn main() {
    let mut engine = Engine::default();

    match engine.process_event(b"N0CALL>APRS:>hello") {
        EngineEvent::Accepted(event) => {
            assert_eq!(event.kind().code(), "accepted");
            assert_eq!(event.packet.summary().semantic, "status");
        }
        EngineEvent::Rejected(event) => {
            eprintln!("rejected code={}", event.diagnostic.code);
        }
        EngineEvent::Malformed(event) => {
            eprintln!(
                "malformed code={} raw_len={} raw_truncated={}",
                event.diagnostic.code,
                event.raw.len(),
                event.raw_truncated
            );
        }
    }
}
```

With the optional `metrics` feature, use the dependency-free metrics helpers to
bridge `Counters` into your own telemetry stack.

```rust
use libaprs_engine::{metrics_support::counter_metrics, Counters};

fn main() {
    let metrics = counter_metrics(Counters {
        accepted: 1,
        rejected: 2,
        malformed: 3,
    });

    assert_eq!(metrics[0].name, "libaprs_engine_packets_accepted_total");
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
  version = "2.5.0",
  features = ["serde"]
}
```

Prefer `ParsedPacket::to_diagnostic()` for a named diagnostic API. It returns
the same owned structure as `PacketDiagnostic::from_packet(&packet)` while
keeping diagnostic JSON separate from application-owned schemas.

```rust
use libaprs_engine::parse_packet;

fn main() -> Result<(), libaprs_engine::ParseError> {
    let packet = parse_packet(b"N0CALL>APRS:>\xff")?;
    let diagnostic = packet.to_diagnostic();
    assert_eq!(diagnostic.schema_version, 1);
    assert_eq!(diagnostic.raw, b"N0CALL>APRS:>\xff");
    assert_eq!(diagnostic.semantic, "status");
    Ok(())
}
```

## APRS-IS Transport Adapter

Use `aprs-transport-aprs-is` for APRS-IS login framing, profile validation,
q-construct diagnostics, and reader-backed packet splitting. APRS-IS server
comment lines beginning with `#` are ignored, packet bytes are preserved
exactly, and reader-backed input is capped by default to avoid unbounded memory
growth. Network connection management remains application-owned.

```toml
[dependencies]
aprs-transport-aprs-is = "2.5.0"
libaprs-engine = "2.5.0"
```

```rust
use aprs_transport_aprs_is::{
    q_construct_from_tnc2, read_packet_lines_from_reader, AprsIsFilter, AprsIsLogin,
};
use libaprs_engine::parse_packet;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let filter = AprsIsFilter::new("r/49/-72/50")?;
    let login = AprsIsLogin {
        callsign: "N0CALL",
        passcode: -1,
        software: "libaprs-engine 2.5.0",
        filter: Some(filter.as_str()),
    };
    assert!(login.profile_line()?.ends_with("\r\n"));

    let input = std::io::Cursor::new(b"# banner\r\nN0CALL>APRS,TCPIP*,qAC,T2SERVER:>hello\n");
    for line in read_packet_lines_from_reader(input)? {
        if let Some(q) = q_construct_from_tnc2(&line) {
            println!("q_construct={}", q.kind.code());
        }

        // APRS-IS q constructs are transport metadata. Parse a packet only
        // after the application has chosen how to handle that metadata.
        let packet = parse_packet(b"N0CALL>APRS:>hello").map_err(|error| error.code())?;
        println!("semantic={}", packet.aprs_data().kind_name());
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
aprs-transport-file = "2.5.0"
libaprs-engine = "2.5.0"
```

## TCP Transport Adapter

Use `aprs-transport-tcp` when packet bytes come from a blocking TCP stream or
another `Read` implementation. This keeps network I/O outside the parser core.

```toml
[dependencies]
aprs-transport-tcp = "2.5.0"
libaprs-engine = "2.5.0"
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
