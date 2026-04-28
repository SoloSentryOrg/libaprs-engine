# Stability

This project is pre-1.0. Public APIs are usable, and this document defines the
compatibility intent until a `1.0.0` release locks semantic versioning
guarantees. The candidate `1.0.0` public boundary is maintained in
[Public API Boundary](public-api.md).

## Compatibility Policy

- Patch releases should not intentionally break candidate stable APIs.
- Minor releases may add fields, variants, helper methods, crates, or feature
  flags.
- Breaking changes to candidate stable APIs should be reserved for minor
  releases before `1.0.0`, covered by compatibility test updates, and called
  out in `CHANGELOG.md`.
- Candidate stable APIs are covered by
  `crates/libaprs-engine/tests/api_compat.rs` so documented integration
  patterns fail in CI if they drift.
- Release checks should run `cargo-semver-checks` when installed. Pre-1.0
  semver still allows breaking changes in minor releases, but semver output
  must be reviewed before publishing.
- Experimental semantic APIs may change while APRS coverage matures, but changes
  must preserve raw-byte access and fail-closed parsing behavior.
- Diagnostic JSON from `ParsedPacket::to_json()` remains convenience output, not
  a compatibility-stable wire schema.

## 1.0 Readiness Criteria

The project should remove pre-1.0 caveats only after these gates are true:

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

## Candidate Stable APIs

These APIs are intended to remain source-compatible through `1.0.0` unless a
secure code review finds a safety issue that requires a breaking change:

- `parse_packet`
- `parse_packet_with_options`
- `ParseOptions`
- `DEFAULT_PARSE_OPTIONS`
- `MAX_PACKET_LEN`
- `RawPacket`
- `ParsedPacket`
- `ParseError::code`
- existing `DataTypeIdentifier` variants and `DataTypeIdentifier::name`
- `PacketSummary`, with additive fields allowed before `1.0.0`
- `PolicyRejection::code`
- `Engine`
- `EngineResult`
- `Counters`
- `Policy`
- `PolicyDecision`
- `PolicyRejection`
- `LineTransport`
- `PacketSource`
- `PacketSink`
- `TransportErrorCode::code`
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

- `ParsedPacket::to_json`
- `serde_support::PacketDiagnostic`

Do not treat `to_json()` as a long-term wire protocol. Use
`PacketDiagnostic` with the `serde` feature or define an application-owned
schema when external compatibility matters.

## Feature Flags

- `std`: default feature for standard-library support.
- `alloc`: reserved for future allocation-only support.
- `serde`: optional diagnostic serialization support.

The crate is not currently `no_std`. The `alloc` feature exists to make the
future split explicit without promising current `no_std` compatibility.

## Workspace Crates

- `libaprs-engine`: stable-intent parser, engine, policy, line transport, and
  semantic API surface.
- `aprs-cli`: operational inspection tool; command-line flags may expand before
  `1.0.0`.
- `aprs-transport-file`: stable-intent file helper crate.
- `aprs-transport-tcp`: optional TCP helper crate. Network I/O stays outside the
  parser core.
- `aprs-transport-aprs-is`, `aprs-transport-kiss`, `aprs-transport-serial`,
  `aprs-transport-udp`, `aprs-transport-http`,
  `aprs-transport-file-watch`, `aprs-transport-mqtt`,
  `aprs-transport-ax25`, `aprs-transport-corpus`,
  `aprs-transport-channel`, and `aprs-transport-async`: optional transport
  helper crates. APIs may expand before `1.0.0`, but byte preservation remains
  a stable design constraint.
