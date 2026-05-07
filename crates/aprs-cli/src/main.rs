#![forbid(unsafe_code)]

use std::env;
use std::fs::File;
use std::io::{self, Write};
use std::process::ExitCode;

use libaprs_engine::{
    read_all_with_limit, support_matrix, DiagnosticLayer, Engine, EngineResult, LineTransport,
    ParsedPacket, Policy, SupportMatrix, MAX_PACKET_LEN,
};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct CliOptions {
    json: bool,
    permissive: bool,
    explain: bool,
    summary: bool,
    command: CommandMode,
    fail_on: FailOn,
    filter: Option<String>,
    input_path: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum CommandMode {
    #[default]
    Parse,
    Validate,
    Stats,
    Explain,
    Replay,
    SupportMatrix,
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
    if options.command == CommandMode::SupportMatrix {
        print_support_matrix(options.json);
        return Ok(ExitCode::SUCCESS);
    }

    let input = read_input(options.input_path.as_deref())?;
    let policy = if options.permissive {
        Policy::permissive()
    } else {
        Policy::default()
    };
    let mut engine = Engine::new(policy);
    let mut rejected = false;
    let mut malformed = false;

    for line in LineTransport::new(&input)
        .packets_with_limit(MAX_PACKET_LEN)
        .map_err(|err| format!("failed to split packet lines: {err}"))?
    {
        match engine.process(line) {
            EngineResult::Accepted { packet }
                if !matches_filter(&options, packet.aprs_data().kind_name()) => {}
            EngineResult::Accepted { packet } => match options.command {
                CommandMode::Validate | CommandMode::Stats | CommandMode::SupportMatrix => {}
                CommandMode::Replay => {
                    io::stdout()
                        .write_all(packet.raw().as_bytes())
                        .map_err(|err| format!("failed to write packet: {err}"))?;
                    io::stdout()
                        .write_all(b"\n")
                        .map_err(|err| format!("failed to write newline: {err}"))?;
                }
                CommandMode::Parse | CommandMode::Explain if options.json => {
                    println!("{}", packet_json(&packet));
                }
                CommandMode::Parse | CommandMode::Explain => println!(
                    "accepted source={} destination={} semantic={}",
                    lossy(packet.source()),
                    lossy(packet.destination()),
                    packet.aprs_data().kind_name()
                ),
            },
            EngineResult::Rejected { reason, .. } => {
                rejected = true;
                if options.command == CommandMode::Explain || options.explain {
                    println!("rejected reason={reason:?} code={}", reason.code());
                } else if options.command != CommandMode::Validate
                    && options.command != CommandMode::Stats
                    && options.command != CommandMode::Replay
                    && options.command != CommandMode::SupportMatrix
                {
                    println!("rejected reason={reason:?}");
                }
            }
            EngineResult::ParseError(error) => {
                malformed = true;
                if options.command == CommandMode::Explain || options.explain {
                    println!("malformed error={error:?} code={}", error.code());
                } else if options.command != CommandMode::Validate
                    && options.command != CommandMode::Stats
                    && options.command != CommandMode::Replay
                    && options.command != CommandMode::SupportMatrix
                {
                    println!("malformed error={error:?}");
                }
            }
        }
    }

    let counters = engine.counters();
    if options.command == CommandMode::Validate {
        if rejected || malformed {
            println!(
                "invalid accepted={} rejected={} malformed={}",
                counters.accepted, counters.rejected, counters.malformed
            );
        } else {
            println!(
                "valid accepted={} rejected={} malformed={}",
                counters.accepted, counters.rejected, counters.malformed
            );
        }
    }
    if options.summary || options.command == CommandMode::Stats {
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
            "parse" => options.command = CommandMode::Parse,
            "validate" => options.command = CommandMode::Validate,
            "stats" => options.command = CommandMode::Stats,
            "explain" => {
                options.command = CommandMode::Explain;
                options.explain = true;
            }
            "replay" => options.command = CommandMode::Replay,
            "support-matrix" => options.command = CommandMode::SupportMatrix,
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
            _ if arg.starts_with('-') => {
                return Err(format!(
                    "unknown option: {}\n{}",
                    diagnostic_value(&arg),
                    usage()
                ));
            }
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
        _ => Err(format!(
            "invalid --fail-on value: {}\n{}",
            diagnostic_value(value),
            usage()
        )),
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
        Some(path) => {
            let display_path = diagnostic_value(path);
            read_all_with_limit(
                File::open(path).map_err(|err| format!("failed to open {display_path}: {err}"))?,
                libaprs_engine::DEFAULT_TRANSPORT_READ_LIMIT,
            )
            .map_err(|err| format!("failed to read {display_path}: {err}"))
        }
        None => read_all_with_limit(io::stdin(), libaprs_engine::DEFAULT_TRANSPORT_READ_LIMIT)
            .map_err(|err| format!("failed to read stdin: {err}")),
    }
}

fn print_support_matrix(json: bool) {
    let matrix = support_matrix();
    if json {
        println!("{}", support_matrix_json(matrix));
    } else {
        println!("support-matrix schema_version={}", matrix.schema_version);
        println!("semantic_families:");
        for item in matrix.semantic_families {
            println!(
                "- kind={} status={} notes={}",
                item.kind,
                item.status.code(),
                item.notes
            );
        }
        println!("transport_adapters:");
        for item in matrix.transport_adapters {
            println!(
                "- crate={} status={} boundary={} notes={}",
                item.crate_name,
                item.status.code(),
                item.boundary,
                item.notes
            );
        }
        println!("diagnostic_layers:");
        for layer in matrix.diagnostic_layers {
            println!("- code={}", layer.code());
        }
    }
}

fn support_matrix_json(matrix: SupportMatrix) -> String {
    let mut json = String::new();
    json.push_str("{\"schema_version\":");
    json.push_str(&matrix.schema_version.to_string());
    json.push_str(",\"semantic_families\":[");
    for (index, item) in matrix.semantic_families.iter().enumerate() {
        if index > 0 {
            json.push(',');
        }
        json.push_str("{\"kind\":\"");
        json.push_str(&json_escape(item.kind));
        json.push_str("\",\"status\":\"");
        json.push_str(item.status.code());
        json.push_str("\",\"notes\":\"");
        json.push_str(&json_escape(item.notes));
        json.push_str("\"}");
    }
    json.push_str("],\"transport_adapters\":[");
    for (index, item) in matrix.transport_adapters.iter().enumerate() {
        if index > 0 {
            json.push(',');
        }
        json.push_str("{\"crate\":\"");
        json.push_str(&json_escape(item.crate_name));
        json.push_str("\",\"boundary\":\"");
        json.push_str(&json_escape(item.boundary));
        json.push_str("\",\"status\":\"");
        json.push_str(item.status.code());
        json.push_str("\",\"notes\":\"");
        json.push_str(&json_escape(item.notes));
        json.push_str("\"}");
    }
    json.push_str("],\"diagnostic_layers\":[");
    for (index, layer) in matrix.diagnostic_layers.iter().enumerate() {
        if index > 0 {
            json.push(',');
        }
        json.push_str("{\"code\":\"");
        json.push_str(match layer {
            DiagnosticLayer::Parse => "parse",
            DiagnosticLayer::Policy => "policy",
            DiagnosticLayer::Transport => "transport",
        });
        json.push_str("\"}");
    }
    json.push_str("]}");
    json
}

fn packet_json(packet: &ParsedPacket) -> String {
    format!(
        "{{\"schema_version\":1,\"raw\":\"{}\",\"source\":\"{}\",\"destination\":\"{}\",\"path\":\"{}\",\"payload\":\"{}\",\"data_type\":\"{}\",\"semantic\":\"{}\"}}",
        json_escape_bytes(packet.raw().as_bytes()),
        json_escape_bytes(packet.source()),
        json_escape_bytes(packet.destination()),
        json_escape_bytes(packet.path()),
        json_escape_bytes(packet.payload()),
        packet.data_type_identifier().name(),
        packet.aprs_data().kind_name(),
    )
}

fn json_escape_bytes(bytes: &[u8]) -> String {
    let mut escaped = String::new();
    for byte in bytes {
        match byte {
            b'"' => escaped.push_str("\\\""),
            b'\\' => escaped.push_str("\\\\"),
            b'\n' => escaped.push_str("\\n"),
            b'\r' => escaped.push_str("\\r"),
            b'\t' => escaped.push_str("\\t"),
            0x20..=0x7e => escaped.push(char::from(*byte)),
            _ => {
                escaped.push_str("\\u00");
                escaped.push(hex_digit(byte >> 4));
                escaped.push(hex_digit(byte & 0x0f));
            }
        }
    }
    escaped
}

fn hex_digit(value: u8) -> char {
    match value {
        0..=9 => char::from(b'0' + value),
        10..=15 => char::from(b'a' + value - 10),
        _ => '0',
    }
}

fn json_escape(value: &str) -> String {
    let mut escaped = String::new();
    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            '\u{08}' => escaped.push_str("\\b"),
            '\u{0c}' => escaped.push_str("\\f"),
            ch if ch.is_control() => {
                escaped.push_str("\\u");
                escaped.push_str(&format!("{:04x}", u32::from(ch)));
            }
            ch => escaped.push(ch),
        }
    }
    escaped
}

fn lossy(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn diagnostic_value(value: &str) -> String {
    format!("{value:?}")
}

fn usage() -> String {
    "usage: aprs-cli [parse|validate|stats|explain|replay|support-matrix] [--json] [--permissive] [--explain] [--summary] [--filter SEMANTIC] [--fail-on none|malformed|rejected] [PATH]".to_string()
}
