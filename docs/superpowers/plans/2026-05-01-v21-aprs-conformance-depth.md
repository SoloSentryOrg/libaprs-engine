# v2.1.0 APRS Conformance Depth Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use
> superpowers:subagent-driven-development (recommended) or
> superpowers:executing-plans to implement this plan task-by-task. Steps use
> checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add byte-preserving APRS semantic coverage for the `v2.1.0`
conformance-depth release without breaking the `2.x` public API.

**Architecture:** Keep the codec envelope unchanged. Add semantic helper methods
and fixtures around existing `AprsData` variants, preserving raw bytes and
returning optional typed views only when bytes can be decoded conservatively.

**Tech Stack:** Rust 2021, standard library, existing workspace crates, Cargo
tests, release verification scripts, GitHub Project #3.

---

### Task 1: Compressed Weather-Symbol Positions

**Files:**
- Modify: `crates/libaprs-engine/src/lib.rs`
- Test: `crates/libaprs-engine/tests/codec.rs`
- Test: `crates/libaprs-engine/tests/api_compat.rs`
- Modify: `docs/api.md`
- Modify: `docs/conformance.md`
- Modify: `CHANGELOG.md`

- [x] **Step 1: Write failing tests**

Add tests that parse compressed positions using symbol code `_`, then assert
that `CompressedPosition::weather()` returns the exact comment bytes as a
`Weather` report. Add object and item tests where the body starts with a
compressed weather-symbol position.

- [x] **Step 2: Verify RED**

Run:

```bash
cargo test -p libaprs-engine --test codec compressed
```

Expected before implementation: compilation fails because
`CompressedPosition::weather()` is not defined.

- [x] **Step 3: Implement the helper**

Add `CompressedPosition::weather()` and share the object/item embedded-weather
path with compressed positions. Return `None` unless `symbol_code == b'_'` and
the compressed-position comment is non-empty.

- [x] **Step 4: Verify GREEN**

Run:

```bash
cargo test -p libaprs-engine --test codec compressed
cargo test -p libaprs-engine --test api_compat documented_semantic_helpers_remain_usable
```

Expected after implementation: all selected tests pass.

- [x] **Step 5: Update docs and support matrix**

Update the conformance matrix, API documentation, support-matrix notes, and
changelog so compressed weather extraction is no longer listed as unsupported.

### Task 2: Mic-E Altitude And Ambiguity

**Files:**
- Modify: `crates/libaprs-engine/src/lib.rs`
- Test: `crates/libaprs-engine/tests/codec.rs`
- Test: `crates/libaprs-engine/tests/conformance.rs`
- Modify: `docs/api.md`
- Modify: `docs/conformance.md`

- [ ] **Step 1: Add fixture-backed tests**

Add tests for Mic-E altitude and ambiguity bytes using publishable fixtures.
Tests must assert raw packet preservation, existing coordinate behavior, and
optional helper return values.

- [ ] **Step 2: Verify RED**

Run:

```bash
cargo test -p libaprs-engine --test codec mic_e
```

Expected before implementation: missing helper methods or missing decoded
values.

- [ ] **Step 3: Implement narrow helpers**

Add optional Mic-E helper methods that return typed altitude and ambiguity data
only when the body and destination bytes permit conservative decoding.

- [ ] **Step 4: Verify GREEN**

Run:

```bash
cargo test -p libaprs-engine --test codec mic_e
cargo test -p libaprs-engine --test conformance mic_e
```

Expected after implementation: Mic-E semantic and conformance tests pass.

### Task 3: APRS 1.2 Position Comment Extensions

**Files:**
- Modify: `crates/libaprs-engine/src/lib.rs`
- Test: `crates/libaprs-engine/tests/codec.rs`
- Modify: `docs/api.md`
- Modify: `docs/conformance.md`

- [ ] **Step 1: Add tests for additive comment helpers**

Add tests for `!DAO!`, PHG, range, altitude, and frequency comment extensions.
Tests must verify that comments remain available as original bytes even when
typed helper extraction succeeds.

- [ ] **Step 2: Verify RED**

Run:

```bash
cargo test -p libaprs-engine --test codec position_extension
```

Expected before implementation: helper methods are absent or return no typed
values.

- [ ] **Step 3: Implement conservative typed views**

Add helper structs and methods that scan existing comment bytes and return
typed values without removing or normalizing the original comment.

- [ ] **Step 4: Verify GREEN**

Run:

```bash
cargo test -p libaprs-engine --test codec position_extension
cargo test -p libaprs-engine --test api_compat documented_semantic_helpers_remain_usable
```

Expected after implementation: extension helper tests and API compatibility
tests pass.

### Task 4: Release Readiness Sweep

**Files:**
- Modify: `docs/conformance.md`
- Modify: `docs/api.md`
- Modify: `docs/public-api.md`
- Modify: `docs/release.md`
- Modify: `CHANGELOG.md`

- [ ] **Step 1: Verify documentation consistency**

Run:

```bash
scripts/verify-docs.sh
rg -n "unsupported|partial|v2.1.0|v3.0.0" ROADMAP.md docs CHANGELOG.md
```

Expected: no stale unsupported statements for completed `v2.1.0` features.

- [ ] **Step 2: Run local release gate**

Run:

```bash
env -u CARGO_HOME scripts/verify-release.sh
```

Expected: release verification passes, with only documented optional skips for
benchmarks or post-publication checks when not publishing.

- [ ] **Step 3: Secure review before merge**

Review the diff for raw-byte preservation, fail-closed behavior, untrusted input
handling, panic risk, dependency changes, and documentation accuracy. Fix all
findings before merging to `main`.
