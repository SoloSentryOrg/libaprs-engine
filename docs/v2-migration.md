# `v2.0.0` Migration Plan

## BLUF

- No public API is removed during the `1.x` line.
- `v2.0.0` should include only breaking changes justified by downstream
  evidence, semver checks, and secure review.
- The current `1.x` guidance is to build on byte-oriented parser, policy,
  engine, event, and transport contracts.
- APIs listed below are soft-deprecated for new stable integrations only when a
  safer documented alternative exists.
- The current breaking-change decision record does not approve publishing a
  `v2.0.0-rc.1` until at least one break is justified by downstream evidence.
- The final `v2.0.0` release must be promoted from a tested release candidate.

See [`v2.0.0` Breaking-Change Decision Record](v2-breaking-changes.md) for the
current go/no-go status.

## Soft Deprecations For New Integrations

These APIs remain supported in `1.x`. The project avoids Rust
`#[deprecated]` attributes for now because warnings would break the local
`-D warnings` release gate and create churn for current users.

| API or pattern | `1.x` status | Preferred `1.x` alternative | Possible `v2.0.0` change |
| --- | --- | --- | --- |
| `ParsedPacket::to_json()` for external contracts | Supported diagnostic convenience. | Use `ParsedPacket::to_diagnostic()`, `serde_support::PacketDiagnostic`, `PacketSummary`, `EngineEvent`, or an application-owned schema. | Rename or replace with an explicitly diagnostic API only if downstream confusion persists after the additive replacement path. |
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

These are possible candidates only. None is approved for a release candidate
until the decision record marks it justified. Each one still requires
release-candidate evidence before it can ship.

- Rename or replace `ParsedPacket::to_json()` with a diagnostic-only name or a
  versioned diagnostic type.
- Split stable packet-envelope APIs from evolving semantic interpretation APIs
  if downstream users need a smaller compatibility surface.
- Refine transport traits around runtime-neutral receive loops only if repeated
  transport integrations need the same abstraction.
- Stabilize event or diagnostic serialization under explicit schema versions
  only if application-owned schemas prove insufficient.
- Rename broad parse, policy, or transport diagnostic names only when the old
  names cause real migration risk or ambiguity.

## Release Candidate Gates

Before publishing `v2.0.0-rc.1`:

- update this migration plan with exact breaking changes,
- run `cargo-semver-checks` and record intentional breaks,
- add compatibility tests for the replacement APIs,
- run downstream smoke against the release-candidate crates,
- complete secure code review with no open findings,
- pass local release gates and remote CI/security workflows, and
- record release evidence in `docs/release.md`.
