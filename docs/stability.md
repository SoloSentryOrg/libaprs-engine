# Stability

This project is pre-1.0. Public APIs are usable, but compatibility is not yet
guaranteed under semantic versioning.

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

## Experimental APIs

These APIs may change as APRS semantic coverage matures:

- `AprsData`
- semantic field structs such as `Position`, `Weather`, `Telemetry`, and `MicE`
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
