#!/usr/bin/env sh
set -eu

failures=0

note_failure() {
  echo "merge gate guard test failed: $1" >&2
  failures=$((failures + 1))
}

expect_checks() {
  description="$1"
  files="$2"
  expected="$3"

  actual="$(
    MERGE_GATE_CHANGED_FILES="$files" scripts/check-merge-gate.sh --print-required-checks \
      | paste -sd ',' -
  )"

  if [ "$actual" != "$expected" ]; then
    note_failure "$description: expected '$expected', got '$actual'"
  fi
}

expect_checks \
  "docs-only changes require only Docs" \
  "README.md
docs/release.md
.github/ISSUE_TEMPLATE/bug_report.md" \
  "Docs"

expect_checks \
  "source changes require Rust checks" \
  "crates/libaprs-engine/src/lib.rs" \
  "Rust stable,Rust 1.80.0"

expect_checks \
  "docs verifier changes require Docs and Rust checks" \
  "scripts/verify-docs.sh" \
  "Docs,Rust stable,Rust 1.80.0"

expect_checks \
  "security-sensitive changes require Rust and security checks" \
  "Cargo.lock" \
  "Rust stable,Rust 1.80.0,cargo-security"

expect_checks \
  "mixed docs and security changes require all relevant checks" \
  "README.md
Cargo.toml" \
  "Docs,Rust stable,Rust 1.80.0,cargo-security"

expect_checks \
  "empty file list falls back to Rust checks" \
  "" \
  "Rust stable,Rust 1.80.0"

if [ "$failures" -ne 0 ]; then
  exit 1
fi

echo "merge gate guard tests passed"
