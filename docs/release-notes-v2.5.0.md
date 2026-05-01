# libaprs-engine v2.5.0

`v2.5.0` publishes the additive APRS conformance, interoperability, encoding,
service-toolkit, and API-readiness work completed after `v2.0.0`.

## Highlights

- Adds compressed-position weather access, including object/item embedded
  compressed weather helper coverage.
- Adds APRS-IS profile helpers for uppercase login callsign validation,
  conservative filter validation, and q-construct diagnostics over raw TNC2
  bytes.
- Adds owned-byte APRS encoders for generic payloads, status, uncompressed
  position, messages, acknowledgements, rejections, bulletins, announcements,
  telemetry, telemetry metadata, objects, and items.
- Adds runtime-neutral service helpers for duplicate suppression, caller-reset
  packet-rate budgets, and semantic-family blocklists.
- Adds safe interoperability fixtures for APRS-IS q-construct, KISS/TNC2, and
  LoRa APRS examples.
- Adds API-guidelines, supply-chain, and `v3.0.0` decision records for the next
  major-version planning gate.

## Compatibility

This is an additive `2.x` release. No `2.0.0` public APIs are intentionally
removed or broken.

## Migration

Existing `2.0.0` users can update dependency versions to `2.5.0`. Applications
that handle APRS-IS q constructs should continue treating q path components as
transport metadata rather than weakening the core parser's conservative
AX.25-like address boundary.
