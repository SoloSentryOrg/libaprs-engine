# `v3.0.0` Migration Plan

## BLUF

- No intentional public API break is approved for `v3.0.0-rc.1`.
- Existing `2.6.0` parser, policy, engine, transport, encoder, service, and
  diagnostic integrations should only need dependency-version updates for RC
  testing.
- Keep packet input byte-oriented, preserve accepted raw bytes, and continue to
  treat packet and transport input as untrusted.
- Use an exact dependency requirement when testing the release candidate so
  applications validate the same package line.
- Any semver-check, secure-review, CI, conformance, or downstream finding must
  be fixed forward or documented before final `v3.0.0`.

## Dependency Update

For release-candidate validation, pin the exact RC version:

```toml
[dependencies]
libaprs-engine = { version = "=3.0.0-rc.1", features = ["serde"] }
aprs-transport-file = "=3.0.0-rc.1"
aprs-transport-tcp = "=3.0.0-rc.1"
aprs-transport-aprs-is = "=3.0.0-rc.1"
aprs-transport-kiss = "=3.0.0-rc.1"
aprs-transport-serial = "=3.0.0-rc.1"
aprs-transport-udp = "=3.0.0-rc.1"
aprs-transport-http = "=3.0.0-rc.1"
aprs-transport-file-watch = "=3.0.0-rc.1"
aprs-transport-mqtt = "=3.0.0-rc.1"
aprs-transport-ax25 = "=3.0.0-rc.1"
aprs-transport-corpus = "=3.0.0-rc.1"
aprs-transport-channel = "=3.0.0-rc.1"
aprs-transport-async = "=3.0.0-rc.1"
```

Use normal semver requirements only after deciding whether to move from the RC
to final `3.0.0`.

## Source Migration

No source migration is expected from `2.6.0` to `3.0.0-rc.1`.

The stable integration path remains:

```rust
use libaprs_engine::{parse_packet, Policy};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let packet = parse_packet(b"N0CALL>APRS:>hello")?;
    let decision = Policy::strict().evaluate(&packet, &packet.aprs_data());
    println!("{} {}", packet.summary().semantic, decision.code());
    Ok(())
}
```

The stable diagnostic path remains:

```rust
use libaprs_engine::parse_packet;

fn main() -> Result<(), libaprs_engine::ParseError> {
    let packet = parse_packet(b"N0CALL>APRS:>hello")?;
    let diagnostic = packet.to_diagnostic();
    assert_eq!(diagnostic.schema_version, 1);
    Ok(())
}
```

## Validation

Before adopting the RC downstream:

- run the application test suite,
- run `cargo update -p libaprs-engine --precise 3.0.0-rc.1` where applicable,
- verify packet input still enters as bytes rather than text,
- verify malformed packet shape still fails closed, and
- report any compile, semver, or behavioral regression before final `3.0.0`.
