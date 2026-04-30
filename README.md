# libaprs-engine

[![Crates.io](https://img.shields.io/crates/v/libaprs-engine.svg)](https://crates.io/crates/libaprs-engine)
[![Docs.rs](https://docs.rs/libaprs-engine/badge.svg)](https://docs.rs/libaprs-engine)
[![Rust CI](https://github.com/elodiejmirza/libaprs-engine/actions/workflows/rust-ci.yml/badge.svg)](https://github.com/elodiejmirza/libaprs-engine/actions/workflows/rust-ci.yml)

Protocol-first APRS parsing and inspection for Rust.

`libaprs-engine` is a byte-preserving APRS engine. It accepts untrusted
packet bytes, keeps the original bytes intact, rejects malformed packet shape at
the codec boundary, and exposes structured APRS views for downstream policy,
telemetry, indexing, and diagnostics.

## Project Status

- APRS engine with meaningful semantics, conformance fixtures,
  compatibility tests, examples, benchmark, optional transport adapters, and CLI
  inspector.
- Current tagged release: `v1.1.0`.
- Public API is semver-protected from `1.0.0`. The public boundary is tracked in
  [Public API Boundary](docs/public-api.md).
- Core runtime remains network-free and async-free. Optional `serde`
  diagnostics and separate transport adapter crates are available.
- GitHub Actions workflow is active and checks Rust `1.80.0` plus stable,
  including formatting, tests, examples, metadata, docs, and clippy.

## Workspace Crates

- `libaprs-engine`: library crate with packet types, parser, semantic views,
  policy, engine orchestration, counters, JSON diagnostics, shared transport
  contracts, bounded-read helpers, and line transport.
- `aprs-cli`: command-line packet inspector built on the library crate.
- `aprs-transport-file`: optional file transport helper crate that reads packet
  files as bytes and returns newline-separated packet byte vectors.
- `aprs-transport-tcp`: optional TCP transport helper crate that reads packet
  bytes from a reader or TCP address outside the parser core.
- `aprs-transport-aprs-is`: optional APRS-IS helper crate for login line
  framing and reader-backed packet splitting.
- `aprs-transport-kiss`: optional KISS frame encode/decode helper crate.
- `aprs-transport-serial`: optional serial-like reader helper crate.
- `aprs-transport-udp`: optional UDP datagram helper crate.
- `aprs-transport-http`: optional HTTP body ingestion helper crate.
- `aprs-transport-file-watch`: optional append-only packet log helper crate.
- `aprs-transport-mqtt`: optional MQTT payload helper crate.
- `aprs-transport-ax25`: optional AX.25 UI frame helper crate.
- `aprs-transport-corpus`: optional corpus replay helper crate.
- `aprs-transport-channel`: optional in-process channel helper crate.
- `aprs-transport-async`: optional runtime-neutral async splitting helper crate.

## Which Crate Should I Use?

- Use `libaprs-engine` for parsing, semantic inspection, policy, counters, and
  diagnostics.
- Use `aprs-cli` for command-line inspection and corpus triage.
- Use `aprs-transport-file` when packet bytes come from files or stdin-style
  buffers.
- Use `aprs-transport-tcp` when packet bytes come from blocking TCP or another
  `Read` implementation.
- Use `aprs-transport-aprs-is` when connecting to APRS-IS and you need login
  line framing plus APRS-IS comment filtering.
- Use `aprs-transport-kiss` for KISS TNC/TCP/serial frame encode/decode.
- Use `aprs-transport-serial`, `aprs-transport-udp`, `aprs-transport-http`,
  `aprs-transport-file-watch`, `aprs-transport-mqtt`,
  `aprs-transport-ax25`, `aprs-transport-corpus`,
  `aprs-transport-channel`, or `aprs-transport-async` when those source
  boundaries match your application.

## Install Or Depend On It

Use crates.io:

```toml
[dependencies]
libaprs-engine = "1.1.0"
aprs-transport-file = "1.1.0"
aprs-transport-tcp = "1.1.0"
aprs-transport-aprs-is = "1.1.0"
aprs-transport-kiss = "1.1.0"
aprs-transport-serial = "1.1.0"
aprs-transport-udp = "1.1.0"
aprs-transport-http = "1.1.0"
aprs-transport-file-watch = "1.1.0"
aprs-transport-mqtt = "1.1.0"
aprs-transport-ax25 = "1.1.0"
aprs-transport-corpus = "1.1.0"
aprs-transport-channel = "1.1.0"
aprs-transport-async = "1.1.0"
```

Use a Git dependency when testing unreleased changes from this repository.

```toml
[dependencies]
libaprs-engine = { git = "https://github.com/elodiejmirza/libaprs-engine", package = "libaprs-engine", tag = "v1.1.0" }
```

For local development from a checkout:

```toml
[dependencies]
libaprs-engine = { path = "../libaprs-engine/crates/libaprs-engine" }
```

Rust imports use underscores:

```rust
use libaprs_engine::parse_packet;
```

## Quick Start

Parse one APRS packet from bytes:

```rust
use libaprs_engine::{parse_packet, AprsData};

fn main() -> Result<(), libaprs_engine::ParseError> {
    let packet = parse_packet(b"N0CALL>APRS:>hello")?;

    assert_eq!(packet.raw().as_bytes(), b"N0CALL>APRS:>hello");
    assert_eq!(packet.source(), b"N0CALL");
    assert_eq!(packet.destination(), b"APRS");

    match packet.aprs_data() {
        AprsData::Status(status) => {
            assert_eq!(status.text, b"hello");
        }
        other => {
            println!("semantic={}", other.kind_name());
        }
    }

    Ok(())
}
```

Run the CLI against newline-separated packets:

```sh
cargo run -p aprs-cli -- --json packets.aprs
cargo run -p aprs-cli -- packets.aprs
cargo run -p aprs-cli -- --filter status packets.aprs
cargo run -p aprs-cli -- --permissive packets.aprs
cat packets.aprs | cargo run -p aprs-cli -- --json
```

## Security Model

- Treat every packet as untrusted bytes.
- Preserve raw input exactly for accepted packets.
- Reject empty, oversized, malformed, or non-AX.25-like packet shape.
- Do not trim, lowercase, normalize, or lossy-convert packet bytes before
  calling the parser.
- Keep payload bytes opaque; they may not be valid UTF-8.
- Use bounded transport reads. The default helper limit is
  `DEFAULT_TRANSPORT_READ_LIMIT`, and oversized transport input reports
  `transport.oversized_input`.
- Return optional typed interpretations when fields cannot be decoded safely.
- Use `Policy` and `Engine` to apply operational rejection rules after codec
  validation.

See [Security Model](docs/security.md) for details.

## Documentation

- [API Guide](docs/api.md): library types, parser, engine, policy, and semantic
  variants.
- [CLI Guide](docs/cli.md): command-line input, output, exit behavior, and
  examples.
- [Examples](docs/examples.md): copyable integration patterns.
- [Operations Guide](docs/operations.md): production deployment patterns,
  diagnostics, logging, limits, and safe defaults.
- [Transport Adapters](docs/transports.md): byte-preserving transport crate
  boundaries and integration examples.
- [Architecture](docs/architecture.md): boundaries, contracts, and pipeline.
- [Security Model](docs/security.md): untrusted input handling and OWASP-aligned
  controls.
- [Public API Boundary](docs/public-api.md): semver-protected public API surface,
  internal boundaries, and semver guidance.
- [Stability](docs/stability.md): API stability levels and feature flags.
- [Conformance Matrix](docs/conformance.md): APRS family support and known gaps.
- [Verification](docs/verification.md): local checks and release gates.
- [Release Checklist](docs/release.md): pre-release steps.
- [Publishing](docs/publishing.md): crates.io package and publish workflow.
- [Contributing](CONTRIBUTING.md): development rules, verification, and secure
  review checklist.

## Verification

Run the local gate before integrating changes:

```sh
cargo fmt --all --check
cargo test --all-features
cargo test --examples
cargo clippy --all-targets --all-features -- -D warnings
cargo metadata --no-deps --format-version 1
```

For the fuller local release gate, run:

```sh
scripts/verify-release.sh
```

## Minimal Packet Scope

The codec validates a conservative `source>path:payload` shape:

- `source` is an uppercase ASCII callsign of 1-6 letters or digits with optional
  `-0` through `-15` SSID.
- `path` contains at least one component; the first component is the
  destination.
- path components use the same address rules, and digipeater components may end
  with `*`.
- `payload` must contain at least the APRS data type identifier byte.
- total packet length must be at most `MAX_PACKET_LEN`.

Semantic parsing covers status, position, timestamped position, compressed
position, messages, bulletins, announcements, acknowledgements, rejects,
objects/items with coordinate helpers, weather, telemetry, telemetry metadata,
queries, capabilities, NMEA identifiers/fields/checksum inspection, Mic-E
message codes and coordinates/speed/course when decodable, Maidenhead locator,
user-defined data, explicit third-party nested parsing, malformed data, and
unsupported data.

Malformed semantic handling is explicit: empty weather reports and third-party
bodies with invalid nested packet envelopes remain byte-preserving malformed
semantic payloads for policy rejection.
