# CLI Guide

`aprs-cli` is a small packet inspector for newline-separated APRS packets. It is
designed to exercise the same byte-preserving parser and policy engine used by
the library.

## Run From The Workspace

Text output:

```sh
cargo run -p aprs-cli -- packets.aprs
cargo run -p aprs-cli -- parse packets.aprs
```

JSON diagnostic output:

```sh
cargo run -p aprs-cli -- --json packets.aprs
```

Filter accepted packets by semantic kind:

```sh
cargo run -p aprs-cli -- --filter status packets.aprs
```

Use permissive policy for inspection of unsupported or malformed semantic
payloads after codec validation:

```sh
cargo run -p aprs-cli -- --permissive packets.aprs
```

Read from stdin:

```sh
cat packets.aprs | cargo run -p aprs-cli -- --json
```

Validate input without printing each accepted packet:

```sh
cargo run -p aprs-cli -- validate packets.aprs
```

Print aggregate counters only:

```sh
cargo run -p aprs-cli -- stats packets.aprs
```

Explain rejections with stable diagnostic codes:

```sh
cargo run -p aprs-cli -- explain packets.aprs
```

Replay accepted packet bytes:

```sh
cargo run -p aprs-cli -- replay --permissive packets.aprs
```

Print the machine-readable support matrix:

```sh
cargo run -p aprs-cli -- support-matrix --json
```

## Input Rules

- Input is read as raw bytes.
- Packets are separated by LF.
- A trailing CR before LF is stripped.
- Empty lines are ignored.
- Packet bytes are passed to `LineTransport` and then `Engine`.
- Invalid UTF-8 payload bytes are preserved and do not prevent parsing.
- File and stdin input are bounded by `DEFAULT_TRANSPORT_READ_LIMIT`.
- Oversized CLI input fails with exit code `2` and the stable diagnostic
  `transport.oversized_input`.

## Options

- `parse`: default inspection command.
- `validate`: print validity and counters without accepted packet details.
- `stats`: print aggregate counters only.
- `explain`: print parse or policy diagnostic codes for non-accepted packets.
- `replay`: emit accepted raw packet bytes with LF separators.
- `support-matrix`: print supported semantic families, transport adapters, and
  diagnostic layers without reading packet input.
- `--json`: print compact diagnostic JSON for accepted packets.
- `--explain`: include stable parse or policy codes with malformed/rejected
  output.
- `--summary`: print final counters to stdout as well as stderr.
- `--filter SEMANTIC`: print only accepted packets whose semantic kind matches
  `SEMANTIC`, such as `status`, `position`, or `telemetry_metadata`.
- `--permissive`: allow unsupported and malformed semantic payloads through
  policy for inspection.
- `--fail-on none|malformed|rejected`: choose whether non-accepted packets
  produce a non-zero exit. `rejected` is the default and includes malformed
  packets.
- `PATH`: read packet bytes from a file path instead of stdin.

## Output

Default text output:

```text
accepted source=N0CALL destination=APRS semantic=status
```

JSON output:

```json
{"raw":"N0CALL>APRS:>hello","source":"N0CALL","destination":"APRS","path":"APRS","payload":">hello","data_type":"status","semantic":"status"}
```

The accepted-packet and support-matrix JSON shapes are documented in
[JSON Schemas](json-schemas.md). Treat accepted-packet JSON as diagnostic
output; use `ParsedPacket` or `EngineEvent` in Rust integrations when exact raw
bytes or stable event structs are required.

Support matrix JSON output:

```json
{"schema_version":1,"semantic_families":[{"kind":"status","status":"supported","notes":"status text bytes are preserved"}],"transport_adapters":[{"crate":"aprs-transport-file","boundary":"newline-separated files and stdin-style byte streams","status":"supported","notes":"bounded file and packet-line reads"}],"diagnostic_layers":[{"code":"parse"},{"code":"policy"},{"code":"transport"}]}
```

Rejected and malformed packets are printed to stdout:

```text
rejected reason=UnsupportedSemantics
malformed error=MissingSourcePathSeparator
```

Counters are printed to stderr:

```text
accepted=1 rejected=0 malformed=0
```

## Exit Codes

- `0`: all processed packets were accepted.
- `1`: at least one packet was rejected by policy or malformed at the codec
  boundary.
- `2`: CLI usage or I/O error.

## Notes For Integrators

- The CLI is an inspection tool, not a daemon.
- The JSON shape is a diagnostic convenience, not a stable external API.
- Use the library crate directly for long-running applications or custom
  transports.
