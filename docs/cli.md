# CLI Guide

`aprs-cli` is a small packet inspector for newline-separated APRS packets. It is
designed to exercise the same byte-preserving parser and policy engine used by
the library.

## Run From The Workspace

Text output:

```sh
cargo run -p aprs-cli -- packets.aprs
```

JSON diagnostic output:

```sh
cargo run -p aprs-cli -- --json packets.aprs
```

Read from stdin:

```sh
cat packets.aprs | cargo run -p aprs-cli -- --json
```

## Input Rules

- Input is read as raw bytes.
- Packets are separated by LF.
- A trailing CR before LF is stripped.
- Empty lines are ignored.
- Packet bytes are passed to `LineTransport` and then `Engine`.
- Invalid UTF-8 payload bytes are preserved and do not prevent parsing.

## Output

Default text output:

```text
accepted source=N0CALL destination=APRS semantic=status
```

JSON output:

```json
{"raw":"N0CALL>APRS:>hello","source":"N0CALL","destination":"APRS","path":"APRS","payload":">hello","data_type":"status","semantic":"status"}
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
