# Public API Boundary

## BLUF

- `libaprs-engine` is still pre-1.0, but the core API below is the candidate
  public contract for `1.0.0`.
- The stable boundary is byte-first: callers pass `&[u8]`, successful parses
  retain exact raw bytes, and malformed packet shape fails closed.
- Parser, policy, engine, counters, packet accessors, and shared transport
  contracts are the API surface downstream applications should build on now.
- APRS semantic variants are usable, but field-level semantic expansion remains
  subject to the APRS semantics batch before `1.0.0`.
- Internal parser helpers, decoder helpers, diagnostic internals, and module
  layout are not part of the public compatibility contract.

## Candidate 1.0 Stable API

These APIs are intended to remain source-compatible through `1.0.0` unless a
secure code review finds a safety issue that requires a breaking change:

- `parse_packet(input: &[u8]) -> Result<ParsedPacket, ParseError>`
- `parse_packet_with_options(input: &[u8], options: ParseOptions)`
- `ParseOptions`, `DEFAULT_PARSE_OPTIONS`, and `MAX_PACKET_LEN`
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
  - `to_json`, with diagnostic JSON treated as convenience output rather than
    a stable wire schema
- `ParseError` variants and `ParseError::code`
- `Policy`, `Policy::strict`, `Policy::permissive`, and `Policy::evaluate`
- `PolicyDecision`, `PolicyRejection`, and `PolicyRejection::code`
- `Engine`, `Engine::new`, `Engine::process`, `Engine::process_packets`,
  `Engine::process_source`, and `Engine::counters`
- `EngineResult`
- `Counters`
- `PacketSummary`, with additive fields allowed before `1.0.0`
- Existing `DataTypeIdentifier` variants and `DataTypeIdentifier::name`
- `LineTransport`
- `PacketSource` and `PacketSink`
- `TransportErrorCode` and `TransportErrorCode::code`
- `DEFAULT_TRANSPORT_READ_LIMIT`
- `read_all_with_limit`
- `oversized_input_error`

## Compatibility Tests

`crates/libaprs-engine/tests/api_compat.rs` is the public API tripwire. It
compiles and exercises documented integration patterns for:

- parser entry points and parse options
- raw-byte preservation and field accessors
- stable parse error and policy rejection codes
- engine, policy, counters, and engine result flow
- data type identifier names
- line transport and shared source/sink traits
- semantic helpers documented in `docs/api.md`

Any `1.0.0` release candidate must pass these tests before publication.

## Experimental Or Evolving API

The semantic API is intentionally visible so applications can inspect APRS data
today. The stable promise before the APRS semantics batch is byte preservation,
not final field completeness. Before `1.0.0`, the APRS semantics batch may
still add fields, add variants, or tighten malformed semantic classification
for:

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

- Before `1.0.0`, breaking changes to the candidate stable API require a minor
  version bump, a changelog entry, compatibility test updates, and secure review.
- Patch releases should not intentionally break candidate stable APIs.
- After `1.0.0`, candidate stable APIs become semver-protected public APIs.
- Experimental semantic additions should be additive where possible. If a
  semantic change is breaking, the release notes must explain the byte-level
  behavior change and migration path.
- Release candidates must run `cargo-semver-checks` when available, then review
  the output before publishing.
