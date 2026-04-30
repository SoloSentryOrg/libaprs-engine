# Operations Guide

## BLUF

- Treat all packet and transport bytes as untrusted input.
- Keep bounded reads enabled before calling the parser or engine.
- Use strict policy for production ingestion unless a documented collection
  workflow requires permissive mode.
- Emit stable diagnostic codes for parser, policy, and transport failures.
- Track accepted, rejected, and malformed counters separately.

## Deployment Pattern

Use the core crate as the byte-preserving validation boundary:

```text
transport bytes -> bounded transport helper -> LineTransport or adapter -> Engine -> application logic
```

The core parser does not own sockets, runtime tasks, authentication, reconnects,
or worker queues. Keep those concerns in application code or optional transport
crates so the codec boundary stays deterministic and easy to review.

## Safe Defaults

- Use `Policy::strict()` or `Policy::default()` for production paths.
- Keep `MAX_PACKET_LEN` as the packet-line limit unless an operator-reviewed
  deployment requires a different `ParseOptions` value.
- Use `DEFAULT_TRANSPORT_READ_LIMIT` or a lower application-owned limit for
  file, stream, HTTP body, or batch reads.
- Leave `Policy::permissive()` for corpus collection, debugging, or migration
  review. Do not use it to silently accept unknown production traffic.
- Preserve raw bytes in logs or evidence stores when investigating malformed
  input. Avoid UTF-8-only logging paths for packet payloads.

## Diagnostics

Structured diagnostics are available from:

- `ParseError::diagnostic()`
- `PolicyRejection::diagnostic()`
- `TransportErrorCode::diagnostic()`
- `Engine::process_event()` for accepted, rejected, and malformed packet events
- `TransportFailureEvent::from_code()` for transport-boundary events

Each diagnostic includes:

- `layer`: `parse`, `policy`, or `transport`
- `code`: stable fully-qualified code such as `parse.missing_separator`
- `name`: stable short name
- `description`: operator-facing explanation
- `remediation`: immediate handling guidance

Use the stable codes for alerts and dashboards. Treat descriptions and
remediation text as human-readable guidance that may expand in minor releases.

For long-running services, prefer `Engine::process_event()` when an integration
needs event-shaped telemetry. It preserves accepted and rejected packet bytes
through `ParsedPacket`, copies malformed raw input bytes into
`MalformedPacketEvent` up to `EVENT_RAW_BYTE_LIMIT`, reports truncation through
`raw_truncated`, and keeps `Engine::counters()` monotonic and saturating.

With the `metrics` feature, `libaprs_engine::metrics_support` exposes stable
counter names and a small `MetricsRecorder` trait. The module has no runtime
dependency and is intended to bridge counters into application-owned telemetry
libraries.

## Support Matrix

The CLI exposes a machine-readable support matrix:

```sh
aprs-cli support-matrix --json
```

Use it to inventory supported semantic families, optional transport adapter
crates, and diagnostic layers in deployment tooling. The output includes
`schema_version`; consumers should reject unsupported schema versions rather
than guessing.

## Logging

Recommended fields for each processed packet:

- source system or transport name
- event kind from `EngineEventKind::code()`
- diagnostic layer and code, when processing fails
- semantic kind, when accepted or policy-rejected
- accepted, rejected, and malformed counters
- byte counts at transport and parser boundaries

Avoid logging packet bytes as lossy UTF-8. If raw evidence is required, store
bytes in an application-owned representation that preserves non-UTF-8 values.

## Limits And Abuse Resistance

- Reject oversized transport batches before splitting into packet lines.
- Reject oversized packet lines before copying into owned buffers.
- Keep reconnect, retry, and backpressure policies outside `libaprs-engine`.
- Treat repeated malformed or policy-rejected spikes as potential abuse or
  upstream data corruption.
- Review any increase to byte limits as a security-relevant configuration
  change.

## Release Gates

Before deploying a new version:

- Run `scripts/verify-release.sh`.
- Confirm GitHub Actions passed for the release commit.
- Review `docs/release.md` for release evidence.
- Check `docs/conformance.md` for current protocol support and known gaps.
- Keep crates.io publication behind clean secure review, local gates, remote CI,
  and GitHub Release verification.
