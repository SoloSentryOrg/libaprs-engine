# Changelog

## Unreleased

## 0.4.0 - 2026-04-26

- Expanded weather field extraction to include negative temperatures,
  luminosity, 1000-plus luminosity, 24-hour snowfall, and raw rain counters.
- Added malformed-weather field tests so invalid optional fields are ignored
  without rejecting or mutating preserved packet bytes.
- Bumped all workspace crates to `0.4.0`.

## 0.3.0 - 2026-04-26

- Expanded the APRS101 conformance fixture corpus with source-referenced
  packet-family examples for status, position, messaging, objects, items,
  weather, telemetry, NMEA, Mic-E, Maidenhead, user-defined, third-party, and
  unsupported identifier handling.
- Added conformance tests that require APRS101 fixture source references and
  verify byte-for-byte raw packet preservation.
- Bumped all workspace crates to `0.3.0`.

## 0.2.0 - 2026-04-26

- Added shared transport contracts with `PacketSource`, `PacketSink`,
  `TransportErrorCode`, `DEFAULT_TRANSPORT_READ_LIMIT`, and
  `read_all_with_limit`.
- Added `Engine::process_packets` and `Engine::process_source` for direct
  engine integration with packet sources.
- Hardened CLI, file, TCP, APRS-IS, serial-like, HTTP, corpus, and file-watch
  input paths with bounded reads and stable `transport.oversized_input`
  diagnostics.
- Added CLI subcommands for `parse`, `validate`, `stats`, `explain`, and
  `replay` while keeping existing flag-oriented usage compatible.
- Added cargo-fuzz target scaffolding for parser, KISS, AX.25, and MQTT topic
  handling.
- Added release verification script, optional semver/fuzz gates, benchmark
  threshold support, and updated integration documentation.
- Bumped all workspace crates to `0.2.0`.

## 0.1.5 - 2026-04-26

- Changed UDP datagram reads to fail closed when a datagram exceeds the
  configured byte limit instead of risking silent truncation.
- Bumped all workspace crates to `0.1.5`.

## 0.1.4 - 2026-04-26

- Added transport helper crates for KISS, serial/readers, UDP datagrams, HTTP
  bodies, append-only file watching, MQTT payloads, AX.25 UI frames, corpus
  replay, in-process channels, and runtime-neutral async splitting.
- Bumped all workspace crates to `0.1.4`.
- Expanded downstream smoke coverage to include every published transport crate.

## 0.1.3 - 2026-04-26

- Added `aprs-transport-aprs-is` for APRS-IS login-line framing and
  reader-backed packet splitting, including CR/LF login-field rejection and
  bounded reader input.
- Added `ParsedPacket::summary()` for structured diagnostics with decoded
  helper details.
- Added CLI `--summary`, `--explain`, and `--fail-on` operator controls.
- Added a downstream smoke project that consumes published crates from
  crates.io.
- Added README badges, crate-selection guidance, and docs for APRS-IS and
  structured diagnostics.
- Added a scheduled/manual security workflow for `cargo audit` and
  `cargo deny check`.
- Enabled crates.io publishing readiness with versioned internal dependencies,
  core crate package validation in CI, and publishing documentation.
- Replaced early-skeleton status wording with a pre-1.0 readiness statement.
- Added API compatibility tests for documented stable-intent parser, engine,
  policy, transport, and helper APIs.
- Added dependency policy configuration in `deny.toml`.
- Expanded CI to check formatting, Cargo metadata, and documentation.
- Updated verification and security docs to reflect active GitHub Actions and
  dependency scanning policy.
- Documented pre-1.0 API compatibility policy and workspace crate stability
  tiers.
- Expanded conformance fixtures for telemetry metadata, checksummed NMEA, and
  Mic-E decoding.
- Added `aprs-transport-tcp` as an optional reader/TCP transport helper crate.
- Added CLI `--filter` and `--permissive` inspection options.

## 0.1.2 - 2026-04-25

- Added telemetry metadata semantics for `PARM.`, `UNIT.`, `EQNS.`, and
  `BITS.` message packets.
- Added NMEA checksum inspection that reports supplied and calculated checksum
  values without rejecting preserved packet bytes.
- Added Mic-E coordinate and speed/course helper decoding when destination and
  body bytes permit safe interpretation.
- Added explicit nested packet parsing for third-party traffic.
- Updated documentation for the expanded APRS semantic scope and `v0.1.2`
  dependency examples.

## 0.1.1 - 2026-04-25

- Fixed Rust `1.80.0` clippy compatibility while keeping stable clippy clean.
- Updated README project status to reflect examples, benchmark, file transport
  adapter, optional serde diagnostics, and active GitHub Actions.
- Updated GitHub Actions checkout from `v4` to `v6` for Node.js 24 support and
  removed the Node.js 20 deprecation warning.
- Supersedes `v0.1.0` for users who want passing remote CI evidence.

## 0.1.0 - 2026-04-25

- Added byte-preserving APRS packet parsing.
- Added APRS semantic packet family representation.
- Added typed interpretation helpers for coordinates, weather, telemetry, and Mic-E.
- Added policy, engine, transport, CLI, JSON diagnostics, conformance fixtures, and parser resilience tests.
- Added parser options, stable error/rejection codes, optional serde diagnostics,
  file transport adapter crate, buildable examples, benchmark target, stability
  docs, and APRS conformance matrix.
