# libaprs-engine v2

APRS engine focused on protocol-first parsing and full APRS semantic coverage.

This workspace currently provides the core crate, `libaprs-engine`, with packet
primitives, semantic classification, and a conservative codec boundary:

- Preserve raw packet bytes exactly.
- Parse untrusted input as bytes, not strings.
- Fail closed on empty, oversized, malformed, or non-AX.25-like packet shapes.
- Expose source, destination, digipeater path components, and payload as byte
  views backed by the preserved raw packet.
- Classify the APRS data type identifier from the first payload byte.
- Parse APRS semantic families with byte-preserving field views: status,
  uncompressed and timestamped position, compressed position, message,
  bulletin, announcement, acknowledgement, reject, object, item, weather,
  telemetry, query, capability, NMEA, Mic-E, Maidenhead locator, user-defined
  data, third-party traffic, malformed data, and unsupported data.
- Interpret typed values for coordinates, compressed coordinates, telemetry
  values and bits, common weather fields, and Mic-E destination-derived status
  and latitude digits when the raw bytes are valid.
- Avoid network, async, serialization, and transport dependencies in v1.

The parser validates the minimal
`source>path:payload` shape plus conservative source/path address components:
uppercase ASCII callsigns of 1-6 letters or digits, optional SSID values from
0-15, and optional trailing `*` repeated markers on path components only.

## Semantic Scope

Full APRS semantics are in scope. APRS101 packet families are represented by
byte-preserving semantic variants while preserving these invariants:

- Raw bytes are always retained.
- Metadata parsing fails closed.
- Payload semantic parsers never panic on untrusted or invalid UTF-8 bytes.
- Typed interpretation returns optional values when a field cannot be decoded
  without weakening byte preservation.
- Unsupported, unknown, and malformed APRS data formats remain explicitly
  represented instead of being guessed.
- Dependencies stay minimal unless a protocol feature clearly justifies one.

## Verification

Run:

```sh
cargo test
cargo metadata --no-deps --format-version 1
cargo clippy --all-targets --all-features -- -D warnings
```
