# Architecture

The engine is protocol-first and byte-preserving. Every boundary that accepts
external packet data treats it as untrusted bytes and fails closed when the
packet is malformed.

## Pipeline

`Types -> Codec -> Policy -> Engine -> Transports -> CLI`

The workspace keeps the core parser crate separate from adapters:

- `libaprs-engine`: codec, APRS semantic views, diagnostics, policy, engine,
  and the byte-oriented line splitter.
- `aprs-transport-file`: optional file adapter crate that reads packet files as
  bytes and delegates splitting to the core line transport.
- `aprs-cli`: inspection binary that exercises the engine from stdin or files.

## Contracts

- **Types:** own protocol data structures and preserve raw packet bytes without
  trimming, normalization, lowercasing, or lossy UTF-8 conversion. Structured
  fields are byte views into the preserved packet. Typed interpretation helpers
  expose decoded values only when the original bytes validate.
- **Codec:** accepts `&[u8]`, validates minimal packet shape and conservative
  source/path address bytes, and returns either a structured packet view or a
  closed error. It does not partially accept malformed packets.
- **Policy:** applies operational constraints after codec validation. Policy
  does not repair malformed codec input.
- **Engine:** orchestrates codec, semantics, policy decisions, and counters. It
  does not parse raw transport bytes directly.
- **Transports:** supply bytes from external systems. Current transport support
  is line-oriented file/stdin input and a separate file adapter crate. Both pass
  packet bytes to the codec unchanged.
- **CLI:** exposes engine behavior for packet inspection with text and JSON
  diagnostics without weakening parser or policy failure modes.

## Trust Boundaries

- Packet bytes crossing from transports or CLI input into the codec are
  untrusted.
- Parsed packet fields are borrowed views into preserved raw bytes.
- The first path component is the destination. Later path components are
  digipeaters.
- Payload bytes are opaque and may be invalid UTF-8.
- The first payload byte is exposed as the APRS data type identifier. Remaining
  information-field bytes stay opaque.
- Source and path bytes must use conservative address components: uppercase
  ASCII callsigns of 1-6 letters or digits, optional SSID values from 0-15, and
  optional trailing `*` repeated markers on path components only.
- Any malformed packet shape is rejected before policy or engine handling.

## Semantic Scope

Full APRS Protocol Reference 1.0.1 semantics are in scope. The implementation
represents these packet families: position, timestamped position, compressed
position, status, messages, bulletins, announcements, acknowledgements,
rejects, objects, items, weather, telemetry, queries, capabilities, NMEA,
Mic-E, Maidenhead locator, user-defined data, and third-party traffic.

Semantic parsing must remain byte-preserving and fail closed. Unknown,
unsupported, and malformed data must be represented explicitly rather than
silently coerced into another type. Typed interpretation currently covers
decimal coordinates, compressed coordinates, telemetry sequence/value/bit
fields, common weather fields, and Mic-E destination-derived status and latitude
digits. Transports, policy rules, and CLI behavior remain separate layers from
protocol semantics.

## Verification And Release

The repository includes conformance fixtures, malformed packet fixtures,
deterministic byte-fuzz tests, CLI tests, examples, a benchmark target, release
metadata, and a release checklist. GitHub Actions can run the same Cargo
verification commands when account policy permits jobs to start.
