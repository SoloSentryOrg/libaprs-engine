# Release Checklist

## Local Gate

- Run `scripts/verify-release.sh`.
- Run `scripts/test-publish-release-guards.sh`.
- Run `scripts/test-fuzz-corpus-guard.sh` and `scripts/check-fuzz-corpus.sh`
  when fuzz corpus entries or fuzz targets changed.
- Run `scripts/check-workflow-optimizations.sh` and `scripts/verify-docs.sh`
  when workflow, release-gate, or documentation behavior changes.
- Complete a secure code review and fix all findings before publishing.
- After `libaprs-engine` is published to crates.io, run package validation for
  dependent crates with `LIBAPRS_PACKAGE_ALL=1 scripts/verify-release.sh`.
- Run `cargo bench -p libaprs-engine` when parser performance changed.
- Verify the declared MSRV through the release script. It runs Rust `1.80.0`
  checks when that toolchain is installed.
- Run `cargo audit` and `cargo deny check`, either locally or through the
  security workflow, before publishing.
- Record fuzz compile and fuzz corpus hygiene results when parser, transport,
  semantic decoder, or corpus files changed.
- Treat the Rust CI release-script job as a release dependency gate: it installs
  pinned `cargo-audit` and `cargo-deny` versions with
  `scripts/install-release-tools.sh` and runs both checks through
  `scripts/verify-release.sh`.
- The release-script job is intentionally skipped for pull requests and runs on
  `main` pushes or manual dispatch. Pull requests use the Rust matrix and docs
  fast-lane checks for quicker feedback.
- GitHub Actions Cargo caches intentionally include registry and git state, not
  `target`, to avoid runner disk exhaustion during post-job cache compression.
- Security workflows install pinned prebuilt release/security tools instead of
  compiling them during each run. Tool archives are SHA256 verified before use.
- GitHub Actions caches `~/.cargo/advisory-db` with a weekly key and older
  restore fallback. `cargo audit` still fetches advisory updates during the
  audit step, so restored stale cache data is refreshed before results are
  evaluated.
- Confirm `CHANGELOG.md` describes the release.
- Review `docs/public-api.md` and `crates/libaprs-engine/tests/api_compat.rs`
  when the release changes exported library APIs.
- Review `docs/downstream-feedback.md` and `docs/v2-migration.md` before any
  release that adds a soft deprecation, replacement API, or `v2.0.0`
  breaking-change candidate.
- Use `scripts/publish-release.sh` when publishing to crates.io and GitHub
  Releases; it encodes the crate publish order from `docs/publishing.md` and
  refuses to publish without explicit clean secure-review, gate, release-tag,
  and GitHub Release evidence.
- Use the default Cargo home for normal audit, deny, semver, package, and
  publish commands. In restricted environments where `~/.cargo` is not
  writable, use `CARGO_HOME=/tmp/libaprs-cargo-home` and copy crates.io
  credentials into that temporary Cargo home only at runtime when publishing.

## Remote Gate

- Confirm GitHub Actions is enabled before relying on remote CI.
- If GitHub Actions is blocked before job creation, document that CI was skipped
  and rely on the local gate.
- Do not publish while a required remote CI or security workflow is failing.

## Tagging

- Tag the release after local and remote verification pass.
- Push the tag before publishing so the GitHub Release step can use
  `--verify-tag`.
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
- Fuzz evidence: `cargo +nightly fuzz check` passed when available, corpus
  hygiene passed, and any new minimized regression inputs are documented.
- `LIBAPRS_REMOTE_CI=passed`: remote CI passed for the release commit. Use
  `skipped-documented` only when CI was unavailable and the release evidence
  records the reason.
- `LIBAPRS_RELEASE_COMMIT="$(git rev-parse HEAD)"`: the publish target matches
  the checked-out commit.
- `LIBAPRS_GITHUB_RELEASE=publish`: create or update the GitHub Release after
  crates.io publication and verify it is marked latest.
- `LIBAPRS_RELEASE_TAG=<tag>`: the pushed release tag, for example `v1.0.1`.
- `LIBAPRS_GITHUB_REPO=SoloSentryOrg/libaprs-engine`: the repository where the
  GitHub Release must be created. Use `LIBAPRS_GITHUB_RELEASE=skipped-documented`
  only if GitHub Releases are unavailable and release evidence records the
  reason.

## v2.0.0-rc.2 Release Evidence

- Tag: `v2.0.0-rc.2`.
- Commit: `33beabc8f699b5b747def04a543466492884f81c`.
- Scope: refreshes package metadata and downstream examples after the repository
  migration to `SoloSentryOrg/libaprs-engine`.
- Secure review: clean before tagging and publication; no open findings.
- Local gate:
  `CARGO_HOME=/tmp/libaprs-cargo-home scripts/verify-release.sh` passed after
  refreshing the stale temporary advisory cache at
  `/tmp/libaprs-cargo-home/advisory-db`.
- Remote GitHub Actions: release PR Rust CI run `25211638944` and security run
  `25211638864` passed; `main` push Rust CI run `25211688133` and security run
  `25211688144` passed for the release commit.
- crates.io publication: all workspace crates published as `2.0.0-rc.2`.
- GitHub Release: `v2.0.0-rc.2` created, marked latest, and verified at
  <https://github.com/SoloSentryOrg/libaprs-engine/releases/tag/v2.0.0-rc.2>.
- Post-publication downstream smoke:
  `CARGO_HOME=/tmp/libaprs-cargo-home LIBAPRS_RUN_DOWNSTREAM_SMOKE=1 LIBAPRS_PACKAGE_ALL=1 scripts/verify-release.sh`
  passed against crates.io and refreshed
  `examples/downstream-smoke/Cargo.lock` with `2.0.0-rc.2` checksums.
- Notes: crates.io metadata for the core crate, transport crates, and CLI now
  advertises the organization repository URL.

## v2.0.0-rc.1 Release Evidence

- Tag: `v2.0.0-rc.1`.
- Commit: `37c6ee29b5fe5bde018c9e8b510e2aba03750314`.
- Scope: removes the library `ParsedPacket::to_json()` diagnostic convenience
  method, keeps raw-byte-preserving structured diagnostics through
  `ParsedPacket::to_diagnostic()`, and keeps CLI `--json` as CLI-owned
  diagnostic output.
- Schema evidence: `serde_support::PacketDiagnostic` and CLI accepted-packet
  JSON now expose `schema_version = 1`.
- Secure review: clean before tagging and publication; no open findings.
- Local gate: `CARGO_HOME=/tmp/libaprs-cargo-home scripts/verify-release.sh`
  passed on a clean `/tmp` clone of the release commit before tagging and
  publication.
- Remote GitHub Actions: `main` push security run `25189100168` and Rust CI
  run `25189100182` passed for the release commit.
- crates.io publication: all workspace crates published as `2.0.0-rc.1`.
- GitHub Release: `v2.0.0-rc.1` created, marked latest, and verified at
  <https://github.com/SoloSentryOrg/libaprs-engine/releases/tag/v2.0.0-rc.1>.
- Post-publication downstream smoke:
  `CARGO_HOME=/tmp/libaprs-cargo-home LIBAPRS_RUN_DOWNSTREAM_SMOKE=1 LIBAPRS_PACKAGE_ALL=1 scripts/verify-release.sh`
  passed after refreshing the smoke project to target `2.0.0-rc.1`.
- Notes: downstream smoke now fails the release gate if the smoke manifest still
  targets an older workspace release version.

## v1.7.0 Release Evidence

- Tag: `v1.7.0`.
- Commit: `e11aba7d19723cebe95215c1d4c58cccd645c79a`.
- Secure review: clean before tagging and publication; no open findings.
- Local gate: `CARGO_HOME=/tmp/libaprs-cargo-home scripts/verify-release.sh`
  passed before publication.
- Remote GitHub Actions: release PR Rust CI run `25187204721` and security run
  `25187204691` passed; `main` push Rust CI run `25187254023` and security run
  `25187254024` passed for the merge commit.
- crates.io publication: all workspace crates published as `1.7.0`.
- GitHub Release: `v1.7.0` created, marked latest, and verified at
  <https://github.com/SoloSentryOrg/libaprs-engine/releases/tag/v1.7.0>.
- Post-publication downstream smoke:
  `CARGO_HOME=/tmp/libaprs-cargo-home LIBAPRS_RUN_DOWNSTREAM_SMOKE=1 LIBAPRS_PACKAGE_ALL=1 scripts/verify-release.sh`
  passed against crates.io and refreshed
  `examples/downstream-smoke/Cargo.lock` with `1.7.0` checksums.
- Notes: adds `ParsedPacket::to_diagnostic()` behind the `serde` feature as an
  explicitly structured diagnostic alternative to convenience JSON, expands API
  compatibility tests, and adds a downstream feedback issue template.

## v1.6.0 Release Evidence

- Tag: `v1.6.0`.
- Commit: `89e4b08cf6f7c8fca6e6bfd2bd7a89fe1f1ba3ca`.
- Local gate: `CARGO_HOME=/tmp/libaprs-cargo-home scripts/verify-release.sh`
  passed before publication.
- Remote GitHub Actions: release PR Rust CI run `25185223073` and security run
  `25185223085` passed; `main` push Rust CI run `25185287243` and security run
  `25185287223` passed for the merge commit.
- crates.io publication: all workspace crates published as `1.6.0`.
- GitHub Release: `v1.6.0` created, marked latest, and verified at
  <https://github.com/SoloSentryOrg/libaprs-engine/releases/tag/v1.6.0>.
- Post-publication downstream smoke:
  `CARGO_HOME=/tmp/libaprs-cargo-home LIBAPRS_RUN_DOWNSTREAM_SMOKE=1 LIBAPRS_PACKAGE_ALL=1 scripts/verify-release.sh`
  passed against crates.io and refreshed
  `examples/downstream-smoke/Cargo.lock` with `1.6.0` checksums.
- Notes: publishes the completed `1.x` roadmap batches since `v1.1.0`,
  including conformance expansion, abuse-resistance gates, observability events,
  transport maturity guidance, downstream feedback, and `v2.0.0` migration
  planning.

## v1.1.0 Release Evidence

- Tag: `v1.1.0`.
- Commit: `2b81cfbd007deecd6863bef4e5fb77685d9254bd`.
- Local gate: `CARGO_HOME=/tmp/libaprs-cargo-home scripts/verify-release.sh`
  passed before publication.
- Remote GitHub Actions: release PR Rust CI run `25125462505` and security run
  `25125462437` passed; `main` push Rust CI run `25125723560` and security run
  `25125723586` passed for the merge commit.
- crates.io publication: all workspace crates published as `1.1.0`.
- GitHub Release: `v1.1.0` created, marked latest, and verified at
  <https://github.com/SoloSentryOrg/libaprs-engine/releases/tag/v1.1.0>.
- Post-publication downstream smoke:
  `CARGO_HOME=/tmp/libaprs-cargo-home LIBAPRS_RUN_DOWNSTREAM_SMOKE=1 LIBAPRS_PACKAGE_ALL=1 scripts/verify-release.sh`
  passed against crates.io and refreshed
  `examples/downstream-smoke/Cargo.lock` with `1.1.0` checksums.
- Notes: production ergonomics release with structured diagnostics, a
  machine-readable support matrix, operations documentation, a service ingest
  example, and workspace crate version promotion to `1.1.0`.

## v1.0.0 Release Evidence

- Tag: `v1.0.0`.
- Commit: `97a458eeccaa876856fb4103e469829ac44d7033`.
- Local gate: `CARGO_HOME=/tmp/libaprs-cargo-home scripts/verify-release.sh`
  passed before publication.
- Remote GitHub Actions: final release PR Rust CI run `25076219082` and
  security run `25076219002` passed; `main` push Rust CI run `25076239313`
  and security run `25076239328` passed for the merge commit.
- crates.io publication: all workspace crates published as `1.0.0`.
- GitHub Release: `v1.0.0` created and marked latest.
- Post-publication downstream smoke:
  `CARGO_HOME=/tmp/libaprs-cargo-home LIBAPRS_RUN_DOWNSTREAM_SMOKE=1 LIBAPRS_PACKAGE_ALL=1 scripts/verify-release.sh`
  passed against crates.io and refreshed
  `examples/downstream-smoke/Cargo.lock` with `1.0.0` checksums.
- Notes: final `1.0.0` promotion of the tested `1.0.0-rc.1` release
  candidate after clean secure review, local release gate, remote CI, security
  gate, exact release commit verification, and post-publication downstream
  smoke.

## v1.0.0-rc.1 Release Candidate Evidence

- Tag: `v1.0.0-rc.1`.
- Commit: `174a2a879b1c39b4b2844fafad7bc754dba6233d`.
- Local gate: `CARGO_HOME=/tmp/libaprs-cargo-home scripts/verify-release.sh`
  passed before publication.
- Remote GitHub Actions: Rust CI push run `25075045233` passed on `main`;
  security push run `25074771650` passed for the release-candidate version
  bump.
- crates.io publication: all workspace crates published as `1.0.0-rc.1`.
- Post-publication downstream smoke:
  `LIBAPRS_RUN_DOWNSTREAM_SMOKE=1 LIBAPRS_PACKAGE_ALL=1 scripts/verify-release.sh`
  passed against crates.io and refreshed
  `examples/downstream-smoke/Cargo.lock` with `1.0.0-rc.1` checksums.
- Notes: first `1.0.0` release candidate after API stabilization, APRS
  semantics completion, security and robustness hardening, transport
  reliability hardening, and CI release-gate improvements. Do not promote to
  `1.0.0` until the release candidate has had downstream review time and any
  findings are fixed.

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
  <https://github.com/SoloSentryOrg/libaprs-engine/actions/runs/24937390267>.
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
