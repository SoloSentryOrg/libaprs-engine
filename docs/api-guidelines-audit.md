# Rust API Guidelines Audit

![libaprs-engine documentation header](assets/brand/docs-header.svg)

## BLUF

- Current `3.0.0-rc.1` release-candidate preparation remains source-compatible
  with `2.6.0`.
- New encoder, service-toolkit, and APRS-IS profile APIs are documented and
  covered by `api_compat` or crate-level integration tests.
- Public helpers use explicit result types, stable machine-readable codes, and
  caller-owned storage or transport choices.
- No new runtime, async, network, serialization, or storage dependency is added
  by these APIs.
- No evidence currently justifies an intentional `v3.0.0-rc.1` public API
  breaking change.

## Audit Scope

This audit covers public APIs added or expanded after `v2.0.0`:

- `libaprs_engine::encoder`
- `libaprs_engine::service`
- `aprs_transport_aprs_is` profile helpers
- compile-tested examples for encoding, service helpers, APRS-IS profile use,
  and KISS TCP/serial-style framing

## API Guideline Findings

| Area | Decision | Evidence |
| --- | --- | --- |
| Naming | Module and type names describe behavior without hiding policy. | Encoders construct bytes; service helpers do not imply service orchestration. |
| Error handling | Fallible APIs return typed errors with stable `code()` strings. | `EncodeError`, `AprsIsProfileError`, and existing transport errors avoid string-only control flow. |
| Ownership | APIs return owned bytes only where construction requires ownership. | Encoders return `Vec<u8>`; parser views still borrow preserved bytes. |
| Extensibility | New public enums are scoped to new modules. | Existing public enums were not widened for profile or encoder behavior. |
| Dependencies | No new dependencies are required. | Helpers use only the standard library and existing workspace crates. |
| Security posture | Inputs are validated before output or policy decisions. | Encoder, profile, and service tests cover invalid addresses, invalid filters, duplicates, and rate budgets. |

## Compatibility Evidence

- `crates/libaprs-engine/tests/api_compat.rs` covers encoder and service helper
  public APIs.
- `crates/libaprs-engine/tests/codec.rs` covers encoder round trips through the
  parser.
- `crates/libaprs-engine/tests/engine.rs` covers service helper behavior.
- `crates/aprs-transport-aprs-is/tests/aprs_is_transport.rs` covers APRS-IS
  profile behavior.
- `crates/aprs-transport-corpus/tests/fixtures/interoperability/` stores safe
  APRS-IS, KISS/TNC2, and LoRa APRS fixture lines with tests for the expected
  parser and transport-diagnostic boundaries.
- `cargo test --examples` compiles the new user-facing examples.

## Future Major Decision

Keep `v3.0.0` no-intentional-break unless internal downstream evidence shows
that additive modules are no longer enough for safe integrations.
