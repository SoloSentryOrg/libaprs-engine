# Transport Common Layer Review

## BLUF

- Keep the shared `PacketSource` and `PacketSink` traits in `libaprs-engine`.
- Do not introduce a larger transport framework in the `1.x` line yet.
- Keep runtime, retry, reconnect, timeout, authentication, and backpressure
  ownership in application or adapter crates.
- Add narrow helper options when they reduce unsafe defaults, such as
  `TcpReadOptions`.
- Revisit a stronger common trait layer only after more downstream integration
  evidence exists.

## Current Shared Contract

The core crate exposes:

- `PacketSource`: reads a bounded batch of owned packet byte vectors.
- `PacketSink`: sends one packet byte slice without mutation or normalization.
- `LineTransport`: splits LF/CRLF packet byte buffers and enforces per-packet
  limits before owned copies.
- `read_all_with_limit`: bounds reader-backed byte batches before parsing.

This is enough for current adapters to share byte-preserving behavior without
forcing a runtime, network stack, or queue model into the parser crate.

## Decision

Do not add a richer common transport trait before `v2.0.0`.

Reasons:

- TCP, UDP, MQTT, KISS, AX.25, files, channels, and async readers have different
  failure and ownership models.
- A common async or reconnect trait would either choose a runtime or hide
  operational policy behind generic abstractions.
- The current parser boundary is easier to secure because transport helpers
  return bytes and the core engine remains deterministic.

## Additive `1.x` Direction

- Add adapter-specific options where they expose caller-owned limits and
  timeouts clearly.
- Keep options structs small, copyable where practical, and standard-library
  only unless a crate-specific dependency is unavoidable.
- Add integration tests for every new transport failure mode.
- Keep examples explicit about reconnect, retry, and backpressure being
  application behavior.

## `v2.0.0` Revisit Criteria

Reconsider the common layer if downstream users repeatedly need:

- a single trait for long-running receive loops,
- shared transport event types beyond `TransportFailureEvent`,
- common retry/backoff configuration,
- runtime-neutral cancellation hooks, or
- shared sink acknowledgements/backpressure semantics.

Any stronger common layer must preserve raw bytes, fail closed on transport
framing errors, and keep parser core network-free.
