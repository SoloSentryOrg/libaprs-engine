use std::io::{self, Cursor};
use std::time::Duration;

use aprs_transport_aprs_is::{read_packet_lines_from_reader_with_limit, AprsIsLogin};
use libaprs_engine::{Engine, EngineEvent, DEFAULT_TRANSPORT_READ_LIMIT};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let login = AprsIsLogin {
        callsign: "N0CALL",
        passcode: -1,
        software: "libaprs-engine 1.1.0",
        filter: Some("r/49/-72/50"),
    };

    let plan = ReconnectPlan {
        max_attempts: 2,
        backoff: Duration::from_millis(25),
        max_read_bytes: DEFAULT_TRANSPORT_READ_LIMIT,
    };
    let mut engine = Engine::default();

    run_receive_session(&login, plan, &mut engine, |attempt| {
        if attempt == 0 {
            Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "simulated idle APRS-IS session",
            ))
        } else {
            Ok(Cursor::new(
                b"# server banner\r\nN0CALL>APRS:>session online\n".as_slice(),
            ))
        }
    })?;

    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ReconnectPlan {
    max_attempts: usize,
    backoff: Duration,
    max_read_bytes: usize,
}

fn run_receive_session<R>(
    login: &AprsIsLogin<'_>,
    plan: ReconnectPlan,
    engine: &mut Engine,
    mut connect: impl FnMut(usize) -> io::Result<R>,
) -> io::Result<()>
where
    R: io::Read,
{
    let login_line = login.line().map_err(io::Error::other)?;

    for attempt in 0..plan.max_attempts {
        match connect(attempt) {
            Ok(reader) => {
                let packets =
                    read_packet_lines_from_reader_with_limit(reader, plan.max_read_bytes)?;
                for packet in packets {
                    match engine.process_event(&packet) {
                        EngineEvent::Accepted(event) => {
                            println!("{} {}", login_line.trim_end(), event.kind().code());
                        }
                        EngineEvent::Rejected(event) => {
                            println!("{} {}", event.kind().code(), event.diagnostic.code);
                        }
                        EngineEvent::Malformed(event) => {
                            println!("{} {}", event.kind().code(), event.diagnostic.code);
                        }
                    }
                }
                return Ok(());
            }
            Err(error) if attempt + 1 < plan.max_attempts => {
                eprintln!("session attempt {attempt} failed: {error}; retrying after backoff");
                std::thread::sleep(plan.backoff);
            }
            Err(error) => return Err(error),
        }
    }

    Ok(())
}
