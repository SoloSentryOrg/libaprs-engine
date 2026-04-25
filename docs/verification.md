# Verification

Run local verification before using a new revision or submitting a change.

## Required Local Checks

```sh
cargo test
cargo metadata --no-deps --format-version 1
cargo clippy --all-targets --all-features -- -D warnings
```

## What The Checks Cover

- Unit and integration tests for codec behavior.
- Conformance fixtures for valid and malformed packets.
- Deterministic byte-fuzz tests to catch panics and raw-byte preservation
  regressions.
- CLI tests for stdin and invalid UTF-8 payload handling.
- Engine and policy tests for accepted, rejected, and malformed counters.
- Clippy with warnings denied.
- Cargo metadata validation for workspace consumers.

## Current Remote CI Status

The repository contains `.github/workflows/rust-ci.yml`, but GitHub-hosted
Actions startup is currently blocked before job creation by account, billing, or
policy state outside this repository. Until that is resolved, use the local
verification commands above as the authoritative gate.

## Compatibility

The workspace declares `rust-version = "1.80"` for both crates. Verify with a
toolchain at or above that version.
