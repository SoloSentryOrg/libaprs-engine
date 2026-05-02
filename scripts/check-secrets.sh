#!/usr/bin/env sh
set -eu

if ! command -v gitleaks >/dev/null 2>&1; then
  echo "gitleaks is not installed; install with: brew install gitleaks" >&2
  exit 1
fi

gitleaks detect --redact --source .
