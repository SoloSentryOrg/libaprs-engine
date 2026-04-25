# libaprs-engine v2

Early APRS engine skeleton focused on protocol-first parsing boundaries.

This workspace currently provides the first core crate, `libaprs-engine`, with
minimal packet primitives and a conservative codec boundary:

- Preserve raw packet bytes exactly.
- Parse untrusted input as bytes, not strings.
- Fail closed on empty, oversized, malformed, or non-AX.25-like packet shapes.
- Avoid network, async, serialization, and transport dependencies in v1.

The initial parser intentionally validates only the minimal
`source>path:payload` shape plus conservative source/path address components:
uppercase ASCII callsigns of 1-6 letters or digits, optional SSID values from
0-15, and optional trailing `*` repeated markers on path components only. It is
not a complete APRS protocol implementation yet.

## Verification

Run:

```sh
cargo test
cargo metadata --no-deps --format-version 1
cargo clippy --all-targets --all-features -- -D warnings
```
