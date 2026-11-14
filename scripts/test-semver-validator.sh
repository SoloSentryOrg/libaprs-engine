#!/usr/bin/env sh
set -eu

# Exercise rustdoc parsing and a real breaking-change rejection with the
# selected Rust toolchain. Nothing is published; all fixture data is temporary.
fixture_dir="$(mktemp -d "${TMPDIR:-/tmp}/libaprs-semver-validator.XXXXXX")"
trap 'rm -rf "$fixture_dir"' EXIT HUP INT TERM

for variant in baseline compatible breaking; do
  mkdir -p "$fixture_dir/$variant/src"
  cat > "$fixture_dir/$variant/Cargo.toml" <<'EOF'
[package]
name = "libaprs-semver-validator-fixture"
version = "1.0.0"
edition = "2021"
publish = false

[workspace]
EOF
  printf '%s\n' 'pub fn preserved() {}' > "$fixture_dir/$variant/src/lib.rs"
done

printf '%s\n' 'pub fn removed() {}' >> "$fixture_dir/baseline/src/lib.rs"
printf '%s\n' 'pub fn removed() {}' >> "$fixture_dir/compatible/src/lib.rs"

cargo semver-checks check-release \
  --manifest-path "$fixture_dir/compatible/Cargo.toml" \
  --baseline-root "$fixture_dir/baseline" --release-type patch

status=0
cargo semver-checks check-release \
  --manifest-path "$fixture_dir/breaking/Cargo.toml" \
  --baseline-root "$fixture_dir/baseline" --release-type patch \
  > "$fixture_dir/breaking.log" 2>&1 || status=$?

# Exit 100 means an actual semver violation. Tool/schema errors (101), skipped
# analysis and unexpected success must fail this regression test.
if [ "$status" -ne 100 ]; then
  cat "$fixture_dir/breaking.log" >&2
  echo "expected a semver violation (100), observed $status" >&2
  exit 1
fi

echo "semver validator accepts compatible API and rejects removed public API"
