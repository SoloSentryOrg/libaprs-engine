use std::fs::File;

use libaprs_engine::{
    read_all_with_limit, Engine, EngineResult, LineTransport, Policy, DEFAULT_TRANSPORT_READ_LIMIT,
};

fn main() -> std::io::Result<()> {
    let Some(path) = std::env::args().nth(1) else {
        eprintln!("usage: process_file <packets.aprs>");
        std::process::exit(2);
    };

    let input = read_all_with_limit(File::open(path)?, DEFAULT_TRANSPORT_READ_LIMIT)?;
    let mut engine = Engine::new(Policy::strict());

    for packet_bytes in LineTransport::new(&input).packets() {
        match engine.process(packet_bytes) {
            EngineResult::Accepted { packet } => {
                let summary = packet.summary();
                println!(
                    "accepted source={} destination={} semantic={}",
                    String::from_utf8_lossy(summary.source),
                    String::from_utf8_lossy(summary.destination),
                    summary.semantic
                );
            }
            EngineResult::Rejected { reason, .. } => eprintln!("rejected: {}", reason.code()),
            EngineResult::ParseError(error) => eprintln!("malformed: {}", error.code()),
        }
    }

    Ok(())
}
