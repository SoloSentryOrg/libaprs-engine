# Supply Chain Evidence Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add tracked SBOM and SHA-256 evidence for main-branch supply-chain control.

**Architecture:** Generate deterministic per-crate CycloneDX SBOM files and a
tracked `SHA256SUMS` manifest from one local script. CI and the release gate
run the same guard, while Rust source identity remains the Git commit and tag.

**Tech Stack:** POSIX shell, cargo-cyclonedx 0.5.9, Python 3 JSON
normalization, GitHub Actions.

---

### Task 1: Evidence Guard

**Files:**
- Create: `scripts/test-supply-chain-evidence.sh`
- Create: `scripts/generate-supply-chain-evidence.sh`

- [x] **Step 1: Write the failing guard**

```bash
scripts/test-supply-chain-evidence.sh
```

Expected: FAIL until the generator and tracked evidence exist.

- [x] **Step 2: Implement deterministic evidence generation**

Generate `supply-chain/sbom/*.cdx.json` with `SOURCE_DATE_EPOCH=0`,
`--target all`, `--all-features`, and normalized repository-relative paths.

- [x] **Step 3: Verify the guard passes**

```bash
scripts/test-supply-chain-evidence.sh
```

Expected: PASS.

### Task 2: Release And CI Gates

**Files:**
- Create: `.github/workflows/supply-chain.yml`
- Modify: `scripts/verify-release.sh`
- Modify: `scripts/check-merge-gate.sh`
- Modify: `scripts/test-merge-gate-guard.sh`
- Modify: `scripts/check-workflow-optimizations.sh`

- [x] **Step 1: Wire local release verification**

Run `scripts/test-supply-chain-evidence.sh` from `scripts/verify-release.sh`.

- [x] **Step 2: Add remote supply-chain verification**

Run a read-only `Supply Chain` workflow that installs pinned
`cargo-cyclonedx` and checks tracked evidence.

- [x] **Step 3: Require the check through the merge gate**

Classify dependency, workflow, script, policy, and tracked evidence changes as
requiring `Supply Chain`.

### Task 3: Documentation

**Files:**
- Modify: `docs/supply-chain.md`
- Modify: `docs/release.md`
- Create: `supply-chain/README.md`

- [x] **Step 1: Document tracked evidence**

Describe per-crate SBOMs and the SHA-256 manifest.

- [x] **Step 2: Document source integrity**

State that Rust source files are controlled by Git commit and tag, not per-file
hashes in `SHA256SUMS`.

- [x] **Step 3: Document release artifact hashes**

Require SHA-256 hashes for distributed `.crate`, source archive, binary, and
SBOM release assets.
