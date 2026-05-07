# Security Model

![libaprs-engine documentation header](assets/brand/docs-header.svg)

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

See [Threat Model](threat-model.md) for the per-crate untrusted boundaries,
primary abuse cases, and required controls.

For the current release-candidate audit evidence, see
[v3.0.0-rc.1 Security Audit Summary](security-audit-v3.0.0-rc.1.md).

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

Transport adapters also enforce packet or frame limits where the boundary is
known before parsing. `LineTransport::packets_with_limit` fails closed before
owned packet copies are allocated; reader-backed file, TCP, serial, APRS-IS,
HTTP, corpus, file-watch, async, MQTT, UDP, KISS, and AX.25 helpers expose or
use bounded variants for untrusted packet/frame input. Applications still own
socket timeouts, cancellation, queue depth, retries, authentication, and TLS.

Encoder helpers are not a transmit policy. They validate conservative packet
shape and return owned bytes only; callers still own destination selection,
authorization, rate limiting, logging, and transport transmission.

Service toolkit helpers are also deliberately runtime-neutral. Duplicate
suppression, packet-rate budgets, and semantic-family blocklists keep all
storage, clocks, queues, and network behavior application-owned.

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

The encoder module uses the same conservative address shape before emitting
bytes. Lowercase address metadata fails before packet construction so callers do
not accidentally transmit bytes the parser would reject.

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
for packet in libaprs_engine::LineTransport::new(&bytes)
    .packets_with_limit(libaprs_engine::MAX_PACKET_LEN)?
{
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

The Rust CI release-script job installs pinned versions of `cargo-audit` and
`cargo-deny` before running `scripts/verify-release.sh`, so pull requests and
main-branch updates exercise the same dependency-policy checks used by the local
release gate. The separate Security workflow remains scheduled and path-filtered
for dependency-focused monitoring.

The dependency policy is intentionally strict for the current dependency graph:
only MIT-licensed third-party crates are allowed, duplicate crate versions are
denied, wildcard dependencies are denied, and unknown registries or Git sources
are denied. Reintroduce additional allowed licenses only when a dependency
requires them and the license review is recorded.

## Fuzz Corpus Hygiene

Fuzz corpus inputs are release evidence, not scratch space. Keep them small,
sanitized, and safe to publish. `scripts/check-fuzz-corpus.sh` rejects hidden
files, common fuzzer artifact names, temporary logs, and corpus files larger
than `LIBAPRS_MAX_FUZZ_CORPUS_BYTES` bytes, defaulting to 4096 bytes.

When a fuzz finding is found, minimize the input, remove private station or
operator data, add a deterministic regression test, then add the minimized input
to `fuzz/corpus/` only if it remains useful for future fuzzing.

## Operational Recommendations

- Log or count parse failures without echoing untrusted bytes into unsafe sinks.
- Keep raw packet bytes for audit and replay.
- Enforce an upper bound on transport buffers before calling the engine in
  long-running services.
- Prefer shared `PacketSource` and `PacketSink` adapters when composing
  transports, so packet batches and sink behavior remain byte-oriented.
- Use `ParsedPacket::to_diagnostic()`, `PacketSummary`, `EngineEvent`, or an
  application-owned schema for external APIs.
- Run tests and clippy before accepting parser or policy changes.
- Run `cargo audit` or `cargo deny check` before releases when the tools are
  available.
