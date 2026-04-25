# Changelog

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
