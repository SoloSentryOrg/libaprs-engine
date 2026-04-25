# libaprs-engine v2

APRS engine focused on protocol-first parsing and full APRS semantic coverage.

This workspace currently provides the core crate, `libaprs-engine`, with packet
primitives, semantic classification, and a conservative codec boundary:

- Preserve raw packet bytes exactly.
- Parse untrusted input as bytes, not strings.
- Fail closed on empty, oversized, malformed, or non-AX.25-like packet shapes.
- Expose source, destination, digipeater path components, and payload as byte
  views backed by the preserved raw packet.
- Classify the APRS data type identifier from the first payload byte while
  leaving the remaining information field opaque.
- Parse initial APRS semantic families: status, uncompressed position, message,
  object, item, and unsupported data.
- Avoid network, async, serialization, and transport dependencies in v1.

The parser currently validates the minimal
`source>path:payload` shape plus conservative source/path address components:
uppercase ASCII callsigns of 1-6 letters or digits, optional SSID values from
0-15, and optional trailing `*` repeated markers on path components only. It is
expanding toward full APRS Protocol Reference 1.0.1 semantics.

## Semantic Scope

Full APRS semantics are in scope. Implementation should cover APRS101 packet
families incrementally while preserving these invariants:

- Raw bytes are always retained.
- Metadata parsing fails closed.
- Payload semantic parsers never panic on untrusted or invalid UTF-8 bytes.
- Unsupported or not-yet-implemented APRS data formats remain explicitly
  represented instead of being guessed.
- Dependencies stay minimal unless a protocol feature clearly justifies one.

## Verification

Run:

```sh
cargo test
cargo metadata --no-deps --format-version 1
cargo clippy --all-targets --all-features -- -D warnings
```
