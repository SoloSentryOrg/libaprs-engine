#![forbid(unsafe_code)]

use libaprs_engine::{Engine, EngineResult, LineTransport, Policy, MAX_PACKET_LEN};

fn main() -> Result<(), std::io::Error> {
    let input = b"N0CALL>APRS:>service online\nN1CALL>APRS:~opaque\nbad packet\n";
    let mut engine = Engine::new(Policy::strict());

    let packets = LineTransport::new(input).packets_with_limit(MAX_PACKET_LEN)?;

    for packet in packets {
        match engine.process(packet) {
            EngineResult::Accepted { packet } => {
                println!("accepted semantic={}", packet.summary().semantic);
            }
            EngineResult::Rejected { reason, .. } => {
                let diagnostic = reason.diagnostic();
                println!(
                    "rejected code={} remediation={}",
                    diagnostic.code, diagnostic.remediation
                );
            }
            EngineResult::ParseError(error) => {
                let diagnostic = error.diagnostic();
                println!(
                    "malformed code={} remediation={}",
                    diagnostic.code, diagnostic.remediation
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
