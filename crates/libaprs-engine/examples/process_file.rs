use libaprs_engine::{Engine, EngineResult, LineTransport, Policy};

fn main() -> std::io::Result<()> {
    let Some(path) = std::env::args().nth(1) else {
        eprintln!("usage: process_file <packets.aprs>");
        std::process::exit(2);
    };

    let input = std::fs::read(path)?;
    let mut engine = Engine::new(Policy::strict());

    for packet_bytes in LineTransport::new(&input).packets() {
        match engine.process(packet_bytes) {
            EngineResult::Accepted { packet } => println!("{}", packet.to_json()),
            EngineResult::Rejected { reason, .. } => eprintln!("rejected: {}", reason.code()),
            EngineResult::ParseError(error) => eprintln!("malformed: {}", error.code()),
        }
    }

    Ok(())
}
