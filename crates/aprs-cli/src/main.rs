use std::env;
use std::fs;
use std::io::{self, Read};
use std::process::ExitCode;

use libaprs_engine::{Engine, EngineResult, Policy};

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<ExitCode, String> {
    let mut args = env::args().skip(1);
    let mode = args.next();
    let path = args.next();

    let json = mode.as_deref() == Some("--json");
    let input_path = if json { path } else { mode };
    let input = read_input(input_path.as_deref())?;
    let mut engine = Engine::new(Policy::default());
    let mut rejected = false;

    for line in input.lines() {
        if line.is_empty() {
            continue;
        }

        match engine.process(line.as_bytes()) {
            EngineResult::Accepted { packet } if json => println!("{}", packet.to_json()),
            EngineResult::Accepted { packet } => println!(
                "accepted source={} destination={} semantic={}",
                lossy(packet.source()),
                lossy(packet.destination()),
                packet.aprs_data().kind_name()
            ),
            EngineResult::Rejected { reason, .. } => {
                rejected = true;
                println!("rejected reason={reason:?}");
            }
            EngineResult::ParseError(error) => {
                rejected = true;
                println!("malformed error={error:?}");
            }
        }
    }

    let counters = engine.counters();
    eprintln!(
        "accepted={} rejected={} malformed={}",
        counters.accepted, counters.rejected, counters.malformed
    );

    Ok(if rejected {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}

fn read_input(path: Option<&str>) -> Result<String, String> {
    match path {
        Some(path) => {
            fs::read_to_string(path).map_err(|err| format!("failed to read {path}: {err}"))
        }
        None => {
            let mut input = String::new();
            io::stdin()
                .read_to_string(&mut input)
                .map_err(|err| format!("failed to read stdin: {err}"))?;
            Ok(input)
        }
    }
}

fn lossy(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}
