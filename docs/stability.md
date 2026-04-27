# Stability

This project is pre-1.0. Public APIs are usable, and this document defines the
compatibility intent until a `1.0.0` release locks semantic versioning
guarantees.

## Compatibility Policy

- Patch releases should not intentionally break stable-intent APIs.
- Minor releases may add fields, variants, helper methods, crates, or feature
  flags.
- Breaking changes to stable-intent APIs should be reserved for minor releases
  before `1.0.0` and called out in `CHANGELOG.md`.
- Stable-intent APIs are covered by `crates/libaprs-engine/tests/api_compat.rs`
  so documented integration patterns fail in CI if they drift.
- Release checks should run `cargo-semver-checks` when installed. Pre-1.0
  semver still allows breaking changes in minor releases, but semver output
  must be reviewed before publishing.
- Experimental semantic APIs may change while APRS coverage matures, but changes
  must preserve raw-byte access and fail-closed parsing behavior.
- Diagnostic JSON from `ParsedPacket::to_json()` remains convenience output, not
  a compatibility-stable wire schema.

## 1.0 Readiness Criteria

The project should remove pre-1.0 caveats only after these gates are true:

- Stable-intent APIs below have at least one compatibility test and one
  documentation example.
- `cargo-semver-checks` runs in the release CI path and any breaking result is
  either fixed or explicitly called out in `CHANGELOG.md` before release.
- APRS semantic families in `docs/conformance.md` have fixture coverage for
  accepted, malformed, and policy-rejected cases where applicable.
- Parser and transport fuzz targets compile, and any discovered crash or
  panic is reduced to a deterministic regression test.
- Transport adapters document ownership of authentication, timeouts, bounding,
  retries, and byte-preservation responsibilities.

## Stable-Intent APIs

These APIs are intended to remain conceptually stable, though names and exact
types may still change before 1.0:

- `parse_packet`
- `parse_packet_with_options`
- `ParseOptions`
- `RawPacket`
- `ParsedPacket`
- `ParseError::code`
- `PolicyRejection::code`
- `Engine`
- `Policy`
- `LineTransport`
- `PacketSource`
- `PacketSink`
- `TransportErrorCode::code`
- `DEFAULT_TRANSPORT_READ_LIMIT`
- `read_all_with_limit`

## Experimental APIs

These APIs may change as APRS semantic coverage matures:

- `AprsData`
- semantic field structs such as `Position`, `Weather`, `Telemetry`,
  `TelemetryMetadata`, `Nmea`, `MicE`, and `ThirdParty`
- typed interpretation helper methods
- `DataTypeIdentifier`

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
