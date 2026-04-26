use std::env;
use std::fs;
use std::io::{self, Read};
use std::process::ExitCode;

use libaprs_engine::{Engine, EngineResult, LineTransport, Policy};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct CliOptions {
    json: bool,
    permissive: bool,
    explain: bool,
    summary: bool,
    fail_on: FailOn,
    filter: Option<String>,
    input_path: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum FailOn {
    None,
    Malformed,
    #[default]
    Rejected,
}

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
    let options = parse_args(env::args().skip(1))?;
    let input = read_input(options.input_path.as_deref())?;
    let policy = if options.permissive {
        Policy::permissive()
    } else {
        Policy::default()
    };
    let mut engine = Engine::new(policy);
    let mut rejected = false;
    let mut malformed = false;

    for line in LineTransport::new(&input).packets() {
        match engine.process(line) {
            EngineResult::Accepted { packet }
                if !matches_filter(&options, packet.aprs_data().kind_name()) => {}
            EngineResult::Accepted { packet } if options.json => println!("{}", packet.to_json()),
            EngineResult::Accepted { packet } => println!(
                "accepted source={} destination={} semantic={}",
                lossy(packet.source()),
                lossy(packet.destination()),
                packet.aprs_data().kind_name()
            ),
            EngineResult::Rejected { reason, .. } => {
                rejected = true;
                if options.explain {
                    println!("rejected reason={reason:?} code={}", reason.code());
                } else {
                    println!("rejected reason={reason:?}");
                }
            }
            EngineResult::ParseError(error) => {
                malformed = true;
                if options.explain {
                    println!("malformed error={error:?} code={}", error.code());
                } else {
                    println!("malformed error={error:?}");
                }
            }
        }
    }

    let counters = engine.counters();
    if options.summary {
        println!(
            "summary accepted={} rejected={} malformed={}",
            counters.accepted, counters.rejected, counters.malformed
        );
    }
    eprintln!(
        "accepted={} rejected={} malformed={}",
        counters.accepted, counters.rejected, counters.malformed
    );

    Ok(if should_fail(options.fail_on, rejected, malformed) {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    })
}

fn parse_args(args: impl IntoIterator<Item = String>) -> Result<CliOptions, String> {
    let mut options = CliOptions::default();
    let mut args = args.into_iter();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--json" => options.json = true,
            "--permissive" => options.permissive = true,
            "--explain" => options.explain = true,
            "--summary" => options.summary = true,
            "--filter" => {
                let filter = args
                    .next()
                    .ok_or_else(|| "--filter requires a semantic kind".to_string())?;
                options.filter = Some(filter);
            }
            "--fail-on" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--fail-on requires none, malformed, or rejected".to_string())?;
                options.fail_on = parse_fail_on(&value)?;
            }
            "--help" | "-h" => return Err(usage()),
            _ if arg.starts_with('-') => return Err(format!("unknown option: {arg}\n{}", usage())),
            _ => {
                if options.input_path.replace(arg).is_some() {
                    return Err(format!("multiple input paths supplied\n{}", usage()));
                }
            }
        }
    }

    Ok(options)
}

fn parse_fail_on(value: &str) -> Result<FailOn, String> {
    match value {
        "none" => Ok(FailOn::None),
        "malformed" => Ok(FailOn::Malformed),
        "rejected" => Ok(FailOn::Rejected),
        _ => Err(format!("invalid --fail-on value: {value}\n{}", usage())),
    }
}

fn should_fail(fail_on: FailOn, rejected: bool, malformed: bool) -> bool {
    match fail_on {
        FailOn::None => false,
        FailOn::Malformed => malformed,
        FailOn::Rejected => rejected || malformed,
    }
}

fn matches_filter(options: &CliOptions, semantic: &str) -> bool {
    options
        .filter
        .as_deref()
        .map_or(true, |filter| filter == semantic)
}

fn read_input(path: Option<&str>) -> Result<Vec<u8>, String> {
    match path {
        Some(path) => fs::read(path).map_err(|err| format!("failed to read {path}: {err}")),
        None => {
            let mut input = Vec::new();
            io::stdin()
                .read_to_end(&mut input)
                .map_err(|err| format!("failed to read stdin: {err}"))?;
            Ok(input)
        }
    }
}

fn lossy(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn usage() -> String {
    "usage: aprs-cli [--json] [--permissive] [--explain] [--summary] [--filter SEMANTIC] [--fail-on none|malformed|rejected] [PATH]".to_string()
}
