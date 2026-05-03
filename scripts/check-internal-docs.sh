#!/usr/bin/env sh
set -eu

failures=0

note_failure() {
  echo "internal docs verification failed: $1" >&2
  failures=$((failures + 1))
}

if git grep -n -E '(\.\./)?docs/downstream-feedback\.md|downstream-feedback\.md' -- README.md .github docs \
  ':!docs/downstream-feedback.md' \
  ':!docs/release.md' \
  ':!docs/v2-breaking-changes.md' \
  ':!docs/v3-breaking-changes.md' \
  ':!docs/api-guidelines-audit.md' \
  ':!docs/superpowers/**'; then
  note_failure "internal downstream feedback log is linked from public-facing docs"
fi

if [ -e .github/ISSUE_TEMPLATE/downstream_feedback.md ]; then
  note_failure "internal downstream feedback must not be exposed as a public issue template"
fi

if [ "$failures" -ne 0 ]; then
  exit 1
fi

echo "internal docs verification passed"
