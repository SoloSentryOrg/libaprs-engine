# Roadmap To The Next Major Version

## BLUF

- The next major version target is `v3.0.0`, and `v3.0.0-rc.2` is published as
  the security fix-forward release candidate.
- `v2.5.0` completed the planned `v2.1.0` through `v2.5.0` additive release
  track in one published release.
- `v2.6.0` completed the pre-`v3.0.0` evidence-first readiness track: it stayed
  additive, hardened gates, and collected proof before any breaking API cleanup.
- Prioritise protocol coverage, interoperability, safe packet construction,
  production service building blocks, release assurance, and evidence gates
  before any breaking API cleanup.
- Do not invent a breaking API change just to justify the major version:
  `v3.0.0-rc.2` is a no-intentional-break release candidate unless semver
  checks prove otherwise.
- Every release must preserve raw bytes, fail closed on malformed packet shape,
  complete secure review, pass local and remote gates, publish crates.io
  packages, create GitHub Release evidence, update GitHub Project #3, and sync
  this roadmap backup.

## Current Project Status

GitHub Project
[#3](https://github.com/orgs/SoloSentryOrg/projects/3) is the primary roadmap
and project-management store. This table is the repository backup snapshot,
last synced on 2026-05-07.

| Version or epic | Project status | Release status |
| --- | --- | --- |
| `v1.1.0`: Production Ergonomics | Done | Released 2026-04-29 |
| `v1.2.0`: APRS Conformance Expansion | Done | Completed in `v1.6.0`; no standalone release |
| `v1.3.0`: Security And Abuse Resistance | Done | Completed in `v1.6.0`; no standalone release |
| `v1.4.0`: Observability And Service Integration | Done | Completed in `v1.6.0`; no standalone release |
| `v1.5.0`: Transport Maturity | Done | Completed in `v1.6.0`; no standalone release |
| `v1.6.0`: Downstream Feedback And Deprecation Planning | Done | Released 2026-04-30 |
| `v1.7.0`: Structured Packet Diagnostic API | Done | Released 2026-04-30 |
| `v2.0.0-rc.1`: Breaking API Candidate | Done | Released 2026-04-30 |
| `v2.0.0-rc.2`: Metadata And Org Migration Release Candidate | Done | Released 2026-05-01 |
| `v2.0.0`: Final Major Release | Done | Released 2026-05-01 |
| `v2.1.0`: APRS Conformance Depth | Done | Completed in `v2.5.0`; no standalone release |
| `v2.2.0`: Interoperability Profiles | Done | Completed in `v2.5.0`; no standalone release |
| `v2.3.0`: Encoding And Packet Construction | Done | Completed in `v2.5.0`; no standalone release |
| `v2.4.0`: Production Service Toolkit | Done | Completed in `v2.5.0`; no standalone release |
| `v2.5.0`: Assurance And API Readiness | Done | Released 2026-05-01 |
| `v2.6.0`: Evidence-First Readiness | Done | Released 2026-05-03 |
| `v3.0.0-rc.1`: Major Release Candidate | Done | Released 2026-05-04 as prerelease |
| `v3.0.0-rc.2`: Security Fix-Forward Release Candidate | Done | Released 2026-05-07 as prerelease |
| `v3.0.0`: Final Major Release | In progress | Not released |

Release evidence is recorded in [docs/release.md](docs/release.md).

## Release Principles

- Protocol-first: callers pass bytes, accepted packets retain exact raw bytes,
  and malformed packet shape fails closed.
- Security-first: treat every packet and transport input as untrusted.
- Semver discipline: keep `2.x` additive unless a security issue requires a
  documented exception.
- Evidence-based: require conformance fixtures, compatibility tests, release
  gates, and release evidence for each published version.
- Runtime isolation: keep the core parser network-free and async-free; put
  integration concerns in optional crates or examples.
- Major-version restraint: do not break `2.x` APIs until there is real
  downstream or maintenance evidence that an additive design is not enough.

## Research Signals

- APRS 1.0, 1.1, and 1.2 remain the primary protocol references. The `1.2`
  proposal list highlights telemetry range updates, `!DAO!`, frequency
  extension, Mic-E type-code work, weather extensions, and message variants.
- APRS-IS interoperability depends on TNC2 monitor format, server-side filters,
  q-construct handling, uppercase callsign expectations, and TCP/WebSocket
  connection modes.
- Existing parser ecosystems set expectations for broad semantic extraction
  across normal and compressed positions, Mic-E, NMEA, objects, items, messages,
  telemetry, and weather packets.
- LoRa APRS deployments commonly expose KISS over TCP or serial, so KISS and
  byte-preserving TNC boundaries should remain prominent.
- Rust API maturity should continue to follow the Rust API Guidelines:
  documented public APIs, meaningful errors, common traits, caller control,
  type-safe options, and future-proof struct boundaries.

## 1. `v2.1.0`: APRS Conformance Depth

Priority: highest. This release should close known semantic gaps and make the
support matrix more defensible without breaking the `2.x` API.

- Add conformance tests and fixtures for currently partial or documented gaps:
  - compressed-position weather extraction,
  - Mic-E altitude, ambiguity, and telemetry extensions,
  - third-party nested packet helper behavior,
  - `!DAO!` high-precision position extension,
  - PHG, range, altitude, and frequency comment extensions,
  - telemetry values in the `000-999` range, and
  - APRS message variants from the `1.2` proposal list where they can be
    represented additively.
- Keep malformed semantic payloads visible to policy instead of panicking or
  lossy-decoding.
- Update `docs/conformance.md`, `docs/api.md`, and the support matrix as
  semantic gaps close.
- Add fixtures before implementation and preserve exact packet bytes in every
  accepted case.

Target outcome: `libaprs-engine` has materially broader APRS semantic coverage
and clearer evidence for what is supported, partial, malformed, and unsupported.

## 2. `v2.2.0`: Interoperability Profiles

Priority: high. This release should prove the crate works well with common APRS
network and TNC-style deployments.

- Add APRS-IS profile helpers and tests for:
  - login line validation,
  - filter string construction and validation,
  - q-construct diagnostics,
  - uppercase callsign expectations, and
  - documented TCP/WebSocket integration boundaries.
- Add KISS TCP and serial examples that map cleanly to Dire Wolf and LoRa APRS
  TNC use cases.
- Add an interoperability fixture corpus for APRS-IS, KISS, and LoRa APRS
  packet captures that are safe to publish.
- Keep authentication, reconnect loops, TLS, and runtime ownership outside the
  core parser.

Target outcome: downstream users can wire the project into common APRS-IS,
KISS, and LoRa APRS deployments without guessing at safe byte boundaries.

## 3. `v2.3.0`: Encoding And Packet Construction

Priority: medium-high. This release should add safe packet construction while
preserving the parser's byte-first contract.

- Add additive encoder APIs for:
  - status packets,
  - uncompressed position packets,
  - object and item packets,
  - messages, acknowledgements, bulletins, and announcements,
  - telemetry reports and metadata packets,
  - APRS-IS login lines, and
  - KISS frames where the transport crate already owns the framing boundary.
- Return owned bytes from encoders and make callers choose when to transmit,
  log, or normalize display text.
- Validate callsigns, paths, lengths, and packet fields before emitting bytes.
- Add round-trip tests where parse support already exists.

Target outcome: the project moves from parser-only to bidirectional protocol
tooling without adding unsafe transmit policy or network side effects.

## 4. `v2.4.0`: Production Service Toolkit

Priority: medium. This release should make long-running services easier to
build without turning the project into a monolithic iGate framework.

- Add reusable examples for ingestion pipelines, bounded replay, and structured
  event handling.
- Add optional policy helpers for duplicate suppression, rate limiting, and
  known-bad packet families.
- Add metrics adapter examples that remain feature-gated and runtime-neutral.
- Add operational playbooks for APRS-IS collectors, KISS/TNC readers, corpus
  replay, and service health checks.
- Keep service orchestration, storage, TLS, and authentication choices
  application-owned.

Target outcome: users can build inspectors, collectors, gateways, and monitoring
services from safe building blocks instead of copying ad hoc glue code.

## 5. `v2.5.0`: Assurance And API Readiness

Priority: medium. This release should decide whether `v3.0.0` is justified and
make that decision evidence-based.

- Add differential fixture comparisons against established parser behavior
  where licensing and fixture sources allow it.
- Expand fuzz campaigns and minimized regression corpora for semantic families
  touched in `v2.1.0` through `v2.4.0`.
- Run a Rust API Guidelines audit of public names, traits, constructors,
  options, error types, feature flags, and documentation examples.
- Add supply-chain evidence improvements such as auditable binary guidance or
  SBOM documentation where useful.
- Update the internal downstream evidence log and the `v3.0.0` breaking-change decision
  record before any release candidate work starts.

Target outcome: the project either approves a narrow `v3.0.0` breaking list or
continues additively in `2.x`.

## 6. `v2.6.0`: Evidence-First Readiness

Status: completed and released on 2026-05-03.

Priority: high. This release should complete the pre-`v3.0.0` evidence track
without approving a major-version break.

- Keep the downstream feedback record as an internal evidence log, not a public
  navigation target or public issue-template workflow.
- Add repository gates that fail when internal evidence docs become
  public-facing by accident.
- Add repository gates that require the `v2.6.0` evidence milestone,
  unreleased release notes, internal evidence marker, and gated `v3.0.0`
  decision record to stay in sync.
- Expand abuse-resistance regression tests for untrusted packet input,
  malformed semantic floods, invalid UTF-8, nested third-party packets, and
  oversized transport boundaries.
- Keep all work additive; if evidence reveals a possible break, record it first
  and continue fixing forward in `2.x` unless a security issue requires a
  separate reviewed exception.

Target outcome: the project has enforceable evidence gates and stronger
security regression coverage before deciding whether a `v3.0.0` release
candidate is justified.

## 7. `v3.0.0-rc.1`: Major Release Candidate

Priority: gated by `v2.6.0`. This release candidate should validate the next
major release line without inventing a breaking API change. If semver checks or
secure review find an actual break, fix it forward or document it as an
intentional break before publication.

Possible breaking-change candidates:

- Clearer separation of codec, semantic decoder, policy, diagnostics, and
  encoder modules.
- Stronger typed semantic views where additive growth in `AprsData` becomes
  awkward for downstream users.
- Stable trait contracts for packet sources, sinks, and transport receive loops.
- Renamed APIs only where downstream evidence shows confusion.
- Stricter feature organization if optional crate or feature behavior becomes
  hard to reason about.

Required gates:

- Run semver checks and document every intentional break; unexpected breaks are
  release blockers.
- Publish a migration guide before the release candidate.
- Run downstream smoke against `v3.0.0-rc.1`.
- Keep release-candidate evidence in `docs/release.md`.
- Regenerate SBOM and SHA-256 supply-chain evidence for the release-candidate
  package metadata.
- Fix any secure review, CI, conformance, or downstream findings before final
  `v3.0.0`.

Target outcome: `v3.0.0` is a promotion of a tested release candidate, not a
fresh unproven build.

## 7.1. `v3.0.0-rc.2`: Security Fix-Forward Release Candidate

Status: completed and released as a prerelease on 2026-05-07.

Priority: fix-forward after the published `v3.0.0-rc.1` candidate. This release
candidate published the security hardening merged after `rc.1` without adding
an intentional public API break.

Completed gates:

- Published a fresh prerelease version because `3.0.0-rc.1` is already
  published and immutable on crates.io.
- Preserve the no-intentional-public-API-break decision.
- Ran the full local release gate, remote CI, secure review, dependency
  policy checks, SBOM/hash evidence checks, and semver checks.
- Published crates through `scripts/publish-release.sh` with a prerelease GitHub
  Release so stable `v2.6.0` remains the latest release.
- Refreshed downstream smoke lockfile and release evidence after crates.io
  publication.

Target outcome: `v3.0.0-rc.2` is the tested candidate for final `v3.0.0`
promotion.

## 8. `v3.0.0`: Final Major Release

Priority: final. Publish only after the release candidate has clean evidence and
downstream review time.

- Promote the tested release candidate.
- Publish crates in guarded dependency order.
- Create or update the GitHub Release through `scripts/publish-release.sh`.
- Keep the `2.x` to `3.0.0` migration guide prominent.
- Tag only after local release gates, remote CI, security gates, and secure
  review pass for the exact release commit.

Target outcome: `v3.0.0` is a controlled major release with a clear migration
path and defensible release evidence.

## Recommended Execution Order

- `v2.5.0` completed the additive conformance, interoperability, encoding,
  service-toolkit, and API-readiness track.
- Do not start breaking API work until internal downstream evidence records specific
  evidence that an additive design is not enough.
- Keep each release independently reviewable and publishable.
- Keep GitHub Project #3 as the live roadmap and update this file as its
  repository-backed snapshot.
- After each release, update GitHub Project #3 first and then sync this file
  with the release date, project status, release status, and evidence links.
- Do not publish any release unless secure review, local gates, security gates,
  remote CI, crates.io publication, GitHub Release evidence, and post-publication
  smoke checks are complete.
