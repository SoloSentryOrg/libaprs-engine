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
cargo package -p libaprs-engine
cargo bench -p libaprs-engine
cargo +1.80.0 test --all-features
cargo +1.80.0 clippy --all-targets --all-features -- -D warnings
cargo check --manifest-path examples/downstream-smoke/Cargo.toml
```

## What The Checks Cover

- Unit and integration tests for codec behavior.
- Conformance fixtures for valid and malformed packets.
- API compatibility tests for documented stable-intent API surfaces.
- Deterministic byte-fuzz tests to catch panics and raw-byte preservation
  regressions.
- CLI tests for stdin and invalid UTF-8 payload handling.
- Engine and policy tests for accepted, rejected, and malformed counters.
- Clippy with warnings denied.
- Cargo metadata validation for workspace consumers.
- Crates.io package validation for the core crate. Dependent crates can be fully
  packaged after `libaprs-engine` is available in the crates.io index.
- Buildable examples that downstream developers can copy.
- A downstream smoke project that consumes the published crates from crates.io.
- Optional feature coverage, including serde diagnostics.
- A simple parser throughput benchmark.

## Current Remote CI Status

The repository contains `.github/workflows/rust-ci.yml`. It runs on pushes,
pull requests, and manual dispatch for Rust `1.80.0` and stable. The CI gate
checks formatting, tests, all-features tests, examples, Cargo metadata, docs,
and clippy with warnings denied.

The repository also has a scheduled/manual security workflow that runs
`cargo audit` and `cargo deny check`.

## Compatibility

The workspace crates declare `rust-version = "1.80"`. Verify with a toolchain
at or above that version.

## Dependency Scanning

The default core path remains dependency-light. The repository includes
`deny.toml` for dependency policy. Run `cargo deny check` when `cargo-deny` is
installed. Run `cargo audit` when `cargo-audit` is installed.

```sh
cargo audit
cargo deny check
```
