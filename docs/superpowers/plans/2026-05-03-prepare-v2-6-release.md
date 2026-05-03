# Prepare v2.6.0 Release Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Prepare the repository for the `v2.6.0` release without publishing crates or tagging before PR gates pass.

**Architecture:** Keep release prep limited to version metadata, package docs, changelog, release notes, and verification evidence. Use the existing guarded release scripts for validation and leave publication to the clean release commit after remote CI and secure review pass.

**Tech Stack:** Cargo workspace manifests, Markdown release docs, shell verification scripts, GitHub PR workflow.

---

### Task 1: Version Metadata

**Files:**
- Modify: `crates/*/Cargo.toml`
- Modify: `Cargo.lock`
- Modify: `fuzz/Cargo.lock`
- Modify: `examples/downstream-smoke/Cargo.toml`
- Modify: `examples/downstream-smoke/src/main.rs`
- Modify: `docs/publishing.md`
- Modify: `docs/api.md`
- Modify: `docs/transports.md`
- Modify: `README.md`

- [x] **Step 1: Bump workspace crate versions**

Change every workspace crate `version = "2.5.0"` to `version = "2.6.0"`.

- [x] **Step 2: Bump workspace internal dependency requirements**

Change each workspace `libaprs-engine = { version = "2.5.0", path = "../libaprs-engine" }` requirement to `2.6.0`.

- [x] **Step 3: Bump public dependency examples**

Update current-version examples in README, API, transport, publishing, APRS-IS profile, and downstream-smoke docs/examples from `2.5.0` to `2.6.0`, leaving historical release evidence unchanged.

- [x] **Step 4: Refresh lockfile metadata**

Run:

```sh
cargo metadata --no-deps --format-version 1 >/dev/null
```

Expected: command exits `0` and `Cargo.lock` reflects `2.6.0` path crate versions if Cargo updates it.

### Task 2: Release Documentation

**Files:**
- Modify: `CHANGELOG.md`
- Modify: `docs/release-notes-v2.6.0.md`
- Modify: `ROADMAP.md`

- [x] **Step 1: Add changelog entry**

Add `## 2.6.0 - 2026-05-03` under `Unreleased` with bullets for internal evidence handling, docs gates, and abuse-resistance tests.

- [x] **Step 2: Finalize release notes**

Change the release notes from planned language to release language and include operator impact plus security notes.

- [x] **Step 3: Keep roadmap pre-publication status honest**

Leave `v2.6.0` as not released until crates, tag, GitHub Release, release evidence, and project updates are complete.

### Task 3: Verification And Secure Review

**Files:**
- Review the full diff.

- [x] **Step 1: Run focused verification**

Run:

```sh
scripts/verify-docs.sh
cargo metadata --no-deps --format-version 1
cargo test -p libaprs-engine --test engine
cargo test -p libaprs-engine --test codec
cargo test --examples
```

Expected: all commands exit `0`.

- [x] **Step 2: Run release gate**

Run:

```sh
scripts/verify-release.sh
```

Expected: exits `0`. Downstream smoke, package-all, and benchmark gates may be skipped unless their release-publication environment variables are set.

- [x] **Step 3: Secure review**

Review for raw-byte preservation, fail-closed behavior, version consistency, accidental publication of internal evidence, and release-script bypasses.

### Task 4: PR

**Files:**
- Stage only intended release-prep files.

- [x] **Step 1: Commit**

Commit message:

```sh
Prepare v2.6.0 release
```

- [x] **Step 2: Push and open PR**

Push `codex/prepare-v2-6-release` and open a PR targeting `main` with release-prep scope and verification evidence.
