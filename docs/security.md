# Security Model

`libaprs-engine` is designed around untrusted packet input. The project follows
OWASP-aligned principles for validation, least surprise at trust boundaries, and
fail-closed behavior.

## Security Invariants

- Packet input is bytes, not text.
- Accepted packets retain the original raw bytes exactly.
- Malformed packet shape fails closed with `ParseError`.
- No partial packet is returned when codec validation fails.
- Payload bytes are opaque and may be invalid UTF-8.
- Parser behavior must not depend on lossy conversion.
- Semantic decoders must not panic on untrusted bytes.
- Unknown or unsupported semantics must be represented explicitly.
- Policy may reject valid packets, but policy may not repair invalid codec input.
- Dependencies are intentionally minimal.

## Trust Boundaries

External bytes enter through a transport or the CLI and must be passed to the
codec unchanged. The codec validates only the minimal packet envelope and
conservative address metadata needed to establish safe structure.

The current trusted boundary is:

```text
untrusted bytes -> LineTransport -> parse_packet -> ParsedPacket -> AprsData -> Policy -> EngineResult
```

## Address Validation

The codec accepts a conservative `source>path:payload` packet envelope:

- source callsign: uppercase ASCII letters or digits, 1-6 bytes
- optional source SSID: `-0` through `-15`
- path: one or more comma-separated address components
- first path component: destination
- later path components: digipeaters
- repeated marker: optional trailing `*` on path components only
- payload: at least one byte
- maximum packet size: `MAX_PACKET_LEN`

This is intentionally conservative. Broader transport-specific decoding belongs
outside the codec boundary and should hand validated packet bytes into this
crate.

## UTF-8 Handling

Do not use `String`, `str::lines`, or text normalization before parsing packet
bytes. Use `Vec<u8>`, `&[u8]`, and `LineTransport`.

Safe:

```rust
let bytes = std::fs::read("packets.aprs")?;
for packet in libaprs_engine::LineTransport::new(&bytes).packets() {
    let result = libaprs_engine::parse_packet(packet);
}
```

Unsafe for this project:

```rust
let text = std::fs::read_to_string("packets.aprs")?;
for packet in text.lines() {
    let result = libaprs_engine::parse_packet(packet.as_bytes());
}
```

The second example rejects or changes inputs before the codec sees them.

## Policy Layer

`Policy::strict()` is the default posture. It rejects unsupported semantics,
malformed semantic variants, and excessive path components after codec
validation. `Policy::permissive()` is useful for exploration and corpus
collection, but applications should prefer strict policy for production
ingestion unless they have a clear review path for unsupported data.

## Dependency Policy

The current library crate uses the Rust standard library only. Add dependencies
only when they reduce security or correctness risk more than they increase
supply-chain and maintenance risk.

## Operational Recommendations

- Log or count parse failures without echoing untrusted bytes into unsafe sinks.
- Keep raw packet bytes for audit and replay.
- Enforce an upper bound on transport buffers before calling the engine in
  long-running services.
- Treat `to_json()` as diagnostics; define your own stable schema for external
  APIs.
- Run tests and clippy before accepting parser or policy changes.
