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

Transport helper crates must also enforce bounded reads before splitting or
framing packets. The shared default limit is
`libaprs_engine::DEFAULT_TRANSPORT_READ_LIMIT`. Callers that need a different
limit should use the explicit `*_with_limit` APIs and select the smallest value
that fits their source. Oversized batches fail closed with
`transport.oversized_input`.

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
let file = std::fs::File::open("packets.aprs")?;
let bytes = libaprs_engine::read_all_with_limit(
    file,
    libaprs_engine::DEFAULT_TRANSPORT_READ_LIMIT,
)?;
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
validation. NMEA checksum mismatches are reported by the codec and can be
rejected by enabling `Policy::reject_invalid_nmea_checksum`; the codec still
preserves raw bytes and does not repair or normalize the sentence.
`Policy::permissive()` is useful for exploration and corpus collection, but
applications should prefer strict policy for production ingestion unless they
have a clear review path for unsupported data.

## Dependency Policy

The default library path has no third-party runtime dependency. Optional
features may add dependencies only when they are explicitly enabled. The current
optional `serde` feature provides owned diagnostic structures and is disabled by
default.

The repository includes `deny.toml` to make dependency expectations explicit:
known vulnerable or yanked crates are denied, unmaintained advisories are
warnings, wildcard dependencies are denied, and unknown registries or Git
sources are denied. This is a local/release gate when `cargo-deny` is installed;
it does not add runtime dependencies.

## Operational Recommendations

- Log or count parse failures without echoing untrusted bytes into unsafe sinks.
- Keep raw packet bytes for audit and replay.
- Enforce an upper bound on transport buffers before calling the engine in
  long-running services.
- Prefer shared `PacketSource` and `PacketSink` adapters when composing
  transports, so packet batches and sink behavior remain byte-oriented.
- Treat `to_json()` as diagnostics; define your own stable schema for external
  APIs.
- Run tests and clippy before accepting parser or policy changes.
- Run `cargo audit` or `cargo deny check` before releases when the tools are
  available.
