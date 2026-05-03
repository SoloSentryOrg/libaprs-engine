#!/usr/bin/env sh
set -eu

failures=0

note_failure() {
  echo "docs verification failed: $1" >&2
  failures=$((failures + 1))
}

if git grep -n '[[:blank:]]$' -- '*.md'; then
  note_failure "markdown files contain trailing whitespace"
fi

if git grep -n -E 'github\.com/elodiejmirza/libaprs-engine|elodiejmirza/libaprs-engine' -- '*.md'; then
  note_failure "markdown files contain stale personal repository references"
fi

for required_heading in \
  "## Quick PR Verification" \
  "## Full Release Verification" \
  "## Codex Environment Startup"; do
  if ! grep -F "$required_heading" docs/verification.md >/dev/null 2>&1; then
    note_failure "docs/verification.md is missing $required_heading"
  fi
done

sh scripts/check-internal-docs.sh || failures=$((failures + 1))
sh scripts/check-v2-6-evidence.sh || failures=$((failures + 1))

if [ "$failures" -ne 0 ]; then
  exit 1
fi

echo "docs verification passed"
