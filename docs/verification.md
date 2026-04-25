# Verification

Run local verification before using a new revision or submitting a change.

## Required Local Checks

```sh
cargo fmt --all --check
cargo test
cargo test --all-features
cargo test --examples
cargo clippy --all-targets --all-features -- -D warnings
cargo metadata --no-deps --format-version 1
cargo doc --no-deps --all-features
cargo bench -p libaprs-engine
cargo +1.80.0 test --all-features
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
- Buildable examples that downstream developers can copy.
- Optional feature coverage, including serde diagnostics.
- A simple parser throughput benchmark.

## Current Remote CI Status

The repository contains `.github/workflows/rust-ci.yml`, but GitHub-hosted
Actions startup is currently blocked before job creation by account, billing, or
policy state outside this repository. Until that is resolved, use the local
verification commands above as the authoritative gate.

The `v0.1.0` tag was created from a local-only release gate with remote CI
intentionally skipped.

## Compatibility

The workspace crates declare `rust-version = "1.80"`. Verify with a toolchain
at or above that version.

## Dependency Scanning

The default core path remains dependency-light. When dependency use grows, add a
release gate for one of these tools:

```sh
cargo audit
cargo deny check
```
