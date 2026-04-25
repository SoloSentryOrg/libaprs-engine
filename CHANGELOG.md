# Changelog

## Unreleased

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
