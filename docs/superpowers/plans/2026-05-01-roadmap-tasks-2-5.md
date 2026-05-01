# Roadmap Tasks 2-5 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use
> superpowers:subagent-driven-development (recommended) or
> superpowers:executing-plans to implement this plan task-by-task. Steps use
> checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the next four roadmap release-batch slices for interoperability,
encoding, production service helpers, and assurance without breaking the `2.x`
API.

**Architecture:** Keep packet parsing byte-preserving and fail-closed. Add
interoperability helpers in the APRS-IS adapter crate, add owned-byte encoder
helpers and service-policy utilities in the core crate, and record assurance
evidence in documentation and tests.

**Tech Stack:** Rust 2021, standard library, existing workspace crates, Cargo
tests, GitHub Project #3, `scripts/verify-docs.sh`, and local release gates.

---

### Task 1: v2.2.0 Interoperability Profiles

**Files:**
- Modify: `crates/aprs-transport-aprs-is/src/lib.rs`
- Test: `crates/aprs-transport-aprs-is/tests/aprs_is_transport.rs`
- Modify: `docs/transports.md`
- Modify: `docs/api.md`
- Modify: `CHANGELOG.md`

- [x] **Step 1: Write failing tests**

Add tests for uppercase APRS-IS login validation, APRS-IS filter validation,
and q-construct diagnostics over raw TNC2 packet bytes.

- [x] **Step 2: Verify RED**

Run:

```bash
cargo test -p aprs-transport-aprs-is profile
```

Expected before implementation: missing APRS-IS profile helper APIs.

- [x] **Step 3: Implement helpers**

Add `AprsIsProfileError`, `AprsIsFilter`, `AprsIsQConstruct`,
`AprsIsQConstructKind`, `AprsIsLogin::profile_line()`,
`validate_aprs_is_callsign()`, `validate_aprs_is_filter()`, and
`q_construct_from_tnc2()`.

- [x] **Step 4: Verify GREEN**

Run:

```bash
cargo test -p aprs-transport-aprs-is profile
```

Expected after implementation: profile tests pass.

### Task 2: v2.3.0 Encoding And Packet Construction

**Files:**
- Create: `crates/libaprs-engine/src/encoder.rs`
- Modify: `crates/libaprs-engine/src/lib.rs`
- Test: `crates/libaprs-engine/tests/api_compat.rs`
- Test: `crates/libaprs-engine/tests/codec.rs`
- Modify: `docs/api.md`
- Modify: `docs/public-api.md`
- Modify: `CHANGELOG.md`

- [x] **Step 1: Write failing tests**

Add tests for status, uncompressed position, message, telemetry, object, and
item encoders. Each test must round-trip through `parse_packet()` and assert
the emitted bytes exactly.

- [x] **Step 2: Verify RED**

Run:

```bash
cargo test -p libaprs-engine encoder
```

Expected before implementation: missing encoder APIs.

- [x] **Step 3: Implement encoders**

Add owned-byte encoder functions that validate callsigns, path components,
payload length, coordinates, message addressee width, telemetry value ranges,
object timestamps, and item names before emitting bytes.

- [x] **Step 4: Verify GREEN**

Run:

```bash
cargo test -p libaprs-engine encoder
```

Expected after implementation: encoder tests pass.

### Task 3: v2.4.0 Production Service Toolkit

**Files:**
- Create: `crates/libaprs-engine/src/service.rs`
- Modify: `crates/libaprs-engine/src/lib.rs`
- Test: `crates/libaprs-engine/tests/api_compat.rs`
- Test: `crates/libaprs-engine/tests/engine.rs`
- Modify: `docs/operations.md`
- Modify: `docs/examples.md`
- Modify: `CHANGELOG.md`

- [x] **Step 1: Write failing tests**

Add tests for bounded duplicate suppression, explicit packet-rate budgets, and
semantic-family blocklist helpers.

- [x] **Step 2: Verify RED**

Run:

```bash
cargo test -p libaprs-engine service
```

Expected before implementation: missing service helper APIs.

- [x] **Step 3: Implement service helpers**

Add `DuplicateWindow`, `DuplicateDecision`, `PacketRateBudget`,
`RateLimitDecision`, `SemanticFamily`, and `SemanticBlocklist` without adding
storage, networking, async, or time dependencies.

- [x] **Step 4: Verify GREEN**

Run:

```bash
cargo test -p libaprs-engine service
```

Expected after implementation: service-helper tests pass.

### Task 4: v2.5.0 Assurance And API Readiness

**Files:**
- Create: `docs/api-guidelines-audit.md`
- Create: `docs/supply-chain.md`
- Create: `docs/v3-breaking-changes.md`
- Modify: `docs/downstream-feedback.md`
- Modify: `docs/conformance.md`
- Modify: `docs/security.md`
- Modify: `docs/verification.md`
- Modify: `README.md`
- Modify: `CHANGELOG.md`

- [x] **Step 1: Write verification coverage**

Add documentation and API-compat checks proving the new public surfaces are
documented, additive, byte-preserving, and release-gated.

- [x] **Step 2: Verify docs RED/GREEN path**

Run:

```bash
scripts/verify-docs.sh
```

Expected after implementation: docs verification passes and new docs are linked
from README.

- [x] **Step 3: Run full verification and secure review**

Run:

```bash
cargo test
cargo clippy --all-targets --all-features -- -D warnings
scripts/verify-docs.sh
git diff --check
```

Expected: all checks pass before commit, push, and PR.
