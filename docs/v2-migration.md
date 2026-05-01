# `v2.0.0` Migration Plan

## BLUF

- `v2.0.0-rc.1` removes the library `ParsedPacket::to_json()` method.
- Use byte-oriented parser, policy, engine, event, summary, and serde
  diagnostic contracts instead.
- Parser, policy, transport, and APRS semantic behavior are unchanged in this
  release candidate.
- The final `v2.0.0` release promotes the tested release-candidate line after
  review time and clean gates.

See [`v2.0.0` Breaking-Change Decision Record](v2-breaking-changes.md) for the
current go/no-go status.

## Breaking Change In `v2.0.0-rc.1`

`ParsedPacket::to_json()` is removed from the library API. It was diagnostic
convenience output, but its name made it easy to treat as a stable wire schema.
The CLI keeps `--json` for operator diagnostics and now includes
`schema_version`.

Before:

```rust
let packet = libaprs_engine::parse_packet(b"N0CALL>APRS:>hello")?;
let json = packet.to_json();
println!("{json}");
```

After, for structured diagnostics:

```rust
let packet = libaprs_engine::parse_packet(b"N0CALL>APRS:>hello")?;
let diagnostic = packet.to_diagnostic();
assert_eq!(diagnostic.schema_version, 1);
assert_eq!(diagnostic.semantic, "status");
```

`to_diagnostic()` is available when the `serde` feature is enabled:

```toml
libaprs-engine = { version = "2.0.0", features = ["serde"] }
```

After, for application wire formats, define and version an application-owned
schema using packet accessors, `PacketSummary`, or `EngineEvent`.

## Deferred Candidates

| API or pattern | `1.x` status | Preferred `1.x` alternative | Possible `v2.0.0` change |
| --- | --- | --- | --- |
| Unbounded in-memory transport helpers for untrusted input | Supported only for already bounded byte slices. | Use `try_read_packet_lines`, `packets_with_limit`, reader/path `*_with_limit`, or adapter options such as `TcpReadOptions`. | Make bounded helper names the primary surface and move convenience helpers to examples. |
| Treating `AprsData` as a fully stable semantic schema | Supported but evolving. | Use raw byte access, `PacketSummary`, and optional helper methods defensively. | Split stable packet envelope types from richer semantic interpretation types. |
| Application-owned event JSON inferred from current debug output | Unsupported as a compatibility contract. | Serialize application-owned structs or serde diagnostics under your own schema version. | Add first-class stable event serialization only if downstream users need it. |
| Broad transport receive-loop assumptions | Not provided in `1.x`. | Compose `PacketSource`, `PacketSink`, adapter options, and application retry/backpressure logic. | Add runtime-neutral transport traits only if repeated downstream evidence supports them. |

## Intended Stable `1.x` Migration Path

Parser integrations should prefer:

```rust
use libaprs_engine::{parse_packet, Policy};

let packet = parse_packet(b"N0CALL>APRS:>hello")?;
let decision = Policy::strict().evaluate(&packet, &packet.aprs_data());
```

Service integrations should prefer:

```rust
use libaprs_engine::{Engine, EngineEvent};

let mut engine = Engine::default();
match engine.process_event(b"N0CALL>APRS:>hello") {
    EngineEvent::Accepted(event) => {
        let summary = event.packet.summary();
        let semantic = summary.semantic;
    }
    EngineEvent::Rejected(event) => {
        let code = event.reason.code();
    }
    EngineEvent::Malformed(event) => {
        let code = event.error.code();
        let truncated = event.raw_truncated;
    }
    EngineEvent::TransportFailure(event) => {
        let code = event.code.code();
    }
}
```

Serde diagnostic integrations should prefer:

```rust
use libaprs_engine::parse_packet;

let packet = parse_packet(b"N0CALL>APRS:>hello")?;
let diagnostic = packet.to_diagnostic();
let raw_bytes = diagnostic.raw;
```

Transport integrations should prefer bounded byte APIs:

```rust
use libaprs_engine::{LineTransport, MAX_PACKET_LEN};

let packets = LineTransport::new(b"N0CALL>APRS:>hello\n")
    .packets_with_limit(MAX_PACKET_LEN)?;
```

## Final `v2.0.0` Breaking-Change Candidate List

The approved `v2.0.0-rc.1` breaking change is:

- Remove `ParsedPacket::to_json()` from the library API after adding
  `ParsedPacket::to_diagnostic()` and `PacketDiagnostic::schema_version`.

These remain possible candidates only. None is approved until the decision
record marks it justified:

- Split stable packet-envelope APIs from evolving semantic interpretation APIs
  if downstream users need a smaller compatibility surface.
- Refine transport traits around runtime-neutral receive loops only if repeated
  transport integrations need the same abstraction.
- Stabilize event or diagnostic serialization under explicit schema versions
  only if application-owned schemas prove insufficient.
- Rename broad parse, policy, or transport diagnostic names only when the old
  names cause real migration risk or ambiguity.

## Final Release Gates

Before publishing final `v2.0.0`:

- update this migration plan with exact breaking changes,
- run `cargo-semver-checks` and record intentional breaks,
- add compatibility tests for the replacement APIs,
- run downstream smoke against the final crates after publication,
- complete secure code review with no open findings,
- pass local release gates and remote CI/security workflows, and
- record release evidence in `docs/release.md`.
