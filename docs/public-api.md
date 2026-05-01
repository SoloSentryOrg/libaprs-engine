# Public API Boundary

## BLUF

- `libaprs-engine` has reached `2.0.0`; the core API below is the
  semver-protected public contract.
- The stable boundary is byte-first: callers pass `&[u8]`, successful parses
  retain exact raw bytes, and malformed packet shape fails closed.
- Parser, policy, engine, counters, packet accessors, and shared transport
  contracts are the API surface downstream applications should build on now.
- APRS semantic variants are usable. Field-level semantic expansion should be
  additive within the `2.x` release line.
- Internal parser helpers, decoder helpers, diagnostic internals, and module
  layout are not part of the public compatibility contract.

## Stable API

These APIs are intended to remain source-compatible through the `2.x` release
line unless a secure code review finds a safety issue that requires a breaking
change:

- `parse_packet(input: &[u8]) -> Result<ParsedPacket, ParseError>`
- `parse_packet_with_options(input: &[u8], options: ParseOptions)`
- `ParseOptions`, `DEFAULT_PARSE_OPTIONS`, `MAX_PACKET_LEN`, and
  `EVENT_RAW_BYTE_LIMIT`
- `RawPacket::as_bytes`
- `ParsedPacket` accessors:
  - `raw`
  - `source`
  - `path`
  - `destination`
  - `digipeaters`
  - `path_components`
  - `payload`
  - `data_type_identifier`
  - `information`
  - `aprs_data`, with the semantic enum details governed by the evolving API
    section below until the APRS semantics batch is complete
  - `summary`
  - `to_diagnostic`, behind the `serde` feature, as the preferred structured
    packet diagnostic API
- `ParseError` variants and `ParseError::code`
- `ParseError::diagnostic`
- `Policy`, `Policy::strict`, `Policy::permissive`, and `Policy::evaluate`
- `PolicyDecision`, `PolicyRejection`, `PolicyRejection::code`, and
  `PolicyRejection::diagnostic`
- `Engine`, `Engine::new`, `Engine::process`, `Engine::process_event`,
  `Engine::process_packets`, `Engine::process_source`, and `Engine::counters`
- `EngineResult`
- `EngineEvent`, `EngineEventKind`, `AcceptedPacketEvent`,
  `PolicyRejectedPacketEvent`, `MalformedPacketEvent`, and
  `TransportFailureEvent`
- `Counters`
- `PacketSummary`, with additive fields allowed in minor releases
- `DiagnosticLayer`, `ErrorDiagnostic`, `SupportStatus`, `SupportItem`,
  `TransportSupport`, `SupportMatrix`, and `support_matrix`
- Existing `DataTypeIdentifier` variants and `DataTypeIdentifier::name`
- `LineTransport`, including bounded `packets_with_limit`
- `PacketSource` and `PacketSink`
- `TransportErrorCode`, `TransportErrorCode::code`, and
  `TransportErrorCode::diagnostic`
- `encoder` module packet-construction helpers:
  - `EncodeError` and `EncodeError::code`
  - `encode_packet`
  - `encode_status`
  - `encode_uncompressed_position` and `UncompressedPositionEncoding`
  - `encode_message`
  - `encode_ack` and `encode_reject`
  - `encode_bulletin` and `encode_announcement`
  - `encode_telemetry`
  - `encode_telemetry_metadata` and `TelemetryMetadataEncodingKind`
  - `encode_object` and `ObjectEncoding`
  - `encode_item` and `ItemEncoding`
- `service` module runtime-neutral helpers:
  - `DuplicateDecision` and `DuplicateWindow`
  - `RateLimitDecision` and `PacketRateBudget`
  - `SemanticFamily` and `SemanticBlocklist`
- `aprs-transport-tcp::TcpReadOptions` and
  `read_packet_lines_from_tcp_addr_with_options`
- `aprs-transport-aprs-is` profile helpers:
  - `AprsIsFilter`
  - `AprsIsProfileError`
  - `AprsIsQConstruct` and `AprsIsQConstructKind`
  - `AprsIsLogin::profile_line`
  - `validate_aprs_is_callsign`
  - `validate_aprs_is_filter`
  - `q_construct_from_tnc2`
- `DEFAULT_TRANSPORT_READ_LIMIT`
- `read_all_with_limit`
- `oversized_input_error`

## Compatibility Tests

`crates/libaprs-engine/tests/api_compat.rs` is the public API tripwire. It
compiles and exercises documented integration patterns for:

- parser entry points and parse options
- raw-byte preservation and field accessors
- stable parse error and policy rejection codes
- structured parser, policy, and transport diagnostics
- stable observability events and optional metrics helpers
- encoder and service-toolkit helper APIs
- structured serde diagnostics as the preferred stable-integration alternative
  to ad hoc JSON contracts
- TCP read-option builder methods used by transport integrations
- engine, policy, counters, and engine result flow
- data type identifier names
- line transport and shared source/sink traits
- semantic helpers documented in `docs/api.md`

Any release must pass these tests before publication.

## Experimental Or Evolving API

The semantic API is intentionally visible so applications can inspect APRS data
today. Minor releases may add fields, add variants, or tighten malformed
semantic classification for:

- `AprsData`
- semantic structs such as `Position`, `Object`, `Item`, `Weather`,
  `Telemetry`, `TelemetryMetadata`, `Nmea`, `MicE`, and `ThirdParty`
- typed helper methods that decode optional values from preserved bytes

The invariant that must not change is byte preservation: semantic views must
continue to reference or preserve original packet bytes and must not require
valid UTF-8.

## Internal API

The following are implementation details and must remain out of the compatibility
contract:

- parser helper functions
- coordinate decoder helpers
- Mic-E decoder helpers
- weather and telemetry field parsing helpers
- diagnostic module internals
- transport module internals other than the exported items listed above
- workspace crate layout outside documented crate names

Do not expose these helpers just to simplify tests. Add public methods only when
they represent a stable integration need.

## Semver Guidance

- Breaking changes to stable APIs require a major version bump, a changelog
  entry, compatibility test updates, and secure review.
- Patch releases must not intentionally break stable APIs.
- Minor releases should keep stable APIs source-compatible.
- Experimental semantic additions should be additive where possible. If a
  semantic change is breaking, the release notes must explain the byte-level
  behavior change and migration path.
- Release candidates must run `cargo-semver-checks` when available, then review
  the output before publishing.
