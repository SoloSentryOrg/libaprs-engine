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
cargo fmt --manifest-path fuzz/Cargo.toml --all --check
cargo +1.80.0 test --all-features
cargo +1.80.0 clippy --all-targets --all-features -- -D warnings
cargo check --manifest-path examples/downstream-smoke/Cargo.toml
```

The repository also provides a local release gate script:

```sh
scripts/verify-release.sh
```

Before crates are published, the script skips the crates.io downstream smoke
project by default. After publishing, run:

```sh
LIBAPRS_RUN_DOWNSTREAM_SMOKE=1 scripts/verify-release.sh
```

The script runs the core checks and uses optional gates when the tools are
installed:

- `cargo-semver-checks` for public API compatibility.
- `cargo-fuzz` for fuzz target compile checks.

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
- Optional cargo-fuzz targets for the parser, KISS decoder, AX.25 decoder, and
  MQTT topic matcher.

## Benchmark Threshold

`parser_throughput` can enforce a local threshold when
`LIBAPRS_MAX_NS_PER_PACKET` is set:

```sh
LIBAPRS_MAX_NS_PER_PACKET=5000 cargo bench -p libaprs-engine
```

Leave the variable unset for informational benchmark runs.

## Fuzzing

Install `cargo-fuzz` and run targets from the repository root:

```sh
cargo install cargo-fuzz
cargo fuzz run parse_packet
cargo fuzz run kiss_decode
cargo fuzz run ax25_decode
cargo fuzz run mqtt_topic
```

Fuzz findings should be reduced to deterministic regression tests before
merging parser or transport changes.

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
