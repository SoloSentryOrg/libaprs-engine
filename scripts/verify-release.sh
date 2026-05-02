#!/usr/bin/env sh
set -eu

run() {
  printf '+ %s\n' "$*" >&2
  "$@"
}

has_tool() {
  command -v "$1" >/dev/null 2>&1
}

has_rust_toolchain() {
  if ! has_tool rustup; then
    return 1
  fi

  rustup toolchain list | awk -v toolchain="$1" '
    $1 == toolchain || index($1, toolchain "-") == 1 { found = 1 }
    END { exit found ? 0 : 1 }
  '
}

package_crate() {
  run cargo package -p "$1"
}

LIBAPRS_MSRV="${LIBAPRS_MSRV:-1.80.0}"

run scripts/test-publish-release-guards.sh
run scripts/test-fuzz-corpus-guard.sh
run scripts/test-downstream-smoke-version-guard.sh
run scripts/test-merge-gate-guard.sh
run scripts/check-workflow-optimizations.sh
run scripts/verify-docs.sh
run scripts/check-fuzz-corpus.sh
if has_tool gitleaks; then
  run scripts/check-secrets.sh
else
  echo "gitleaks not installed; skipping secret-history scan" >&2
fi
run cargo fmt --all --check
run cargo fmt --manifest-path fuzz/Cargo.toml --all --check
run cargo test
run cargo test --all-features
run cargo test --examples
run cargo clippy --all-targets --all-features -- -D warnings
run cargo metadata --no-deps --format-version 1 >/dev/null
run cargo doc --no-deps --all-features
package_crate libaprs-engine

if has_rust_toolchain "$LIBAPRS_MSRV"; then
  run cargo +"$LIBAPRS_MSRV" test --all-features
  run cargo +"$LIBAPRS_MSRV" clippy --all-targets --all-features -- -D warnings
else
  echo "Rust toolchain $LIBAPRS_MSRV is not installed; skipping MSRV gate" >&2
fi

if [ "${LIBAPRS_RUN_DOWNSTREAM_SMOKE:-0}" = "1" ]; then
  run scripts/check-downstream-smoke-versions.sh
  run cargo check --manifest-path examples/downstream-smoke/Cargo.toml
else
  echo "LIBAPRS_RUN_DOWNSTREAM_SMOKE is not set; skipping crates.io downstream smoke" >&2
fi

if [ "${LIBAPRS_PACKAGE_ALL:-0}" = "1" ]; then
  package_crate aprs-transport-file
  package_crate aprs-transport-tcp
  package_crate aprs-transport-aprs-is
  package_crate aprs-transport-async
  package_crate aprs-transport-ax25
  package_crate aprs-transport-channel
  package_crate aprs-transport-corpus
  package_crate aprs-transport-file-watch
  package_crate aprs-transport-http
  package_crate aprs-transport-kiss
  package_crate aprs-transport-mqtt
  package_crate aprs-transport-serial
  package_crate aprs-transport-udp
  package_crate aprs-cli
else
  echo "LIBAPRS_PACKAGE_ALL is not set; skipping dependent crate package validation" >&2
fi

if has_tool cargo-semver-checks; then
  run cargo semver-checks check-release -p libaprs-engine
else
  echo "cargo-semver-checks not installed; skipping semver gate" >&2
fi

if has_tool cargo-audit; then
  run cargo audit
else
  echo "cargo-audit not installed; skipping advisory gate" >&2
fi

if has_tool cargo-deny; then
  run cargo deny check
else
  echo "cargo-deny not installed; skipping dependency policy gate" >&2
fi

if has_tool cargo-fuzz && has_rust_toolchain nightly; then
  run cargo +nightly fuzz check
elif has_tool cargo-fuzz; then
  echo "nightly Rust toolchain is not installed; skipping fuzz compile gate" >&2
else
  echo "cargo-fuzz not installed; skipping fuzz compile gate" >&2
fi

if [ "${LIBAPRS_RUN_BENCH:-0}" = "1" ]; then
  run cargo bench -p libaprs-engine
else
  echo "LIBAPRS_RUN_BENCH is not set; skipping benchmark gate" >&2
fi
