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
- transport adapter crates: optional boundaries for TCP, APRS-IS, KISS,
  serial-like readers, UDP datagrams, HTTP bodies, append-only files, MQTT
  payloads, AX.25 UI frames, corpus replay, in-process channels, and
  runtime-neutral async splitting.
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
- **Transports:** supply bytes from external systems. Transport crates may
  frame, split, or copy bytes for their source protocol, but they do not parse
  APRS semantics or lossy-convert payloads before the codec boundary. Shared
  `PacketSource` and `PacketSink` traits define the minimal adapter contract,
  and helper readers enforce explicit byte limits before allocating full
  batches.
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
rejects, objects, items, weather, telemetry, telemetry metadata, queries,
capabilities, NMEA, Mic-E, Maidenhead locator, user-defined data, and
third-party traffic.

Semantic parsing must remain byte-preserving and fail closed. Unknown,
unsupported, and malformed data must be represented explicitly rather than
silently coerced into another type. Typed interpretation currently covers
decimal coordinates, compressed coordinates, telemetry sequence/value/bit
fields, telemetry metadata fields, weather fields, NMEA checksum inspection,
Mic-E coordinates/speed/course when decodable, and explicit nested third-party
parsing. Transports, policy rules, and CLI behavior remain separate layers from
protocol semantics.

Semantic malformed handling is conservative. Empty weather reports and
third-party bodies that do not pass the nested packet codec envelope are
represented as malformed semantic payloads so strict policy can reject them.

## Verification And Release

The repository includes conformance fixtures, malformed packet fixtures,
deterministic byte-fuzz tests, CLI tests, examples, a benchmark target, release
metadata, and a release checklist. GitHub Actions can run the same Cargo
verification commands when account policy permits jobs to start.

## Optional Transports

Transport adapters live outside `libaprs-engine` so the parser core remains
network-free and focused on bytes, codec validation, policy, and semantics.
`aprs-transport-file` handles file/stdin-style packet sources,
`aprs-transport-tcp` handles blocking TCP or reader-backed packet sources,
`aprs-transport-aprs-is` handles APRS-IS login framing plus APRS-IS comment
filtering, `aprs-transport-kiss` handles KISS byte stuffing, and the remaining
transport crates cover serial-like readers, UDP datagrams, HTTP bodies,
append-only packet files, MQTT payloads, AX.25 UI frames, corpus replay,
in-process channels, and runtime-neutral async splitting. These adapters
preserve packet bytes and hand APRS packet bytes to the codec unchanged. File,
TCP, serial-like, APRS-IS, corpus, and file-watch helpers expose bounded default
reads plus explicit `*_with_limit` variants for application-specific limits.
