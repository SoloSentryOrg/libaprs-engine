# Threat Model

![libaprs-engine documentation header](assets/brand/docs-header.svg)

This project treats APRS packets, transport frames, files, network payloads,
and command-line input as untrusted bytes. The core security goals are raw-byte
preservation, bounded resource use, fail-closed malformed input handling, and
no lossy text conversion before the codec boundary.

## Shared Assumptions

- Attackers can send malformed, oversized, high-volume, invalid UTF-8, or
  semantically unusual packet bytes.
- Transport adapters may receive partial frames, overlong records, repeated
  delimiters, or stalled streams.
- Library consumers may choose permissive policy for corpus collection, but
  production ingestion should use strict policy unless a review process exists.
- The parser must not require network, async, or serialization dependencies.
- Crates.io dependencies, advisory state, and release tooling are supply-chain
  inputs and must be verified before publication.

## Shared Controls

- `MAX_PACKET_LEN` bounds core packet parsing.
- `DEFAULT_TRANSPORT_READ_LIMIT` bounds shared reader-backed transport helpers.
- Transport crates expose per-record or per-frame limits before codec parsing.
- `ParseError` rejects malformed packet shape without returning partial
  packets.
- Semantic decoders represent unsupported and malformed data explicitly.
- Counters use saturating accounting.
- Release verification runs formatting, tests, clippy, docs, package
  validation, MSRV checks, semver checks, audit, deny, fuzz compile checks, and
  fuzz corpus hygiene when tools are installed.

## Crate Threats And Boundaries

| Crate | Untrusted boundary | Primary threats | Required controls |
| --- | --- | --- | --- |
| `libaprs-engine` | `parse_packet`, `LineTransport`, policy and engine entry points | malformed bytes, invalid UTF-8, oversized packets, semantic confusion, counter overflow | byte slices only, raw-byte retention, `MAX_PACKET_LEN`, explicit malformed/unsupported semantics, saturating counters |
| `aprs-cli` | stdin, files, command arguments | non-UTF-8 packet loss, oversized files, misleading exit status | byte reads, `LineTransport`, bounded file reads, explicit fail-on modes |
| `aprs-transport-file` | packet files and reader-backed file input | oversized files, overlong lines, invalid UTF-8 | bounded reads, per-packet limits, byte-preserving records |
| `aprs-transport-file-watch` | appended file bytes | unbounded append growth, partial records | appended-byte limits, packet-line limits, caller-owned polling policy |
| `aprs-transport-corpus` | corpus directories and files | private data leakage, oversized corpus files, unstable ordering | bounded file reads, stable ordering, fuzz corpus guard for regression seeds |
| `aprs-transport-tcp` | TCP streams and generic readers | stalled streams, overlong lines, retry storms | caller-owned timeouts/reconnects, bounded reads, packet-line limits |
| `aprs-transport-aprs-is` | APRS-IS server lines and login filters | line injection, server comments, oversized lines | CRLF-safe login construction, comment filtering, line limits |
| `aprs-transport-serial` | serial readers | partial records, invalid bytes, oversized batches | caller-owned serial configuration, bounded reads, packet-line limits |
| `aprs-transport-http` | HTTP body bytes | oversized request bodies, malformed line framing | body-size limits, packet-line limits, no text normalization |
| `aprs-transport-udp` | datagrams | oversized datagrams, packet truncation assumptions | datagram-size limits, byte-preserving handoff |
| `aprs-transport-kiss` | KISS frames | unclosed frames, invalid escaping, oversized frames | frame-size limits, escape validation, explicit frame errors |
| `aprs-transport-ax25` | AX.25 UI frames | invalid frame metadata, oversized frames, non-APRS payloads | frame-size limits, UI frame validation, APRS payload extraction only |
| `aprs-transport-mqtt` | MQTT topics and payload bytes | topic confusion, oversized payloads | topic matching helpers, payload-size limits |
| `aprs-transport-async` | async byte streams | unbounded buffers, runtime-specific cancellation gaps | runtime-neutral helpers, caller-owned timeout/cancellation/backpressure |
| `aprs-transport-channel` | in-process packet queues | unbounded queue growth, ownership confusion | caller-owned queue bounds, owned byte vectors at the channel boundary |

## Out Of Scope

- Authentication, TLS, reconnect policy, queue depth, service-level rate
  limiting, and authorization are owned by consuming applications.
- APRS-IS passcode validation and network session security are not part of the
  core parser.
- Fuzz corpora must not contain private station data, credentials, or production
  incident payloads unless they have been minimized and sanitized.

## Release Evidence

Each release should record:

- secure code review result
- local `scripts/verify-release.sh` result
- remote CI and security workflow result
- advisory and dependency policy result
- fuzz compile and fuzz corpus hygiene result
- any new minimized regression corpus entries and the issue they prevent
