#!/usr/bin/env sh
set -eu

workflow=".github/workflows/rust-ci.yml"
docs_workflow=".github/workflows/docs.yml"
merge_gate_workflow=".github/workflows/merge-gate.yml"
secret_scan_workflow=".github/workflows/secret-scan.yml"
supply_chain_workflow=".github/workflows/supply-chain.yml"
factory_rust_builder_workflow=".github/workflows/factory-rust-builder-ubuntu-validation.yml"
dependabot_config=".github/dependabot.yml"
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

reject_text_in_file() {
  file="$1"
  pattern="$2"
  description="$3"

  if [ -f "$file" ] && grep -F "$pattern" "$file" >/dev/null 2>&1; then
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
reject_text_in_file "$merge_gate_workflow" "pull_request:" "merge-gate workflow should not run untrusted PR-head workflow definitions"
require_text_in_file "$merge_gate_workflow" "pull_request_target:" "merge-gate workflow should run from trusted base context"
require_text_in_file "$merge_gate_workflow" "ref: \${{ github.event.repository.default_branch }}" "merge-gate checkout should use the trusted default branch"
require_text_in_file "$merge_gate_workflow" "persist-credentials: false" "merge-gate checkout should not persist credentials"
require_text_in_file "$merge_gate_workflow" "scripts/check-merge-gate.sh" "merge-gate workflow should run the merge-gate verifier"
require_file "$secret_scan_workflow" "secret-scan workflow should exist for repository-wide secret scanning"
require_text_in_file "$secret_scan_workflow" "name: Secret Scan" "secret-scan workflow should have a stable name"
require_text_in_file "$secret_scan_workflow" "fetch-depth: 0" "secret-scan workflow should scan full git history"
require_text_in_file "$secret_scan_workflow" "scripts/install-release-tools.sh gitleaks" "secret-scan workflow should install pinned Gitleaks"
require_text_in_file "$secret_scan_workflow" "scripts/check-secrets.sh" "secret-scan workflow should run the local Gitleaks wrapper"
require_file "$supply_chain_workflow" "supply-chain workflow should exist for SBOM and hash drift checks"
require_text_in_file "$supply_chain_workflow" "name: Supply Chain" "supply-chain workflow should have a stable required-check name"
require_text_in_file "$supply_chain_workflow" "permissions:" "supply-chain workflow should declare explicit permissions"
require_text_in_file "$supply_chain_workflow" "contents: read" "supply-chain workflow should use read-only contents permissions"
require_text_in_file "$supply_chain_workflow" "cargo install cargo-cyclonedx --version 0.5.9 --locked" "supply-chain workflow should install a pinned SBOM tool"
require_text_in_file "$supply_chain_workflow" "scripts/test-supply-chain-evidence.sh" "supply-chain workflow should run the local evidence guard"
require_text_in_file "$supply_chain_workflow" '"scripts/*.sh"' "supply-chain workflow should run when hashed scripts change"
require_text_in_file "$supply_chain_workflow" '".github/workflows/*.yml"' "supply-chain workflow should run when hashed workflows change"
require_text_in_file "$supply_chain_workflow" '".github/workflows/*.yaml"' "supply-chain workflow should run when YAML workflows change"
require_text_in_file "$supply_chain_workflow" '".github/dependabot.yml"' "supply-chain workflow should run when Dependabot config changes"
reject_text_in_file "$supply_chain_workflow" "            target" "supply-chain workflow should not cache target build artifacts"
require_file "$factory_rust_builder_workflow" "factory rust-builder Ubuntu validation workflow should exist"
require_text_in_file "$factory_rust_builder_workflow" "name: Factory Rust Builder Ubuntu Validation" "factory rust-builder workflow should have a stable name"
require_text_in_file "$factory_rust_builder_workflow" "contents: read" "factory rust-builder workflow should use read-only contents permissions"
require_text_in_file "$factory_rust_builder_workflow" "packages: read" "factory rust-builder workflow should use read-only package permissions"
require_text_in_file "$factory_rust_builder_workflow" "ghcr.io/solosentryorg/rust-builder-ubuntu@sha256:" "factory rust-builder workflow should consume an immutable factory digest"
require_text_in_file "$factory_rust_builder_workflow" "docker manifest inspect" "factory rust-builder workflow should preflight GHCR digest readability"
require_text_in_file "$factory_rust_builder_workflow" "Factory image digest not readable" "factory rust-builder workflow should report clear GHCR access failures"
reject_text_in_file "$factory_rust_builder_workflow" "docker build" "factory rust-builder workflow should not build consumer-side images"
reject_text_in_file "$factory_rust_builder_workflow" "docker push" "factory rust-builder workflow should not publish consumer-side images"
reject_text_in_file "$factory_rust_builder_workflow" "UBUNTU_PRO_TOKEN" "factory rust-builder workflow should not consume Ubuntu Pro secrets"
require_text_in_file "$factory_rust_builder_workflow" "x86_64-unknown-linux-musl" "factory rust-builder workflow should verify musl target support"
require_text_in_file "$factory_rust_builder_workflow" "cargo clippy --all-targets --all-features -- -D warnings" "factory rust-builder workflow should verify clippy compatibility"
require_text_in_file "$factory_rust_builder_workflow" "cargo fmt --all --check" "factory rust-builder workflow should verify workspace formatting"
require_text_in_file "scripts/check-merge-gate.sh" "is_factory_rust_builder_path()" "merge-gate selector should classify factory rust-builder validation paths"
require_text_in_file "scripts/check-merge-gate.sh" "Validate rust-builder-ubuntu consumer compatibility" "merge-gate selector should require the factory rust-builder validation check"
require_text_in_file "scripts/test-merge-gate-guard.sh" "factory Rust builder workflow changes require the factory validation check" "merge-gate guard should test the factory rust-builder validation selector"
require_text_in_file "scripts/install-release-tools.sh" "gitleaks)" "release-tool installer should support pinned Gitleaks"
require_file "$dependabot_config" "Dependabot config should exist for scheduled dependency maintenance"
require_text_in_file "$dependabot_config" 'package-ecosystem: "cargo"' "Dependabot should track Cargo dependencies"
require_text_in_file "$dependabot_config" 'package-ecosystem: "github-actions"' "Dependabot should track GitHub Actions"
require_text_in_file "$dependabot_config" 'interval: "weekly"' "Dependabot updates should be scheduled weekly"

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

release_script_block="$(
  awk '
    /^  release-script:/ { in_release_job = 1 }
    in_release_job && /^  [A-Za-z0-9_-]+:/ && $0 !~ /^  release-script:/ { in_release_job = 0 }
    in_release_job { print }
  ' "$workflow"
)"
release_script_order_issue="$(
  printf '%s\n' "$release_script_block" |
    awk '
      /cargo install cargo-cyclonedx --version 0.5.9 --locked/ { install_line = NR }
      /scripts\/verify-release\.sh/ { verify_line = NR }
      END {
        if (!install_line) {
          print "missing_install"
        } else if (!verify_line) {
          print "missing_verify"
        } else if (install_line > verify_line) {
          print "install_after_verify"
        }
      }
    '
)"
case "$release_script_order_issue" in
  "")
    ;;
  missing_verify)
    note_failure "release-script should run verify-release"
    ;;
  *)
    note_failure "release-script should install pinned SBOM tool before verify-release"
    ;;
esac

stable_only_count="$(grep -F -c "matrix.toolchain == 'stable'" "$workflow" || true)"
if [ "$stable_only_count" -lt 5 ]; then
  note_failure "stable-only checks should avoid duplicating docs/package/default tests on MSRV"
fi

if [ "$failures" -ne 0 ]; then
  exit 1
fi

echo "workflow optimization checks passed"
