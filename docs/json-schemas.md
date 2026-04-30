# JSON Schemas

## BLUF

- JSON output is for diagnostics and operations, not for replacing the Rust API.
- `support-matrix --json` is versioned with `schema_version`.
- `ParsedPacket::to_json()` and `aprs-cli --json` share the accepted-packet
  diagnostic shape.
- Rejected and malformed CLI records remain text output; use the Rust event API
  for stable event structs.
- Consumers should reject unsupported schema versions instead of guessing.

## Accepted Packet Diagnostic

Produced by `ParsedPacket::to_json()` and `aprs-cli --json` for accepted
packets.

```json
{
  "type": "object",
  "additionalProperties": false,
  "required": [
    "raw",
    "source",
    "destination",
    "path",
    "payload",
    "data_type",
    "semantic"
  ],
  "properties": {
    "raw": { "type": "string", "description": "Lossless escaped byte display" },
    "source": { "type": "string", "description": "Escaped source address bytes" },
    "destination": { "type": "string", "description": "Escaped destination address bytes" },
    "path": { "type": "string", "description": "Escaped destination/path bytes" },
    "payload": { "type": "string", "description": "Escaped payload bytes including data type byte" },
    "data_type": { "type": "string", "description": "Stable APRS data type identifier name" },
    "semantic": { "type": "string", "description": "Stable APRS semantic family name" }
  }
}
```

The byte fields are diagnostic strings escaped by the crate. They are not a
replacement for `ParsedPacket::raw().as_bytes()` when exact byte recovery is
required.

## Support Matrix

Produced by `aprs-cli support-matrix --json`.

```json
{
  "type": "object",
  "additionalProperties": false,
  "required": [
    "schema_version",
    "semantic_families",
    "transport_adapters",
    "diagnostic_layers"
  ],
  "properties": {
    "schema_version": { "type": "integer", "const": 1 },
    "semantic_families": {
      "type": "array",
      "items": {
        "type": "object",
        "additionalProperties": false,
        "required": ["kind", "status", "notes"],
        "properties": {
          "kind": { "type": "string" },
          "status": { "enum": ["supported", "partial", "unsupported"] },
          "notes": { "type": "string" }
        }
      }
    },
    "transport_adapters": {
      "type": "array",
      "items": {
        "type": "object",
        "additionalProperties": false,
        "required": ["crate", "boundary", "status", "notes"],
        "properties": {
          "crate": { "type": "string" },
          "boundary": { "type": "string" },
          "status": { "enum": ["supported", "partial", "unsupported"] },
          "notes": { "type": "string" }
        }
      }
    },
    "diagnostic_layers": {
      "type": "array",
      "items": {
        "type": "object",
        "additionalProperties": false,
        "required": ["code"],
        "properties": {
          "code": { "enum": ["parse", "policy", "transport"] }
        }
      }
    }
  }
}
```

## Event API Mapping

Use Rust event structs for service integrations instead of parsing CLI text:

- `EngineEvent::Accepted` maps to event kind `accepted`.
- `EngineEvent::Rejected` maps to event kind `policy_rejected`.
- `EngineEvent::Malformed` maps to event kind `malformed`.
- `TransportFailureEvent` maps to event kind `transport_failure`.

Event structs retain raw packet bytes for accepted and rejected packets through
`ParsedPacket`. Malformed events expose `MalformedPacketEvent::raw` capped at
`EVENT_RAW_BYTE_LIMIT`; check `raw_truncated` before treating that field as the
complete malformed input.
