#!/usr/bin/env sh
set -eu

failures=0

note_failure() {
  echo "v2.6.0 evidence verification failed: $1" >&2
  failures=$((failures + 1))
}

if ! grep -F '`v2.6.0`: Evidence-First Readiness' ROADMAP.md >/dev/null 2>&1; then
  note_failure "ROADMAP.md is missing the v2.6.0 evidence-first milestone"
fi

if [ ! -f docs/release-notes-v2.6.0.md ]; then
  note_failure "docs/release-notes-v2.6.0.md is missing"
fi

if ! grep -F 'internal downstream evidence log' docs/downstream-feedback.md >/dev/null 2>&1; then
  note_failure "docs/downstream-feedback.md must be marked internal"
fi

if ! grep -F 'No `v3.0.0` public API breaking change is approved for `v3.0.0-rc.2`.' docs/v3-breaking-changes.md >/dev/null 2>&1; then
  note_failure "v3 breaking-change decision must remain evidence-based"
fi

if [ "$failures" -ne 0 ]; then
  exit 1
fi

echo "v2.6.0 evidence verification passed"
