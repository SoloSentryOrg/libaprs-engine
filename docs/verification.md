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
```

The repository also provides a local release gate script:

```sh
scripts/verify-release.sh
```

The publish guard also has a focused test:

```sh
scripts/test-publish-release-guards.sh
```

That test stubs `git` and `cargo` and verifies `scripts/publish-release.sh`
does not publish unless clean secure-review evidence, local release evidence,
security-gate evidence, remote-CI evidence, and release-commit evidence are
provided.

Before crates are published, the script and pull-request CI skip the crates.io
downstream smoke project by default because unpublished version requirements
cannot resolve from crates.io. After publishing, run locally:

```sh
LIBAPRS_RUN_DOWNSTREAM_SMOKE=1 scripts/verify-release.sh
```

Or trigger the `Rust CI` workflow manually with `downstream_smoke=true`.

After `libaprs-engine` is published and visible in the crates.io index, validate
all dependent packages with:

```sh
LIBAPRS_PACKAGE_ALL=1 scripts/verify-release.sh
```

Run benchmark checks only when parser performance changed:

```sh
LIBAPRS_RUN_BENCH=1 scripts/verify-release.sh
```

The script runs the core checks and uses optional gates when the tools are
installed:

- `cargo-semver-checks` for public API compatibility.
- `cargo audit` for RustSec advisory checks.
- `cargo deny check` for dependency policy checks.
- `cargo-fuzz` for fuzz target compile checks.
- Rust `1.80.0` tests and clippy when the MSRV toolchain is installed.

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
- Optional semantic-family fuzz targets for weather, telemetry, messages, and
  explicit third-party nested parsing.
- CI release-script coverage installs `cargo-semver-checks`, `cargo-audit`, and
  `cargo-deny` so public API compatibility, advisory checks, and dependency
  policy are checked in the normal release path.
- CI release-script coverage runs `scripts/test-publish-release-guards.sh` so
  publication cannot regress to a path that skips secure-review evidence.

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
cargo fuzz run weather_semantics
cargo fuzz run telemetry_semantics
cargo fuzz run message_semantics
cargo fuzz run third_party_semantics
```

Seed corpora live under `fuzz/corpus/` and include representative parser and
semantic-family packets. Add minimized regression inputs there only when they
are safe to publish and do not contain private station or operator data.

Fuzz findings should be reduced to deterministic regression tests before
merging parser or transport changes.

## Restricted Cargo Home

Some sandboxed development environments can read `~/.cargo` but cannot write
registry, package-cache, advisory, or lock files there. In those environments,
use a writable temporary Cargo home outside the repository:

```sh
mkdir -p /tmp/libaprs-cargo-home
CARGO_HOME=/tmp/libaprs-cargo-home scripts/verify-release.sh
```

For publishing from a restricted environment, copy crates.io credentials into
that temporary Cargo home at runtime only, then run `cargo publish` with
`CARGO_HOME=/tmp/libaprs-cargo-home`. Do not store Cargo credentials, registry
caches, package caches, advisory databases, or temporary Cargo homes inside the
repository.

## Current Remote CI Status

The repository contains `.github/workflows/rust-ci.yml`. It runs on pushes,
pull requests, and manual dispatch for Rust `1.80.0` and stable. The CI gate
checks formatting, tests, all-features tests, examples, Cargo metadata, docs,
package validation, downstream smoke, fuzz workspace formatting, and clippy
with warnings denied. It also runs `scripts/verify-release.sh` as a dedicated
job with pinned semver, audit, and deny tools, so the required local release
gate and dependency-policy checks cannot drift from CI. Fuzz and benchmarks
remain local or release-time gates unless installed in that job.

The repository also has a scheduled/manual security workflow that runs
`cargo audit` and `cargo deny check`. It also runs on dependency, manifest,
lockfile, dependency-policy, and security-workflow changes.

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
