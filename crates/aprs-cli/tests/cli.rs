use std::process::Command;

#[test]
fn cli_reads_json_packets_from_stdin() {
    let binary = env!("CARGO_BIN_EXE_aprs-cli");
    let output = Command::new(binary)
        .arg("--json")
        .output_with_stdin(b"N0CALL>APRS:>hello\n")
        .expect("CLI should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf8");
    assert!(stdout.contains("\"semantic\":\"status\""));
}

#[test]
fn cli_preserves_non_utf8_packet_bytes_from_stdin() {
    let binary = env!("CARGO_BIN_EXE_aprs-cli");
    let output = Command::new(binary)
        .arg("--json")
        .output_with_stdin(b"N0CALL>APRS:>\xff\n")
        .expect("CLI should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf8");
    assert!(stdout.contains("\"raw\":\"N0CALL>APRS:>\\u00ff\""));
}

#[test]
fn cli_filters_accepted_packets_by_semantic_kind() {
    let binary = env!("CARGO_BIN_EXE_aprs-cli");
    let output = Command::new(binary)
        .args(["--filter", "status"])
        .output_with_stdin(b"N0CALL>APRS:>hello\nN0CALL>APRS:!4903.50N/07201.75W-Test\n")
        .expect("CLI should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf8");
    assert!(stdout.contains("semantic=status"));
    assert!(!stdout.contains("semantic=position"));
}

#[test]
fn cli_permissive_mode_accepts_unsupported_semantics() {
    let binary = env!("CARGO_BIN_EXE_aprs-cli");
    let output = Command::new(binary)
        .arg("--permissive")
        .output_with_stdin(b"N0CALL>APRS:~opaque\n")
        .expect("CLI should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf8");
    assert!(stdout.contains("semantic=unsupported"));
}

trait CommandStdinExt {
    fn output_with_stdin(&mut self, input: &[u8]) -> std::io::Result<std::process::Output>;
}

impl CommandStdinExt for Command {
    fn output_with_stdin(&mut self, input: &[u8]) -> std::io::Result<std::process::Output> {
        use std::io::Write;
        use std::process::Stdio;

        let mut child = self.stdin(Stdio::piped()).stdout(Stdio::piped()).spawn()?;
        child
            .stdin
            .as_mut()
            .expect("stdin should be piped")
            .write_all(input)?;
        child.wait_with_output()
    }
}
