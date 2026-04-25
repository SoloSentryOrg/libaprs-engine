# Architecture

The engine is protocol-first and byte-preserving. Every boundary that accepts
external packet data treats it as untrusted bytes and fails closed when the
packet is malformed.

## Pipeline

`Types -> Codec -> Policy -> Engine -> Transports -> CLI`

## Contracts

- **Types:** own protocol data structures and preserve raw packet bytes without
  trimming, normalization, lowercasing, or lossy UTF-8 conversion.
- **Codec:** accepts `&[u8]`, validates minimal packet shape, and returns either
  a structured packet view or a closed error. It does not partially accept
  malformed packets.
- **Policy:** will apply operational constraints after codec validation. Policy
  must not repair malformed codec input.
- **Engine:** will orchestrate validated packets and policy decisions. It should
  not parse raw transport bytes directly.
- **Transports:** will supply bytes from external systems. Transport adapters are
  untrusted input boundaries and must pass bytes to the codec unchanged.
- **CLI:** will expose engine behavior to users without weakening parser or
  policy failure modes.

## Trust Boundaries

- Packet bytes crossing from transports or CLI input into the codec are
  untrusted.
- Parsed packet fields are borrowed views into preserved raw bytes.
- Payload bytes are opaque and may be invalid UTF-8.
- Any malformed packet shape is rejected before policy or engine handling.

## Current Scope

The first crate implements only the minimal `source>path:payload` codec
boundary. Full APRS semantics, transports, policy rules, and CLI behavior are
out of scope for this skeleton.
