#![forbid(unsafe_code)]

use libaprs_engine::{Engine, EngineEvent, LineTransport, Policy, MAX_PACKET_LEN};

fn main() -> Result<(), std::io::Error> {
    let input = b"N0CALL>APRS:>service online\nN1CALL>APRS:~opaque\nbad packet\n";
    let mut engine = Engine::new(Policy::strict());

    let packets = LineTransport::new(input).packets_with_limit(MAX_PACKET_LEN)?;

    for packet in packets {
        match engine.process_event(packet) {
            EngineEvent::Accepted(event) => {
                println!(
                    "event={} semantic={}",
                    event.kind().code(),
                    event.packet.summary().semantic
                );
            }
            EngineEvent::Rejected(event) => {
                println!(
                    "event={} code={} remediation={}",
                    event.kind().code(),
                    event.diagnostic.code,
                    event.diagnostic.remediation
                );
            }
            EngineEvent::Malformed(event) => {
                println!(
                    "event={} code={} raw_len={} raw_truncated={} remediation={}",
                    event.kind().code(),
                    event.diagnostic.code,
                    event.raw.len(),
                    event.raw_truncated,
                    event.diagnostic.remediation
                );
            }
        }
    }

    let counters = engine.counters();
    println!(
        "summary accepted={} rejected={} malformed={}",
        counters.accepted, counters.rejected, counters.malformed
    );

    Ok(())
}
