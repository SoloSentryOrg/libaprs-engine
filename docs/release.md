# Release Checklist

## Local Gate

- Run `scripts/verify-release.sh`.
- Run `scripts/test-publish-release-guards.sh`.
- Complete a secure code review and fix all findings before publishing.
- After `libaprs-engine` is published to crates.io, run package validation for
  dependent crates with `LIBAPRS_PACKAGE_ALL=1 scripts/verify-release.sh`.
- Run `cargo bench -p libaprs-engine` when parser performance changed.
- Verify the declared MSRV through the release script. It runs Rust `1.80.0`
  checks when that toolchain is installed.
- Run `cargo audit` and `cargo deny check`, either locally or through the
  security workflow, before publishing.
- Treat the Rust CI release-script job as a release dependency gate: it installs
  pinned `cargo-audit` and `cargo-deny` versions and runs both checks through
  `scripts/verify-release.sh`.
- Confirm `CHANGELOG.md` describes the release.
- Review `docs/public-api.md` and `crates/libaprs-engine/tests/api_compat.rs`
  when the release changes exported library APIs.
- Use `scripts/publish-release.sh` when publishing to crates.io; it encodes the
  crate publish order from `docs/publishing.md` and refuses to publish without
  explicit clean secure-review and gate evidence.
- In restricted environments where `~/.cargo` is not writable, use
  `CARGO_HOME=/tmp/libaprs-cargo-home` for audit, deny, semver, package, and
  publish commands. Copy crates.io credentials into that temporary Cargo home
  only at runtime when publishing.

## Remote Gate

- Confirm GitHub Actions is enabled before relying on remote CI.
- If GitHub Actions is blocked before job creation, document that CI was skipped
  and rely on the local gate.
- Do not publish while a required remote CI or security workflow is failing.

## Tagging

- Tag the release after local and remote verification pass.
- If remote CI is intentionally skipped, tag only after the local gate passes
  and the skipped remote gate is documented.

## Pre-Publish Evidence

Before running `scripts/publish-release.sh`, record or verify:

- `LIBAPRS_SECURE_REVIEW=clean`: secure code review completed with no open
  findings.
- `LIBAPRS_LOCAL_RELEASE_GATE=passed`: `scripts/verify-release.sh` passed for
  the release commit.
- `LIBAPRS_SECURITY_GATE=passed`: `cargo audit` and `cargo deny check` passed
  locally or in GitHub Actions.
- `LIBAPRS_REMOTE_CI=passed`: remote CI passed for the release commit. Use
  `skipped-documented` only when CI was unavailable and the release evidence
  records the reason.
- `LIBAPRS_RELEASE_COMMIT="$(git rev-parse HEAD)"`: the publish target matches
  the checked-out commit.

## v1.0.0-rc.1 Release Candidate Evidence

- Tag: `v1.0.0-rc.1` pending after release-candidate PR merge and clean gates.
- Commit: pending release-candidate PR merge.
- Local gate: pending `scripts/verify-release.sh` for `1.0.0-rc.1`.
- Remote GitHub Actions: pending release-candidate PR CI and security workflow.
- Downstream smoke: pending crates.io release-candidate publication.
- Notes: first `1.0.0` release candidate after API stabilization, APRS
  semantics completion, security and robustness hardening, transport
  reliability hardening, and CI release-gate improvements. Do not promote to
  `1.0.0` until the release-candidate commit has clean secure review, local
  gate, security gate, remote CI, and downstream smoke evidence.

## v0.6.0 Release Evidence

- Tag: `v0.6.0`
- Commit: release tag target (`git rev-list -n 1 v0.6.0` after tagging).
- Local gate: release verification for `0.6.0`.
- Remote GitHub Actions: release verification for `0.6.0` after push.
- Notes: adds object/item coordinate helpers, NMEA sentence field helpers,
  Mic-E message-code helpers, stronger parser assurance tests, transport
  cookbook examples, and contributor workflow templates.

## v0.5.0 Release Evidence

- Tag: `v0.5.0`
- Commit: release tag target (`git rev-list -n 1 v0.5.0` after tagging).
- Local gate: release verification for `0.5.0`.
- Remote GitHub Actions: release verification for `0.5.0` after push.
- Notes: adds opt-in policy rejection for NMEA checksum mismatches while
  preserving codec raw-byte behavior and checksum reporting.

## v0.4.0 Release Evidence

- Tag: `v0.4.0`
- Commit: release tag target (`git rev-list -n 1 v0.4.0` after tagging).
- Local gate: release verification for `0.4.0`.
- Remote GitHub Actions: release verification for `0.4.0` after push.
- Notes: expands APRS weather semantic extraction for luminosity, snow, raw
  rain counters, and signed temperatures.

## v0.3.0 Release Evidence

- Tag: `v0.3.0`
- Commit: release tag target (`git rev-list -n 1 v0.3.0` after tagging).
- Local gate: release verification for `0.3.0`.
- Remote GitHub Actions: release verification for `0.3.0` after push.
- Notes: expands APRS101 conformance fixtures with source references and
  byte-preservation checks.

## v0.2.0 Release Evidence

- Tag: `v0.2.0`
- Commit: release tag target (`git rev-list -n 1 v0.2.0` after tagging).
- Local gate: release verification for `0.2.0`.
- Remote GitHub Actions: release verification for `0.2.0` after push.
- Notes: adds shared transport contracts, bounded transport I/O hardening,
  engine source processing, CLI subcommands, fuzz scaffolding, semver/fuzz
  release gates, and benchmark threshold support.

## v0.1.5 Release Evidence

- Tag: `v0.1.5`
- Commit: release tag target (`git rev-list -n 1 v0.1.5`).
- Local gate: release verification for `0.1.5`.
- Remote GitHub Actions: release verification for `0.1.5`.
- Notes: hardens UDP datagram ingestion so oversized datagrams fail closed.

## v0.1.4 Release Evidence

- Tag: not created.
- Commit: not retained as a repository release point.
- Local gate: package dry-runs and local verification were performed before
  crates.io publication.
- Remote GitHub Actions: superseded before tagging by `v0.1.5`.
- Notes: adds KISS, serial, UDP, HTTP, file-watch, MQTT, AX.25, corpus,
  channel, and async transport helper crates. Superseded immediately by
  `v0.1.5` after secure review tightened UDP oversized-datagram handling.

## v0.1.3 Release Evidence

- Tag: `v0.1.3`
- Commit: release tag target (`git rev-list -n 1 v0.1.3`).
- Local gate: release verification for `0.1.3`.
- Remote GitHub Actions: release verification for `0.1.3`.
- Notes: adds APRS-IS transport helpers, structured packet summaries, CLI
  operator controls, downstream smoke checks, security workflow, and
  documentation/readiness updates.

## v0.1.2 Release Evidence

- Tag: `v0.1.2`
- Commit: release tag target (`git rev-list -n 1 v0.1.2`).
- Local gate: passed on 2026-04-25.
- Remote GitHub Actions: passed on 2026-04-25 before tag creation.
- Notes: expands APRS semantics with telemetry metadata, NMEA checksum
  inspection, Mic-E coordinate/speed/course helpers, and explicit nested
  third-party parsing.

## v0.1.1 Release Evidence

- Tag: `v0.1.1`
- Commit: release tag target (`git rev-list -n 1 v0.1.1`).
- Local gate: passed on 2026-04-25.
- Remote GitHub Actions: passed on 2026-04-25:
  <https://github.com/elodiejmirza/libaprs-engine/actions/runs/24937390267>.
- Notes: supersedes `v0.1.0` with MSRV clippy compatibility and Node.js
  24-compatible checkout action. Superseded by `v0.1.2`, `v0.1.3`,
  `v0.1.4`, and `v0.1.5`.

## v0.1.0 Release Evidence

- Tag: `v0.1.0`
- Commit: `043630419453548cfdc026410788d89ba00386b7`
- Local gate passed.
- Remote GitHub Actions was skipped because startup is blocked before job
  creation by account, billing, or policy state outside this repository.
- Superseded by `v0.1.1`, `v0.1.2`, `v0.1.3`, `v0.1.4`, and `v0.1.5`.
