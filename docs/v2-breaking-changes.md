# `v2.0.0` Breaking-Change Decision Record

## BLUF

- `v2.0.0-rc.1` is approved for review with one narrow breaking change.
- The approved break removes the ambiguous library `ParsedPacket::to_json()`
  method after the structured `to_diagnostic()` replacement shipped in `1.7.0`.
- The release-candidate scope is intentionally small: no parser envelope,
  policy, transport, or semantic behavior changes.
- Raw-byte preservation, fail-closed malformed-packet behavior, and parser core
  runtime neutrality are non-negotiable and cannot be traded for API cleanup.
- Revisit this record before any further release candidate or any branch that
  changes another stable public API.

## Decision Date

2026-04-30.

## Evidence Reviewed

- GitHub issues: none open or closed at the time of review.
- GitHub pull requests: merged project PRs `#1` through `#31`, including the
  `1.7.0` release, release evidence, API compatibility, transport maturity,
  observability, security, conformance, and structured diagnostic batches.
- Local public API audit: `crates/*/src` public declarations, `docs/api.md`,
  `docs/public-api.md`, `docs/stability.md`, compatibility tests, transport
  tests, and CLI tests.
- Internal downstream evidence log.
- Migration guidance: `docs/v2-migration.md`.
- Release evidence: `docs/release.md`.

## Current Decision

Start a narrow `v2.0.0-rc.1` implementation branch for the diagnostic JSON API
boundary only. The accepted evidence is an internal secure-review and
compatibility-test finding recorded in the internal downstream evidence log: the
library method name `ParsedPacket::to_json()` continued to look like a stable
serialization contract after a safer structured replacement was available.

No other candidate is approved for this release candidate. Continue additive
`1.x` or later `2.x` work for semantic envelope splits, transport trait changes,
or diagnostic taxonomy renames unless new evidence justifies them.

## Candidate Matrix

| Candidate | Decision | Evidence | Required before an RC |
| --- | --- | --- | --- |
| Rename or replace `ParsedPacket::to_json()` | Approved for `v2.0.0-rc.1`. Remove the library method and keep CLI JSON as CLI-owned diagnostic output. | Internal secure-review and compatibility-test finding recorded in the internal downstream evidence log; `ParsedPacket::to_diagnostic()`, `serde_support::PacketDiagnostic`, `PacketSummary`, `EngineEvent`, and application-owned schemas provide safer alternatives. | Add compatibility tests for `to_diagnostic()`, add `schema_version` to `PacketDiagnostic`, update docs and changelog, and review `cargo-semver-checks` output. |
| Split stable packet envelope APIs from evolving semantic interpretation APIs | Not approved as a breaking change yet. Continue additive semantic expansion. | `AprsData` is documented as evolving, and tests cover current semantic helpers. No issue shows the visible enum/struct surface blocking adoption. | Identify specific unstable semantic fields or variants that downstream code cannot absorb additively; add migration examples and semver evidence for the narrower envelope API. |
| Refine transport trait contracts around receive loops | Not approved as a breaking change yet. Keep adapter-specific options and shared byte traits. | Transport docs explicitly defer a stronger common layer until repeated downstream integrations need it. Existing adapters preserve bytes and expose bounded helpers. | Record multiple transport integrations needing the same runtime-neutral receive-loop trait; prove the trait does not force async, network, or runtime dependencies into the core crate. |
| Stabilize diagnostic or event serialization under explicit schema versions | Not approved as a breaking change yet. Keep stable event structs and versioned support-matrix JSON. | Operational docs warn that packet JSON is diagnostic. There is no downstream report requiring first-class event JSON as a crate-owned wire protocol. | Add additive schema types first; document schema versioning and rejection behavior; add tests proving unsupported schema versions fail closed in consumers or examples. |
| Rename broad parse, policy, or transport diagnostic names | Not approved as a breaking change yet. Keep current stable codes. | Current diagnostics expose stable machine-readable codes and no report identifies ambiguous names causing unsafe handling. | Record the exact ambiguous code, affected integration, and safer replacement; add migration tests and release notes mapping old-to-new codes. |

## Non-Negotiable Constraints For Any Future Break

- Accepted packets must retain exact raw input bytes.
- Malformed packet shape must fail closed without partial success.
- Invalid UTF-8 in payloads must not panic and must not be lossy-converted.
- Parser core must remain network-free and runtime-neutral.
- Transport helpers must keep explicit size limits and byte-preserving
  boundaries.
- Counters and observability must remain monotonic and saturating where
  applicable.
- Security review, local release gates, remote CI, security gates, downstream
  smoke, and release evidence must pass for the exact release commit.

## RC Entry Criteria

Before publishing `v2.0.0-rc.1`, update this record so every breaking change has:

- a concrete downstream report, compatibility-test finding, semver finding, or
  secure-review finding,
- the exact `1.x` API being changed,
- the replacement API and migration example,
- compatibility tests for the replacement path,
- `cargo-semver-checks` output reviewed and recorded,
- `docs/v2-migration.md`, `docs/public-api.md`, and `CHANGELOG.md` updates, and
- release evidence in `docs/release.md`.

For this release candidate, the approved break is:

- removed API: `ParsedPacket::to_json()`,
- replacement APIs: `ParsedPacket::to_diagnostic()` with the `serde` feature,
  `serde_support::PacketDiagnostic`, `PacketSummary`, `EngineEvent`, CLI
  `--json`, or application-owned schemas,
- migration test coverage:
  `crates/libaprs-engine/tests/api_compat.rs::stable_diagnostic_alternative_to_json_remains_usable`
  and `crates/libaprs-engine/tests/serde.rs::serde_diagnostic_preserves_non_utf8_bytes`,
- CLI diagnostic coverage:
  `crates/aprs-cli/tests/cli.rs::cli_reads_json_packets_from_stdin`, and
- release evidence target: `docs/release.md` after local and remote gates pass.

## Next Additive Work

- Keep `ParsedPacket::to_diagnostic()` and `serde_support::PacketDiagnostic`
  as the structured diagnostic path.
- Maintain compatibility tests around replacement APIs before considering any
  further public API removals.
- Convert every downstream report into an internal evidence entry, fixture,
  test, or migration note before changing stable APIs.
- Revisit common transport traits only after the criteria in
  `docs/transport-common-layer.md` are met.
- Continue publishing `1.x` releases until a non-empty breaking-change list is
  justified by evidence.
