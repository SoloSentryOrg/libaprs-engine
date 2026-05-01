#!/usr/bin/env sh
set -eu

workflow=".github/workflows/rust-ci.yml"
docs_workflow=".github/workflows/docs.yml"
merge_gate_workflow=".github/workflows/merge-gate.yml"
failures=0

note_failure() {
  echo "workflow optimization check failed: $1" >&2
  failures=$((failures + 1))
}

require_text() {
  pattern="$1"
  description="$2"

  if ! grep -F "$pattern" "$workflow" >/dev/null 2>&1; then
    note_failure "$description"
  fi
}

require_file() {
  file="$1"
  description="$2"

  if [ ! -f "$file" ]; then
    note_failure "$description"
  fi
}

require_text_in_file() {
  file="$1"
  pattern="$2"
  description="$3"

  if [ ! -f "$file" ] || ! grep -F "$pattern" "$file" >/dev/null 2>&1; then
    note_failure "$description"
  fi
}

require_text "Release script" "Rust CI should keep the release-script job"
require_text "if: \${{ github.event_name != 'pull_request' }}" "release-script job should not run on pull requests"
require_text "Cache Cargo registry" "Rust matrix should cache registry/git state separately from target"
require_text "Cargo cache summary" "Rust matrix should emit cache-size diagnostics"
require_text "paths-ignore:" "Rust CI should skip docs-only pull requests and pushes"
require_file "$docs_workflow" "docs-only fast-lane workflow should exist"
require_text_in_file "$docs_workflow" "name: Docs" "docs workflow should have a stable name"
require_text_in_file "$docs_workflow" "scripts/verify-docs.sh" "docs workflow should run the docs verifier"
require_file "$merge_gate_workflow" "merge-gate workflow should exist for branch protection"
require_text_in_file "$merge_gate_workflow" "name: Merge Gate" "merge-gate workflow should have a stable required-check name"
require_text_in_file "$merge_gate_workflow" "scripts/check-merge-gate.sh" "merge-gate workflow should run the merge-gate verifier"

release_cache_block="$(
  awk '
    /^  release-script:/ { in_release_job = 1 }
    in_release_job && /^  [A-Za-z0-9_-]+:/ && $0 !~ /^  release-script:/ { in_release_job = 0 }
    in_release_job && /^      - name: Cache Cargo/ { in_cache_step = 1; next }
    in_cache_step && /^      - name:/ { exit }
    in_cache_step { print }
  ' "$workflow"
)"
if [ -z "$release_cache_block" ]; then
  note_failure "release-script should cache Cargo registry/git state"
elif printf '%s\n' "$release_cache_block" | grep -Fx "            target" >/dev/null 2>&1; then
  note_failure "release-script cache should not include target build artifacts"
fi

stable_only_count="$(grep -F -c "matrix.toolchain == 'stable'" "$workflow" || true)"
if [ "$stable_only_count" -lt 5 ]; then
  note_failure "stable-only checks should avoid duplicating docs/package/default tests on MSRV"
fi

if [ "$failures" -ne 0 ]; then
  exit 1
fi

echo "workflow optimization checks passed"
