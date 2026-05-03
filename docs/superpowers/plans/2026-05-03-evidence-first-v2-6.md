# Evidence-First v2.6.0 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete the additive `v2.6.0` evidence-first readiness track before any `v3.0.0` release-candidate work.

**Architecture:** Keep the core parser unchanged and add evidence through docs gates, roadmap state, and focused regression tests. Treat downstream-feedback.md as an internal evidence source and fail verification if it is exposed through public navigation or public issue templates.

**Tech Stack:** Rust workspace, shell verification scripts, Markdown release and roadmap docs, existing Cargo tests.

---

### Task 1: Internal Evidence Boundary

**Files:**
- Create: `scripts/check-internal-docs.sh`
- Modify: `scripts/verify-docs.sh`
- Modify: `README.md`
- Modify: `docs/stability.md`
- Modify: `docs/downstream-feedback.md`
- Delete: `.github/ISSUE_TEMPLATE/downstream_feedback.md`

- [x] **Step 1: Write the failing guard**

```sh
sh scripts/check-internal-docs.sh
```

Expected: FAIL while `README.md` links to `docs/downstream-feedback.md` and the public downstream-feedback issue template exists.

- [x] **Step 2: Remove public exposure**

Remove the README documentation index link, replace public stability-doc links with an internal evidence-log reference, delete the public issue template, and mark `docs/downstream-feedback.md` as internal.

- [x] **Step 3: Verify the guard passes**

```sh
sh scripts/check-internal-docs.sh
```

Expected: PASS with `internal docs verification passed`.

### Task 2: v2.6 Evidence Gate

**Files:**
- Create: `scripts/check-v2-6-evidence.sh`
- Modify: `scripts/verify-docs.sh`
- Modify: `ROADMAP.md`
- Create: `docs/release-notes-v2.6.0.md`

- [x] **Step 1: Write the failing evidence check**

```sh
sh scripts/check-v2-6-evidence.sh
```

Expected: FAIL until the roadmap milestone, release notes, and internal evidence marker exist.

- [x] **Step 2: Add evidence artifacts**

Add the `v2.6.0` roadmap milestone, unreleased release notes, and docs verification wiring.

- [x] **Step 3: Verify docs gates**

```sh
scripts/verify-docs.sh
```

Expected: PASS.

### Task 3: Abuse-Resistance Evidence Tests

**Files:**
- Modify: `crates/libaprs-engine/tests/engine.rs`
- Modify: `crates/libaprs-engine/tests/codec.rs`

- [x] **Step 1: Add regression tests for untrusted input**

Add focused tests for invalid UTF-8 raw-byte preservation, oversized transport boundaries, malformed semantic floods, and nested third-party packets.

- [x] **Step 2: Run focused tests**

```sh
cargo test -p libaprs-engine --test engine
cargo test -p libaprs-engine --test codec
```

Expected: PASS.

### Task 4: Secure Review And PR

**Files:**
- Review the full diff.

- [ ] **Step 1: Run local gates**

```sh
scripts/verify-docs.sh
cargo test -p libaprs-engine --test engine
cargo test -p libaprs-engine --test codec
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

Expected: PASS.

- [ ] **Step 2: Secure code review**

Review the diff for OWASP-aligned input handling, raw-byte preservation, fail-closed behavior, public exposure of internal evidence, and release-gate bypasses.

- [ ] **Step 3: Publish draft PR**

Stage only intended files, commit, push `codex/evidence-first-v2-6`, and open a draft PR targeting `main`.
