# Downstream Feedback

![libaprs-engine documentation header](assets/brand/docs-header.svg)

## BLUF

- Treat this file as the evidence log for current-major API stability and future
  major-version planning.
- Do not break public APIs based on preference alone; record downstream pain
  first.
- Current feedback is inferred from repository integration patterns, release
  gates, examples, documented operator workflows, and the completed `v2.0.0`
  release cycle.
- New user reports should be converted into tests, docs, or migration notes
  before they influence a breaking change.
- Raw-byte preservation and fail-closed parsing are non-negotiable constraints
  for every downstream use case.

## Downstream Use Cases

### Parser Library Integrations

Applications use `libaprs-engine` directly to parse packet byte slices, inspect
semantic views, and keep raw bytes for replay or audit.

Observed needs:

- stable parse and policy diagnostic codes,
- exact raw-byte access for accepted packets,
- no UTF-8 precondition on payloads,
- additive semantic field extraction, and
- clear guidance on which APIs are stable through the current major release.

### Operational Inspection And Corpus Replay

Operators use `aprs-cli`, fixture corpora, and replay examples to triage packet
families, malformed inputs, and policy rejection rates.

Observed needs:

- machine-readable support matrix output,
- JSON output that is useful for diagnostics but not mistaken for a stable
  external contract,
- strict failure modes for oversized or malformed inputs, and
- examples that use bounded byte reads instead of text conversion.

### Long-Running Ingestion Services

Service integrations use `Engine`, `EngineEvent`, counters, policy rejection
codes, and optional metrics helpers to monitor ingestion quality.

Observed needs:

- stable event kinds for accepted, rejected, malformed, and transport-failure
  paths,
- saturating counters for telemetry,
- bounded malformed raw-byte evidence, and
- dependency-free metric helpers that do not choose a runtime,
- small runtime-neutral helpers for duplicate suppression, rate budgets, and
  semantic-family rejection without pulling storage or async choices into the
  crate.

### Transport Adapter Integrations

Transport users combine file, TCP, APRS-IS, KISS, serial, UDP, HTTP, MQTT,
AX.25, corpus, channel, file-watch, and async helper crates with their own
runtime and operational policy.

Observed needs:

- byte-preserving packet/frame boundaries,
- explicit read and packet-size limits,
- caller-owned timeout, retry, reconnect, and backpressure decisions, and
- profile helpers for APRS-IS filters, q constructs, and login validation
  without requiring the parser core to accept transport-only path metadata,
- narrow options structs instead of a broad transport framework in the current
  major release.

### Packet Construction Integrations

Applications that inspect or bridge APRS traffic also need safe packet
construction for common packet families.

Observed needs:

- owned bytes that callers can choose to transmit, store, or discard,
- parser-compatible address and length validation before bytes are emitted,
- no implicit transport side effects, normalization, or logging, and
- round-trip tests for packet families already supported by the parser.

## Pain Points And Current Decisions

| Area | Pain point | Current decision | Future major consideration |
| --- | --- | --- | --- |
| Diagnostic JSON | `ParsedPacket::to_json()` was convenient but could look like a stable wire schema. | Removed in `v2.0.0-rc.1` after `ParsedPacket::to_diagnostic()` shipped in `1.7.0`. | Keep CLI JSON as CLI-owned diagnostics and use structured Rust diagnostics or application-owned schemas for integrations. |
| Semantic APIs | `AprsData` is useful but still expanding as APRS coverage deepens. | Keep additions source-compatible and preserve raw bytes. | Split stable envelope views from evolving semantic interpretations if field growth becomes awkward. |
| Transport abstractions | Adapters share byte-oriented behavior but differ in runtime and failure ownership. | Keep `PacketSource`, `PacketSink`, `LineTransport`, and adapter-specific options. | Introduce stronger traits only if multiple downstream users need the same receive-loop contract. |
| Encoding APIs | Encoders can be mistaken for a transmit framework. | Keep encoders as owned-byte constructors only; transport, auth, logging, and policy remain caller-owned. | Split encoder modules further only if additive growth becomes confusing. |
| Service helpers | Long-running services need common policy building blocks but not a framework. | Add runtime-neutral duplicate, budget, and blocklist helpers. | Introduce richer policy composition only if downstream reports show repeated safe-use problems. |
| Error taxonomy | Parse, policy, and transport diagnostics are structured, but some names may remain broader than future users want. | Keep codes stable and add detail additively. | Rename or split confusing codes only with migration evidence. |
| Compatibility proof | Stable core APIs have tests, but transport option surfaces need explicit tripwires as adapters mature. | Expand compatibility tests with additive coverage. | Use semver checks plus downstream smoke before any release candidate. |

## Accepted Internal Release-Gate Finding

### Diagnostic JSON API Boundary

- Integration type: parser library and CLI diagnostics.
- Crate and version: `libaprs-engine` `1.7.0` preparing the published
  `2.0.0-rc.1` release candidate.
- Finding source: internal secure-review and compatibility-test review after
  adding `ParsedPacket::to_diagnostic()` in `1.7.0`.
- Raw-byte behavior: the replacement keeps exact accepted-packet bytes in
  `PacketDiagnostic::raw` as `Vec<u8>`.
- Fail-closed behavior: parser behavior is unchanged.
- API pain: the library method name `ParsedPacket::to_json()` looked like a
  stable serialization contract even though project docs classified it as
  diagnostic convenience. Keeping it beside the structured serde replacement
  left two competing integration paths.
- Accepted outcome: remove `ParsedPacket::to_json()` in `v2.0.0-rc.1`, add
  `schema_version` to `PacketDiagnostic`, keep CLI `--json` as CLI-owned
  diagnostic output, and document migration to `to_diagnostic()`,
  `PacketDiagnostic`, `PacketSummary`, `EngineEvent`, or application-owned
  schemas.

## Feedback Intake Template

Record downstream issues with:

- integration type: parser, CLI, service, transport, or release tooling,
- crate and version,
- packet family or transport source,
- whether raw bytes were preserved,
- whether malformed input failed closed,
- API name or behavior that caused confusion,
- reproduction packet or minimized fixture, if publishable, and
- proposed compatible current-major fix or justified future-major break.

## Required Follow-Up For Accepted Feedback

- Add or update a regression test before changing behavior.
- Update `docs/public-api.md` when a public API changes.
- Update `docs/stability.md` when the stability tier changes.
- Update or create the relevant migration plan when a future breaking change is
  identified.
- Add release evidence to `docs/release.md` before publishing.
