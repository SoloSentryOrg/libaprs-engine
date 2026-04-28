use std::fs;

use aprs_transport_corpus::read_corpus_packet_lines;
use libaprs_engine::{Engine, EngineResult, Policy};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let corpus_dir = std::env::temp_dir().join(format!(
        "libaprs-engine-corpus-example-{}",
        std::process::id()
    ));
    fs::create_dir_all(&corpus_dir)?;
    fs::write(corpus_dir.join("status.aprs"), b"N0CALL>APRS:>corpus\n")?;

    let mut engine = Engine::new(Policy::strict());
    let results = read_corpus_packet_lines(&corpus_dir)?
        .into_iter()
        .map(|packet| engine.process(&packet))
        .collect::<Vec<_>>();

    fs::remove_dir_all(&corpus_dir)?;

    assert!(matches!(
        results.as_slice(),
        [EngineResult::Accepted { .. }]
    ));
    Ok(())
}
