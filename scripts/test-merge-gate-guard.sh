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
  "docs-only changes require secret scan and Docs" \
  "README.md
docs/release.md
.github/ISSUE_TEMPLATE/bug_report.md" \
  "Secret Scan,Docs"

expect_checks \
  "source changes require secret scan and Rust checks" \
  "crates/libaprs-engine/src/lib.rs" \
  "Secret Scan,Rust stable,Rust 1.80.0"

expect_checks \
  "docs verifier changes require secret scan, Docs, and Rust checks" \
  "scripts/verify-docs.sh" \
  "Secret Scan,Docs,Rust stable,Rust 1.80.0"

expect_checks \
  "security-sensitive changes require secret scan, Rust, and security checks" \
  "Cargo.lock" \
  "Secret Scan,Rust stable,Rust 1.80.0,cargo-security"

expect_checks \
  "mixed docs and security changes require all relevant checks including secret scan" \
  "README.md
Cargo.toml" \
  "Secret Scan,Docs,Rust stable,Rust 1.80.0,cargo-security"

expect_checks \
  "empty file list falls back to secret scan and Rust checks" \
  "" \
  "Secret Scan,Rust stable,Rust 1.80.0"

if [ "$failures" -ne 0 ]; then
  exit 1
fi

echo "merge gate guard tests passed"
