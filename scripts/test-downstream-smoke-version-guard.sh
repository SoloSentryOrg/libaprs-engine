#!/usr/bin/env sh
set -eu

fail() {
  echo "$1" >&2
  exit 1
}

tmp="${TMPDIR:-/tmp}/libaprs-downstream-smoke-version-test.$$"
trap 'rm -rf "$tmp"' EXIT HUP INT TERM
mkdir -p "$tmp/crates/libaprs-engine" "$tmp/examples/downstream-smoke"

cat >"$tmp/crates/libaprs-engine/Cargo.toml" <<'EOF'
[package]
name = "libaprs-engine"
version = "2.0.0-rc.1"
EOF

cat >"$tmp/examples/downstream-smoke/Cargo.toml" <<'EOF'
[dependencies]
libaprs-engine = { version = "2.0.0-rc.1", features = ["serde"] }
aprs-transport-file = "2.0.0-rc.1"
EOF

scripts/check-downstream-smoke-versions.sh \
  "$tmp/examples/downstream-smoke/Cargo.toml" \
  "$tmp/crates/libaprs-engine/Cargo.toml"

cat >"$tmp/examples/downstream-smoke/Cargo.toml" <<'EOF'
[dependencies]
libaprs-engine = { version = "2.0.0-rc.1", features = ["serde"] }
aprs-transport-file = "1.7.0"
EOF

if scripts/check-downstream-smoke-versions.sh \
  "$tmp/examples/downstream-smoke/Cargo.toml" \
  "$tmp/crates/libaprs-engine/Cargo.toml" \
  >"$tmp/out" 2>"$tmp/err"; then
  fail "stale downstream smoke version was accepted"
fi

grep -q "aprs-transport-file" "$tmp/err" || fail "stale dependency was not reported"

echo "downstream smoke version guard tests passed"
