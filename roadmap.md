# Roadmap To 1.0.0

## BLUF

- The fastest credible path to `1.0.0` is to lock the public API, parser behavior, security posture, documentation, and release gates before expanding the project further.
- Work is batched so each batch can be completed, reviewed, tested, and committed independently.
- Release candidates and final releases must not be published until secure review, local gates, security gates, and remote CI evidence pass for the exact release commit.
- The project remains protocol-first: preserve raw bytes, fail closed on malformed packets, and avoid lossy parsing at every boundary.
- The priority order is API stabilization, APRS semantics, security and robustness, transport reliability, then release candidate hardening.

## 1. API Stabilization Batch

Status: completed for the current `1.0.0` roadmap pass. The candidate public
API boundary is documented in `docs/public-api.md`, and compatibility coverage
is enforced by `crates/libaprs-engine/tests/api_compat.rs`.

- Freeze the public API intended for `1.0.0`:
  - `RawPacket`
  - `ParsedPacket`
  - `ParseError`
  - parser entry points
  - policy APIs
  - transport APIs
- Add explicit semver guidance to the documentation.
- Review all exported types for naming, ownership, lifetimes, error behavior, and extensibility.
- Decide what remains internal before `1.0.0`.
- Add compile-time or public API compatibility tests where useful.

Target outcome: downstream users can build against the API without expecting breaking changes.

## 2. APRS Semantics Completion Batch

Status: completed for the current `1.0.0` roadmap pass. High-value APRS
families are represented, raw bytes are preserved, and malformed semantic
payloads are covered by APRS101 malformed golden fixtures plus strict-policy
rejection tests.

- Complete high-value APRS packet support:
  - position reports
  - messages
  - objects and items
  - status packets
  - telemetry
  - weather
  - Mic-E, if not already complete
- Add strict malformed-packet rejection behavior for each semantic type.
- Preserve raw bytes across every parse path.
- Add golden fixtures for valid and invalid packets.
- Document supported and unsupported APRS features clearly.

Target outcome: the crate is useful as an APRS parser, not just as a hardened parsing boundary.

## 3. Security And Robustness Batch

Status: completed for the current `1.0.0` roadmap pass. Parser robustness
coverage includes malformed corpora, boundary lengths, invalid UTF-8 handling,
deterministic mutation inputs, panic-free fuzz-adjacent assertions, saturating
engine counters, crate-level unsafe bans, and CI dependency-policy gates.

- Expand secure parser testing:
  - malformed corpus tests
  - boundary-length tests
  - invalid UTF-8 tests
  - randomized and fuzz-adjacent tests
  - panic-free assertions
- Confirm all counters and metrics use saturating or checked behavior.
- Keep `#![forbid(unsafe_code)]` across crates.
- Add dependency policy checks to release documentation and CI gates.
- Perform and document a clean secure code review before any release candidate.

Target outcome: the project has defensible OWASP-aligned handling of untrusted packet input.

## 4. Transport Reliability Batch

- Harden existing transports:
  - line transport
  - file and stdin CLI ingestion
  - TCP, KISS, and TNC paths where present
- Add integration tests for byte-preserving transport boundaries.
- Ensure transports never force UTF-8 before parsing.
- Add timeout, frame-size, and backpressure rules where applicable.
- Document transport trust boundaries separately from codec semantics.

Target outcome: users can ingest APRS data without weakening the parser security guarantees.

## 5. Release Candidate Batch

- Cut `1.0.0-rc.1`.
- Run the full local release gate.
- Run remote CI and security workflows.
- Run downstream smoke tests against the release candidate.
- Fix any findings before final release.
- After clean review and passing evidence, tag and publish `1.0.0`.

Target outcome: final `1.0.0` is a promotion of a tested release candidate, not a fresh unproven build.

## Release Rule

- Do not publish `1.0.0`, or any release candidate, until:
  - secure code review is clean
  - local release gate passes
  - security gate passes
  - remote CI passes or has a documented approved skip
  - the release commit exactly matches the reviewed commit
