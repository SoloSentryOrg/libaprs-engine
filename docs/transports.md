# Transport Adapters

Transport crates keep I/O and protocol framing outside `libaprs-engine`. They
return packet bytes or frame payload bytes that callers pass into the core
parser, preserving the protocol-first boundary.

## Integration Rules

- Treat transport input as untrusted bytes.
- Bound reads before buffering packet data.
- Bound packet or frame lengths before allocating owned packet copies.
- Do not convert packet bytes to `String` before parsing.
- Do not trim, lowercase, normalize, or otherwise rewrite APRS packet bytes.
- Apply `parse_packet` or `Engine::process` after transport-specific framing.

## Adapter Matrix

| Crate | Boundary | Primary use | Security note |
| --- | --- | --- | --- |
| `aprs-transport-file` | Newline-separated byte buffers and files | Offline logs, stdin-style files | Bounded path helpers reject oversized batches and packet lines |
| `aprs-transport-tcp` | Blocking `Read` and TCP address helpers | TCP-connected packet streams | Reader helpers reject oversized batches and packet lines; connection timeouts stay application-owned |
| `aprs-transport-aprs-is` | APRS-IS login line and comment filtering | APRS-IS clients | Server comment lines are filtered before parsing and packet lines are bounded |
| `aprs-transport-kiss` | KISS frame encoding and decoding | TNC, serial, or TCP KISS streams | Invalid escapes and oversized decoded payloads fail closed |
| `aprs-transport-serial` | Serial-like byte readers | TNC serial pipelines | Reader helpers reject oversized batches and packet lines; serial configuration stays application-owned |
| `aprs-transport-udp` | UDP datagram receive helper | Datagram packet input | Datagram length is bounded; caller owns socket binding and timeouts |
| `aprs-transport-http` | HTTP body byte splitting | Webhook or upload ingestion | Body and packet-line limits are enforced by bounded helpers |
| `aprs-transport-file-watch` | Append-only file offsets | Packet log tailing | Appended byte batches and packet lines are bounded |
| `aprs-transport-mqtt` | MQTT topic matching and payload copies | MQTT message bridges | Bounded payload-copy helpers are available; broker session and auth stay application-owned |
| `aprs-transport-ax25` | AX.25 UI frame decoding | Link-layer frame ingest | Oversized UI frames fail closed before payload extraction |
| `aprs-transport-corpus` | Fixture/corpus replay | Tests and corpus validation | Per-file byte limits and packet-line limits are enforced |
| `aprs-transport-channel` | In-process channels | Worker pipelines and tests | Channel ownership controls backpressure |
| `aprs-transport-async` | Runtime-neutral async splitting | Async readers without runtime coupling | Runtime, timeouts, and cancellation stay caller-owned |

## File Or Stdin-Like Input

```rust
use aprs_transport_file::try_read_packet_lines;
use libaprs_engine::Engine;

fn main() -> std::io::Result<()> {
    let mut engine = Engine::default();

    for packet_bytes in try_read_packet_lines(b"N0CALL>APRS:>file\n")? {
        let _result = engine.process(&packet_bytes);
    }

    Ok(())
}
```

Use `read_packet_lines` only for already trusted and bounded byte slices. Use
`try_read_packet_lines` or path/reader `*_with_limit` helpers for external
input.

## APRS-IS Reader

```rust
use aprs_transport_aprs_is::{read_packet_lines_from_reader, AprsIsLogin};
use libaprs_engine::parse_packet;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let login = AprsIsLogin {
        callsign: "N0CALL",
        passcode: -1,
        software: "libaprs-engine 1.0.0-rc.1",
        filter: Some("r/49/-72/50"),
    };
    assert!(login.line()?.ends_with("\r\n"));

    let input = std::io::Cursor::new(b"# banner\r\nN0CALL>APRS:>hello\n");
    for packet_bytes in read_packet_lines_from_reader(input)? {
        let packet = parse_packet(&packet_bytes).map_err(|error| error.code())?;
        println!("{}", packet.aprs_data().kind_name());
    }

    Ok(())
}
```

## KISS Frame Pipeline

```rust
use aprs_transport_kiss::{decode_frames, encode_data_frame};
use libaprs_engine::parse_packet;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let encoded = encode_data_frame(0, b"N0CALL>APRS:>kiss").map_err(|error| error.code())?;

    for frame in decode_frames(&encoded).map_err(|error| error.code())? {
        let packet = parse_packet(&frame.payload).map_err(|error| error.code())?;
        println!("{}", packet.summary().semantic);
    }

    Ok(())
}
```

## AX.25 UI Frame Pipeline

```rust
use aprs_transport_ax25::decode_ax25_ui_frame;
use libaprs_engine::parse_packet;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let frame_bytes = build_ax25_ui_frame("N0CALL", "APRS", b">ax25");
    let frame = decode_ax25_ui_frame(&frame_bytes).map_err(|error| error.code())?;
    let mut packet_bytes = Vec::new();
    packet_bytes.extend_from_slice(&frame.source);
    packet_bytes.push(b'>');
    packet_bytes.extend_from_slice(&frame.destination);
    packet_bytes.push(b':');
    packet_bytes.extend_from_slice(&frame.information);

    let packet = parse_packet(&packet_bytes).map_err(|error| error.code())?;

    println!("{}", packet.aprs_data().kind_name());
    Ok(())
}

fn build_ax25_ui_frame(source: &str, destination: &str, information: &[u8]) -> Vec<u8> {
    let mut frame = Vec::new();
    frame.extend_from_slice(&encode_ax25_addr(destination, false));
    frame.extend_from_slice(&encode_ax25_addr(source, true));
    frame.push(0x03);
    frame.push(0xf0);
    frame.extend_from_slice(information);
    frame
}

fn encode_ax25_addr(callsign: &str, last: bool) -> [u8; 7] {
    let mut out = [b' ' << 1; 7];
    for (index, byte) in callsign.bytes().take(6).enumerate() {
        out[index] = byte << 1;
    }
    out[6] = 0x60 | u8::from(last);
    out
}
```

## Operational Guidance

- Put authentication, TLS, broker credentials, socket options, and retry policy
  in the application layer, not in parser code.
- Configure read deadlines, cancellation, and bounded worker queues at the
  application layer. Transport helpers do not spawn workers or own queues.
- Treat `transport.oversized_input`, `kiss_oversized_frame`, and
  `ax25_oversized_frame` as fail-closed security telemetry, not recoverable
  parser warnings.
- Track transport byte counts and parser counters separately so malformed input
  spikes are visible.
- Add deterministic regression fixtures for any transport failure found in
  production before adding new parser behavior.
- Keep private callsigns, precise locations, and operator data out of checked-in
  corpora unless they are already public and safe to redistribute.

## Compile-Tested Examples

The repository includes transport cookbook examples that compile under
`cargo test --examples`:

- `crates/aprs-transport-aprs-is/examples/reader.rs`
- `crates/aprs-transport-kiss/examples/frame_pipeline.rs`
- `crates/aprs-transport-udp/examples/datagram_ingest.rs`
- `crates/aprs-transport-corpus/examples/replay.rs`
