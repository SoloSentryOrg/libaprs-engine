# Stability

![libaprs-engine documentation header](assets/brand/docs-header.svg)

This project has reached the `3.0.0-rc.2` release-candidate line without an
intentional public API break from `2.6.0`. The public APIs listed in
[Public API Boundary](public-api.md) are semver-protected integration contracts
for the current major release line.

## Compatibility Policy

- Patch releases must not intentionally break stable APIs.
- Minor releases may add fields, variants, helper methods, crates, or feature
  flags in source-compatible ways.
- Breaking changes to stable APIs require a major version bump, compatibility
  test updates, secure review, and explicit `CHANGELOG.md` migration notes.
- Stable APIs are covered by
  `crates/libaprs-engine/tests/api_compat.rs` so documented integration
  patterns fail in CI if they drift.
- Release checks should run `cargo-semver-checks` when installed, and semver
  output must be reviewed before publishing.
- Experimental semantic APIs may change while APRS coverage matures, but changes
  must preserve raw-byte access and fail-closed parsing behavior.
- Library diagnostic JSON is not a compatibility-stable wire schema.
  `ParsedPacket::to_json()` was removed in `v2.0.0-rc.1` after an explicit
  replacement path was added.
- Soft deprecations for new integrations are tracked in release-specific
  migration plans. Historical `v2.0.0` guidance remains in
  [`v2.0.0` Migration Plan](v2-migration.md); `v3.0.0-rc.2` guidance is tracked
  in [`v3.0.0` Migration Plan](v3-migration.md).

## Release Maintenance Criteria

Release maintenance keeps these gates true:

- Candidate stable APIs in `docs/public-api.md` have at least one compatibility
  test and one documentation example.
- `cargo-semver-checks` runs in the release CI path and any breaking result is
  either fixed or explicitly called out in `CHANGELOG.md` before release.
- Public API changes are reviewed against `docs/api.md`, `docs/public-api.md`,
  `crates/libaprs-engine/tests/api_compat.rs`, and the semver-check output.
- APRS semantic families in `docs/conformance.md` have fixture coverage for
  accepted, malformed, and policy-rejected cases where applicable.
- Parser and transport fuzz targets compile, and any discovered crash or
  panic is reduced to a deterministic regression test.
- Transport adapters document ownership of authentication, timeouts, bounding,
  retries, and byte-preservation responsibilities.

## Stable APIs

These APIs are intended to remain source-compatible through the `3.x` release
line unless a secure code review finds a safety issue that requires a breaking
change:

- `parse_packet`
- `parse_packet_with_options`
- `ParseOptions`
- `DEFAULT_PARSE_OPTIONS`
- `MAX_PACKET_LEN`
- `EVENT_RAW_BYTE_LIMIT`
- `RawPacket`
- `ParsedPacket`
- `ParseError::code`
- existing `DataTypeIdentifier` variants and `DataTypeIdentifier::name`
- `PacketSummary`, with additive fields allowed in minor releases
- `PolicyRejection::code`
- `Engine`
- `EngineResult`
- `EngineEvent`
- `EngineEventKind`
- `AcceptedPacketEvent`
- `PolicyRejectedPacketEvent`
- `MalformedPacketEvent`
- `TransportFailureEvent`
- `Counters`
- `Policy`
- `PolicyDecision`
- `PolicyRejection`
- `LineTransport`, including bounded `packets_with_limit`
- `PacketSource`
- `PacketSink`
- `TransportErrorCode::code`
- `aprs-transport-tcp::TcpReadOptions`
- `encoder` owned-byte packet construction helpers
- `service` runtime-neutral duplicate, rate-budget, and semantic blocklist
  helpers
- `aprs-transport-aprs-is` profile helpers for login validation, filters, and
  q-construct diagnostics
- `DEFAULT_TRANSPORT_READ_LIMIT`
- `read_all_with_limit`
- `oversized_input_error`

## Experimental APIs

These APIs may change as APRS semantic coverage matures:

- `AprsData`
- semantic field structs such as `Position`, `Object`, `Item`, `Weather`,
  `Telemetry`, `TelemetryMetadata`, `Nmea`, `MicE`, and `ThirdParty`
- typed interpretation helper methods
- newly added `DataTypeIdentifier` variants for APRS families not yet
  represented
- stricter malformed semantic classification where raw-byte preservation is
  maintained

## Diagnostic APIs

These APIs are for inspection and observability:

- `ParsedPacket::to_diagnostic`, behind the `serde` feature
- `ParseError::diagnostic`
- `PolicyRejection::diagnostic`
- `TransportErrorCode::diagnostic`
- `support_matrix`
- `serde_support::PacketDiagnostic`
- `metrics_support`, behind the `metrics` feature

Use `ParsedPacket::to_diagnostic()` or `PacketDiagnostic` with the `serde`
feature, `EngineEvent` structs, CLI JSON schemas, or define an
application-owned schema when external compatibility matters.

## Feature Flags

- `std`: default feature for standard-library support.
- `alloc`: reserved for future allocation-only support.
- `serde`: optional diagnostic serialization support.
- `metrics`: optional dependency-free counter metric helpers.

The crate is not currently `no_std`. The `alloc` feature exists to make the
future split explicit without promising current `no_std` compatibility.

## Workspace Crates

- `libaprs-engine`: stable-intent parser, engine, policy, line transport, and
  semantic API surface.
- `aprs-cli`: operational inspection tool; command-line flags may expand in
  minor releases.
- `aprs-transport-file`: stable-intent file helper crate.
- `aprs-transport-tcp`: optional TCP helper crate. Network I/O stays outside the
  parser core. `TcpReadOptions` is the stable timeout/read-limit configuration
  surface for TCP address helpers.
- `aprs-transport-aprs-is`, `aprs-transport-kiss`, `aprs-transport-serial`,
  `aprs-transport-udp`, `aprs-transport-http`,
  `aprs-transport-file-watch`, `aprs-transport-mqtt`,
  `aprs-transport-ax25`, `aprs-transport-corpus`,
  `aprs-transport-channel`, and `aprs-transport-async`: optional transport
  helper crates. APIs may expand in minor releases, but byte preservation
  remains a stable design constraint.

## Deprecation Planning

The project records deprecation evidence before changing stable APIs:

- integration pain points belong in the internal downstream evidence log,
- migration guidance belongs in a release-specific migration plan,
- stable replacements must have compatibility tests before a release candidate,
  and
- any breaking removal or rename waits for a major-version release candidate.
