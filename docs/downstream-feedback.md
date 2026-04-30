# Downstream Feedback

## BLUF

- Treat this file as the evidence log for `1.x` API stability and `v2.0.0`
  planning.
- Do not break public APIs based on preference alone; record downstream pain
  first.
- Current feedback is inferred from repository integration patterns, release
  gates, examples, and documented operator workflows.
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
- clear guidance on which APIs are stable through `1.x`.

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
- dependency-free metric helpers that do not choose a runtime.

### Transport Adapter Integrations

Transport users combine file, TCP, APRS-IS, KISS, serial, UDP, HTTP, MQTT,
AX.25, corpus, channel, file-watch, and async helper crates with their own
runtime and operational policy.

Observed needs:

- byte-preserving packet/frame boundaries,
- explicit read and packet-size limits,
- caller-owned timeout, retry, reconnect, and backpressure decisions, and
- narrow options structs instead of a broad transport framework in `1.x`.

## Pain Points And Current Decisions

| Area | Pain point | `1.x` decision | `v2.0.0` consideration |
| --- | --- | --- | --- |
| Diagnostic JSON | `ParsedPacket::to_json()` was convenient but could look like a stable wire schema. | Removed from the `v2.0.0-rc.1` review branch after `ParsedPacket::to_diagnostic()` shipped in `1.7.0`. | Keep CLI JSON as CLI-owned diagnostics and use structured Rust diagnostics or application-owned schemas for integrations. |
| Semantic APIs | `AprsData` is useful but still expanding as APRS coverage deepens. | Keep additions source-compatible and preserve raw bytes. | Split stable envelope views from evolving semantic interpretations if field growth becomes awkward. |
| Transport abstractions | Adapters share byte-oriented behavior but differ in runtime and failure ownership. | Keep `PacketSource`, `PacketSink`, `LineTransport`, and adapter-specific options. | Introduce stronger traits only if multiple downstream users need the same receive-loop contract. |
| Error taxonomy | Parse, policy, and transport diagnostics are structured, but some names may remain broader than future users want. | Keep codes stable and add detail additively. | Rename or split confusing codes only with migration evidence. |
| Compatibility proof | Stable core APIs have tests, but transport option surfaces need explicit tripwires as adapters mature. | Expand compatibility tests with additive coverage. | Use semver checks plus downstream smoke before any release candidate. |

## Accepted Internal Release-Gate Finding

### Diagnostic JSON API Boundary

- Integration type: parser library and CLI diagnostics.
- Crate and version: `libaprs-engine` `1.7.0`, preparing
  `2.0.0-rc.1`.
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
- proposed compatible `1.x` fix or justified `v2.0.0` break.

## Required Follow-Up For Accepted Feedback

- Add or update a regression test before changing behavior.
- Update `docs/public-api.md` when a public API changes.
- Update `docs/stability.md` when the stability tier changes.
- Update `docs/v2-migration.md` when a future breaking change is identified.
- Add release evidence to `docs/release.md` before publishing.
