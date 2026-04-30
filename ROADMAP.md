# Roadmap To The Next Major Version

## BLUF

- The next major version target is `v2.0.0`, but the project should earn it
  through production-focused `1.x` releases first.
- Reserve `v2.0.0` for deliberate breaking changes: API cleanup, stronger type
  contracts, and clearer transport/runtime boundaries.
- Prioritise production usability, conformance depth, security assurance,
  observability, then API redesign.
- Do not break `1.x` APIs until there is real evidence from downstream use.
- Every release must keep the current rules: preserve raw bytes, fail closed on
  malformed packets, complete secure review, pass CI and security gates, publish
  crates.io packages, and create the GitHub Release.

## Release Principles

- Protocol-first: callers pass bytes, accepted packets retain exact raw bytes,
  and malformed packet shape fails closed.
- Security-first: treat every packet and transport input as untrusted.
- Semver-discipline: keep `1.x` additive unless a security issue requires a
  documented exception.
- Evidence-based: require conformance fixtures, compatibility tests, release
  gates, and release evidence for each published version.
- Runtime isolation: keep the core parser network-free and async-free; put
  integration concerns in optional crates or examples.

## 1. `v1.1.0`: Production Ergonomics

Priority: highest. This release should make the existing `1.0.0` API easier to
operate without changing its compatibility contract.

- Add richer structured diagnostics for parser, policy, and transport errors.
- Add machine-readable support matrix output from the CLI.
- Improve examples for common integrations:
  - APRS-IS reader
  - KISS stream
  - file corpus replay
  - service ingestion
- Add operator-focused documentation for deployment patterns, logging, limits,
  and safe defaults.
- Keep API changes additive only.

Target outcome: downstream developers can adopt `libaprs-engine` in production
workflows with clearer diagnostics and fewer integration decisions.

## 2. `v1.2.0`: APRS Conformance Expansion

Priority: high. This release should deepen protocol coverage and make the
support matrix more defensible.

- Expand APRS101 fixture coverage with more real-world packet families.
- Add malformed semantic fixtures for:
  - weather
  - Mic-E
  - telemetry
  - object and item packets
  - third-party packets
- Add conformance matrix tests proving every documented family has valid and
  invalid coverage.
- Improve semantic field extraction where current support is intentionally
  partial.
- Publish known unsupported edge cases explicitly.

Target outcome: documentation, tests, and parser behavior stay aligned as APRS
semantic support grows.

## 3. `v1.3.0`: Security And Abuse Resistance

Priority: high. This release should raise confidence for hostile or malformed
input at scale.

- Add deeper fuzz campaigns and minimized regression corpus management.
- Add resource-exhaustion tests across parser and transport boundaries.
- Add threat-model documentation for each crate.
- Strengthen dependency policy gates:
  - license allowlist
  - advisory checks
  - duplicate crate review
  - source allowlist
- Add release evidence requirements for fuzz and security review results.

Target outcome: the project has stronger OWASP-aligned evidence for untrusted
packet handling, dependency hygiene, and parser resilience.

## 4. `v1.4.0`: Observability And Service Integration

Priority: medium-high. This release should make long-running integrations easier
to monitor without weakening parser boundaries.

- Add stable diagnostic and event structs for:
  - accepted packets
  - policy-rejected packets
  - malformed packets
  - transport failures
- Add optional metrics adapters behind feature flags, without forcing runtime
  dependencies into the core crate.
- Add structured JSON schema documentation for CLI output.
- Add examples for long-running process integration.
- Keep counters monotonic and saturating.

Target outcome: operators can monitor ingestion quality, malformed-packet
volume, and policy decisions without scraping unstable text output.

## 5. `v1.5.0`: Transport Maturity

Priority: medium. This release should harden transport adapters while preserving
the core parser's runtime neutrality.

- Harden transport adapters with clearer timeout, backpressure, reconnect, and
  bounded-buffer guidance.
- Add integration tests for transport failure modes.
- Add APRS-IS reconnect and session examples without moving network behavior
  into the core parser.
- Review whether transport crates should share a stronger common trait layer.
- Keep runtime-specific behavior optional and isolated.

Target outcome: transport crates remain safe byte-preserving boundaries for
real deployments, while the parser core stays small and stable.

## 6. `v1.6.0`: Downstream Feedback And Deprecation Planning

Priority: medium. This release should convert real usage into a concrete
`v2.0.0` migration plan.

- Review downstream use cases and pain points.
- Deprecate weak API names or confusing patterns without removing them.
- Add migration notes for anything likely to change in `v2.0.0`.
- Add compile-time compatibility tests for intended `1.x` stable APIs.
- Decide the final `v2.0.0` breaking-change list.

Implementation notes:

- Record downstream evidence in `docs/downstream-feedback.md`.
- Track soft deprecations and migration guidance in `docs/v2-migration.md`.
- Keep deprecations documentation-level during `1.x` unless the release gate is
  explicitly changed to allow expected Rust deprecation warnings.

Target outcome: the project enters `v2.0.0` planning with measured evidence,
not speculative cleanup.

## 7. `v2.0.0-rc.1`: Breaking API Candidate

Priority: gated by `v1.6.0`. This release candidate should include only
justified breaking changes.

Current status: not approved for publication. The current decision record in
`docs/v2-breaking-changes.md` found no concrete downstream issue evidence that
justifies breaking `1.x` APIs yet.

Possible breaking-change candidates:

- Stronger typed packet views.
- Clearer separation between codec, semantic interpretation, policy, and
  diagnostics.
- Refined transport trait contracts.
- Stable diagnostic JSON schema.
- Cleaned-up names for ambiguous public APIs.

Required gates:

- Run semver checks and document every intentional break.
- Publish a migration guide before the release candidate.
- Run downstream smoke against `v2.0.0-rc.1`.
- Keep release-candidate evidence in `docs/release.md`.
- Fix any secure review, CI, conformance, or downstream findings before final
  `v2.0.0`.

Target outcome: `v2.0.0` is a promotion of a tested release candidate, not a
fresh unproven build.

## 8. `v2.0.0`: Final Major Release

Priority: final. Publish only after the release candidate has clean evidence and
downstream review time.

- Promote the tested release candidate.
- Publish crates in guarded dependency order.
- Create or update the GitHub Release through `scripts/publish-release.sh`.
- Keep the `1.x` to `2.0.0` migration guide prominent.
- Tag only after local release gates, remote CI, security gates, and secure
  review pass for the exact release commit.

Target outcome: `v2.0.0` is a controlled major release with a clear migration
path and defensible release evidence.

## Recommended Execution Order

- Start with `v1.1.0` because it improves usability without destabilising the
  new `1.0.0` API.
- Avoid starting `v2.0.0` breaking work until at least `v1.3.0` provides enough
  conformance and security feedback.
- Treat `v1.6.0` as the formal decision point for what deserves a major-version
  break.
- Keep each release batch independently reviewable and publishable.
- Do not publish any release unless secure review, local gates, security gates,
  remote CI, crates.io publication, and GitHub Release evidence are complete.
