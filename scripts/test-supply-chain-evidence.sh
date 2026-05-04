#!/usr/bin/env sh
set -eu

fail() {
  echo "supply-chain evidence test failed: $*" >&2
  exit 1
}

run_check() {
  scripts/generate-supply-chain-evidence.sh --check
}

run_check

[ -f supply-chain/SHA256SUMS ] || fail "missing supply-chain/SHA256SUMS"
[ -d supply-chain/sbom ] || fail "missing supply-chain/sbom directory"

grep -q '  Cargo.lock$' supply-chain/SHA256SUMS ||
  fail "Cargo.lock is not covered by SHA256SUMS"

grep -q '  supply-chain/sbom/libaprs-engine.cdx.json$' supply-chain/SHA256SUMS ||
  fail "libaprs-engine SBOM is not covered by SHA256SUMS"

if grep -Eq '  (crates|fuzz|examples)/.*\.rs$' supply-chain/SHA256SUMS; then
  fail "Rust source files must be identified by Git commit, not duplicated in SHA256SUMS"
fi

if find crates -type f -name '.*.supply-chain.*.json' | grep -q .; then
  fail "generator left temporary SBOM files under crates/"
fi

if find crates -mindepth 2 -maxdepth 2 -type f \
  \( -name '*.cdx.json' -o -name 'sbom.json' -o -name '*-sbom.json' -o -name 'test-sbom.json' \) \
  | grep -q .; then
  fail "SBOM files must only be written under supply-chain/sbom/"
fi

echo "supply-chain evidence tests passed"
