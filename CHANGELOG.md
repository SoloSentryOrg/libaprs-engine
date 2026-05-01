# Changelog

## 2.0.0-rc.2 - 2026-05-01

- Refreshed package metadata after migrating the repository from the personal
  namespace to `SoloSentryOrg/libaprs-engine`.
- Updated README, API examples, transport examples, publishing guidance, and
  downstream smoke dependencies to target `2.0.0-rc.2`.
- Bumped all workspace crates to `2.0.0-rc.2`.

## 2.0.0-rc.1 - 2026-04-30

- Removed `ParsedPacket::to_json()` from the library public API. Use
  `ParsedPacket::to_diagnostic()` with the `serde` feature,
  `serde_support::PacketDiagnostic`, `PacketSummary`, `EngineEvent`, or an
  application-owned schema instead.
- Added `schema_version` to `serde_support::PacketDiagnostic` so serde-backed
  diagnostic integrations have an explicit schema marker.
- Kept accepted-packet CLI JSON as a CLI-owned diagnostic output and added
  `schema_version` to that output.
- Recorded the approved `v2.0.0-rc.1` breaking-change evidence and migration
  path in the v2 planning documents.
- Bumped all workspace crates to `2.0.0-rc.1`.

## 1.7.0 - 2026-04-30

- Added `ParsedPacket::to_diagnostic()` behind the `serde` feature as an
  explicit structured diagnostic replacement path for integrations that should
  not rely on `to_json()` as an external schema.
- Added a downstream feedback issue template for API, migration, and integration
  evidence capture before any future breaking change.
- Bumped all workspace crates to `1.7.0`.

## 1.6.0 - 2026-04-30

- Expanded APRS101 conformance and semantic helper coverage for `1.x`
  production adoption.
- Added abuse-resistance release gates, threat-model documentation, fuzz corpus
  hygiene checks, and resource-exhaustion coverage.
- Added stable observability event structs, bounded malformed-event raw-byte
  evidence, dependency-free metrics helpers, and JSON schema documentation.
- Hardened transport integration guidance with TCP read options, timeout
  coverage, APRS-IS reconnect examples, and transport common-layer decisions.
- Added downstream feedback and `v2.0.0` migration planning documentation.
- Documented soft deprecations for weak or confusing integration patterns
  without removing `1.x` APIs.
- Expanded compatibility tripwires for serde diagnostics and TCP read options.
- Bumped all workspace crates to `1.6.0`.

## 1.1.0 - 2026-04-29

- Added structured parser, policy, and transport error diagnostics with stable
  layers, codes, descriptions, and remediation guidance.
- Added a machine-readable CLI support matrix for semantic families, transport
  adapters, and diagnostic layers.
- Added an operator-focused deployment guide for diagnostics, logging, limits,
  and safe defaults.
- Added a compile-tested service ingestion example that logs stable diagnostic
  codes while preserving raw-byte parser boundaries.
- Bumped all workspace crates to `1.1.0`.

## 1.0.0 - 2026-04-28

- Promoted the tested `1.0.0-rc.1` release candidate to final `1.0.0`.
- Locked the documented public API boundary under `1.0.0` semantic-versioning
  guarantees.
- Bumped all workspace crates to `1.0.0`.

## 1.0.0-rc.1 - 2026-04-28

- Cut the first `1.0.0` release candidate after completing the API
  stabilization, APRS semantics, security, robustness, transport reliability,
  and CI release-gate hardening roadmap batches.
- Tightened semantic validation for coordinate ranges, object timestamps,
  short Mic-E bodies, and Maidenhead locator syntax while preserving raw packet
  bytes for rejected semantic payloads.
- Added APRS101 malformed semantic golden fixtures and strict-policy rejection
  coverage for malformed semantic families.
- Added parser robustness tests for exact packet-size boundaries, invalid
  address bytes, and deterministic mutation corpora.
- Hardened transport adapters with bounded packet-line splitting, fail-closed
  KISS and AX.25 frame-size checks, stable UDP oversized diagnostics, and
  byte-preserving transport regression tests.
- Hardened CI release gates so advisory and dependency-policy checks run during
  the Rust CI release-script job.
- Reduced release-script CI tool-install latency by keeping pinned tool
  versions and verifying SHA-256 checksums for pinned upstream release
  archives before use.
- Bumped all workspace crates to `1.0.0-rc.1`.

## 0.6.0 - 2026-04-28

- Added object/item coordinate helpers, NMEA sentence field helpers, and Mic-E
  message-code helpers while preserving existing byte-oriented APIs.
- Added malformed semantic and mutation regression tests for fail-closed parser
  behavior.
- Added compile-tested transport cookbook examples for APRS-IS, KISS, UDP, and
  corpus replay.
- Added contributor guidance and GitHub issue templates for bugs, parser
  fixtures, and transport requests.
- Bumped all workspace crates to `0.6.0`.

## 0.5.0 - 2026-04-27

- Added opt-in policy rejection for NMEA checksum mismatches while preserving
  codec raw-byte behavior and checksum reporting.
- Added stable `policy.nmea_checksum_mismatch` rejection code.
- Bumped all workspace crates to `0.5.0`.

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
