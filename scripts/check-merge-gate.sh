#!/usr/bin/env sh
set -eu

poll_interval="${MERGE_GATE_POLL_INTERVAL_SECONDS:-10}"
timeout_seconds="${MERGE_GATE_TIMEOUT_SECONDS:-900}"

usage() {
  cat <<'USAGE'
Usage: scripts/check-merge-gate.sh [--print-required-checks]

Checks the PR head SHA for the workflow checks that are relevant to the changed
files. Set MERGE_GATE_CHANGED_FILES to a newline-separated file list for local
classification tests.
USAGE
}

print_only=0
if [ "${1:-}" = "--print-required-checks" ]; then
  print_only=1
elif [ "${1:-}" = "--help" ]; then
  usage
  exit 0
elif [ "${1:-}" != "" ]; then
  usage >&2
  exit 2
fi

changed_files() {
  if [ "${MERGE_GATE_CHANGED_FILES+x}" = "x" ]; then
    printf '%s\n' "$MERGE_GATE_CHANGED_FILES"
    return
  fi

  if [ -z "${GITHUB_REPOSITORY:-}" ] || [ -z "${PR_NUMBER:-}" ]; then
    echo "GITHUB_REPOSITORY and PR_NUMBER are required unless MERGE_GATE_CHANGED_FILES is set" >&2
    exit 2
  fi

  gh api --paginate "/repos/$GITHUB_REPOSITORY/pulls/$PR_NUMBER/files" --jq '.[].filename'
}

triggers_docs_workflow() {
  case "$1" in
    *.md | .github/ISSUE_TEMPLATE/* | .github/workflows/docs.yml | scripts/verify-docs.sh)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

is_rust_ci_ignored_path() {
  case "$1" in
    *.md | .github/ISSUE_TEMPLATE/*)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

is_security_path() {
  case "$1" in
    Cargo.toml | Cargo.lock | crates/*/Cargo.toml | examples/downstream-smoke/Cargo.toml | \
      examples/downstream-smoke/Cargo.lock | fuzz/Cargo.toml | fuzz/Cargo.lock | deny.toml | \
      scripts/check-fuzz-corpus.sh | scripts/test-fuzz-corpus-guard.sh | \
      scripts/install-release-tools.sh | .github/workflows/security.yml)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

is_supply_chain_path() {
  case "$1" in
    Cargo.toml | Cargo.lock | crates/*/Cargo.toml | examples/downstream-smoke/Cargo.toml | \
      examples/downstream-smoke/Cargo.lock | fuzz/Cargo.toml | fuzz/Cargo.lock | deny.toml | \
      docs/release.md | docs/supply-chain.md | supply-chain/* | supply-chain/sbom/* | \
      scripts/*.sh | .github/workflows/*.yml)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

required_checks() {
  needs_docs=0
  needs_rust=0
  needs_security=0
  needs_supply_chain=0
  saw_file=0

  while IFS= read -r path; do
    [ -n "$path" ] || continue
    saw_file=1

    if triggers_docs_workflow "$path"; then
      needs_docs=1
    fi

    if ! is_rust_ci_ignored_path "$path"; then
      needs_rust=1
    fi

    if is_security_path "$path"; then
      needs_security=1
    fi

    if is_supply_chain_path "$path"; then
      needs_supply_chain=1
    fi
  done

  if [ "$saw_file" -eq 0 ]; then
    needs_rust=1
  fi

  echo "Secret Scan"
  if [ "$needs_docs" -eq 1 ]; then
    echo "Docs"
  fi
  if [ "$needs_rust" -eq 1 ]; then
    echo "Rust stable"
    echo "Rust 1.80.0"
  fi
  if [ "$needs_security" -eq 1 ]; then
    echo "cargo-security"
  fi
  if [ "$needs_supply_chain" -eq 1 ]; then
    echo "Supply Chain"
  fi
}

checks="$(changed_files | required_checks)"

if [ "$print_only" -eq 1 ]; then
  printf '%s\n' "$checks"
  exit 0
fi

if [ -z "${GITHUB_REPOSITORY:-}" ] || [ -z "${HEAD_SHA:-}" ]; then
  echo "GITHUB_REPOSITORY and HEAD_SHA are required to verify check runs" >&2
  exit 2
fi

if [ -z "$checks" ]; then
  echo "No required checks were selected"
  exit 0
fi

echo "Required checks for this PR:"
while IFS= read -r check_name; do
  [ -n "$check_name" ] || continue
  printf '  - %s\n' "$check_name"
done <<EOF
$checks
EOF

deadline=$(( $(date +%s) + timeout_seconds ))

while :; do
  all_complete=1

  while IFS= read -r check_name; do
    [ -n "$check_name" ] || continue

    line="$(
      gh api "/repos/$GITHUB_REPOSITORY/commits/$HEAD_SHA/check-runs?per_page=100" \
        --jq ".check_runs[] | select(.name == \"$check_name\") | [.status, (.conclusion // \"\")] | @tsv" \
        | tail -n 1 || true
    )"

    if [ -z "$line" ]; then
      echo "Waiting for $check_name to be created"
      all_complete=0
      continue
    fi

    status="$(printf '%s' "$line" | cut -f1)"
    conclusion="$(printf '%s' "$line" | cut -f2)"

    case "$status:$conclusion" in
      completed:success)
        echo "$check_name passed"
        ;;
      completed:*)
        echo "$check_name completed with conclusion '$conclusion'" >&2
        exit 1
        ;;
      *)
        echo "Waiting for $check_name: status=$status"
        all_complete=0
        ;;
    esac
  done <<EOF
$checks
EOF

  if [ "$all_complete" -eq 1 ]; then
    echo "Merge gate passed"
    exit 0
  fi

  if [ "$(date +%s)" -ge "$deadline" ]; then
    echo "Timed out waiting for required checks" >&2
    exit 1
  fi

  sleep "$poll_interval"
done
