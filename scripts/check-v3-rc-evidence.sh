#!/usr/bin/env sh
set -eu

failures=0

note_failure() {
  echo "v3.0.0-rc.2 evidence verification failed: $1" >&2
  failures=$((failures + 1))
}

# The grep patterns intentionally include literal Markdown backticks.
# shellcheck disable=SC2016
if ! grep -F '`v3.0.0-rc.2`: Security Fix-Forward Release Candidate' ROADMAP.md >/dev/null 2>&1; then
  note_failure "ROADMAP.md is missing the v3.0.0-rc.2 security fix-forward release-candidate milestone"
fi

if [ ! -f docs/release-notes-v3.0.0-rc.2.md ]; then
  note_failure "docs/release-notes-v3.0.0-rc.2.md is missing"
fi

if [ ! -f docs/v3-migration.md ]; then
  note_failure "docs/v3-migration.md is missing"
fi

# shellcheck disable=SC2016
if ! grep -F 'No `v3.0.0` public API breaking change is approved for `v3.0.0-rc.2`.' docs/v3-breaking-changes.md >/dev/null 2>&1; then
  note_failure "v3 breaking-change decision must state the no-intentional-break RC scope"
fi

if ! grep -F 'No intentional public API break' docs/v3-migration.md >/dev/null 2>&1; then
  note_failure "v3 migration guide must state the no-intentional-break migration"
fi

if ! grep -F '## v3.0.0-rc.2 Release Preparation Evidence' docs/release.md >/dev/null 2>&1 &&
  ! grep -F '## v3.0.0-rc.2 Release Evidence' docs/release.md >/dev/null 2>&1; then
  note_failure "docs/release.md is missing v3.0.0-rc.2 release evidence"
fi

if [ "$failures" -ne 0 ]; then
  exit 1
fi

echo "v3.0.0-rc.2 evidence verification passed"
