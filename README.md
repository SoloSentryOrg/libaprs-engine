# libaprs-engine

Protocol-first APRS parsing and inspection for Rust.

`libaprs-engine` is an early, byte-preserving APRS engine. It accepts untrusted
packet bytes, keeps the original bytes intact, rejects malformed packet shape at
the codec boundary, and exposes structured APRS views for downstream policy,
telemetry, indexing, and diagnostics.

## Project Status

- Early skeleton with meaningful APRS semantics, tests, examples, benchmark,
  file transport adapter, and CLI inspector.
- Current tagged release: `v0.1.0`.
- Public API is usable, but not yet covered by semantic versioning stability
  guarantees.
- Core runtime remains network-free and async-free. Optional `serde`
  diagnostics and the separate file transport adapter are available.
- GitHub Actions workflow is active and checks Rust `1.80.0` plus stable.

## Workspace Crates

- `libaprs-engine`: library crate with packet types, parser, semantic views,
  policy, engine orchestration, counters, JSON diagnostics, and line transport.
- `aprs-cli`: command-line packet inspector built on the library crate.
- `aprs-transport-file`: optional file transport helper crate that reads packet
  files as bytes and returns newline-separated packet byte vectors.

## Install Or Depend On It

This repository is not published to crates.io. Use a Git dependency or a local
path dependency.

```toml
[dependencies]
libaprs-engine = { git = "https://github.com/elodiejmirza/libaprs-engine", package = "libaprs-engine", tag = "v0.1.0" }
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
cat packets.aprs | cargo run -p aprs-cli -- --json
```

## Security Model

- Treat every packet as untrusted bytes.
- Preserve raw input exactly for accepted packets.
- Reject empty, oversized, malformed, or non-AX.25-like packet shape.
- Do not trim, lowercase, normalize, or lossy-convert packet bytes before
  calling the parser.
- Keep payload bytes opaque; they may not be valid UTF-8.
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
- [Architecture](docs/architecture.md): boundaries, contracts, and pipeline.
- [Security Model](docs/security.md): untrusted input handling and OWASP-aligned
  controls.
- [Stability](docs/stability.md): API stability levels and feature flags.
- [Conformance Matrix](docs/conformance.md): APRS family support and known gaps.
- [Verification](docs/verification.md): local checks and release gates.
- [Release Checklist](docs/release.md): pre-release steps.

## Verification

Run the local gate before integrating changes:

```sh
cargo fmt --all --check
cargo test --all-features
cargo test --examples
cargo clippy --all-targets --all-features -- -D warnings
cargo metadata --no-deps --format-version 1
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
objects, items, weather, telemetry, queries, capabilities, NMEA, Mic-E,
Maidenhead locator, user-defined data, third-party traffic, malformed data, and
unsupported data.
