#!/usr/bin/env sh
set -eu

cargo fmt --all --check
cargo test
cargo test --all-features
cargo test --examples
cargo clippy --all-targets --all-features -- -D warnings
cargo metadata --no-deps --format-version 1 >/dev/null
cargo doc --no-deps --all-features
cargo package -p libaprs-engine

if [ "${LIBAPRS_RUN_DOWNSTREAM_SMOKE:-0}" = "1" ]; then
  cargo check --manifest-path examples/downstream-smoke/Cargo.toml
else
  echo "LIBAPRS_RUN_DOWNSTREAM_SMOKE is not set; skipping crates.io downstream smoke" >&2
fi

if command -v cargo-semver-checks >/dev/null 2>&1; then
  cargo semver-checks check-release -p libaprs-engine
else
  echo "cargo-semver-checks not installed; skipping semver gate" >&2
fi

if command -v cargo-fuzz >/dev/null 2>&1; then
  cargo fuzz check
else
  echo "cargo-fuzz not installed; skipping fuzz compile gate" >&2
fi
