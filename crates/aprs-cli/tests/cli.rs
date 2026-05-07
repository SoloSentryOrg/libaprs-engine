use std::process::Command;

use libaprs_engine::DEFAULT_TRANSPORT_READ_LIMIT;

#[test]
fn cli_reads_json_packets_from_stdin() {
    let binary = env!("CARGO_BIN_EXE_aprs-cli");
    let output = Command::new(binary)
        .arg("--json")
        .output_with_stdin(b"N0CALL>APRS:>hello\n")
        .expect("CLI should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf8");
    assert!(stdout.contains("\"schema_version\":1"));
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

#[test]
fn cli_summary_prints_counters_to_stdout() {
    let binary = env!("CARGO_BIN_EXE_aprs-cli");
    let output = Command::new(binary)
        .arg("--summary")
        .output_with_stdin(b"N0CALL>APRS:>hello\nN0CALL>APRS:~opaque\n")
        .expect("CLI should run");

    assert!(!output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf8");
    assert!(stdout.contains("summary accepted=1 rejected=1 malformed=0"));
}

#[test]
fn cli_explain_prints_stable_rejection_codes() {
    let binary = env!("CARGO_BIN_EXE_aprs-cli");
    let output = Command::new(binary)
        .arg("--explain")
        .output_with_stdin(b"N0CALL>APRS:~opaque\n")
        .expect("CLI should run");

    assert!(!output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf8");
    assert!(stdout.contains("code=policy.unsupported_semantics"));
}

#[test]
fn cli_fail_on_none_allows_observability_without_failure_exit() {
    let binary = env!("CARGO_BIN_EXE_aprs-cli");
    let output = Command::new(binary)
        .args(["--fail-on", "none"])
        .output_with_stdin(b"N0CALL>APRS:~opaque\n")
        .expect("CLI should run");

    assert!(output.status.success());
}

#[test]
fn cli_rejects_oversized_file_input() {
    let binary = env!("CARGO_BIN_EXE_aprs-cli");
    let path = std::env::temp_dir().join(format!("libaprs-cli-limit-{}", std::process::id()));
    std::fs::write(&path, vec![b'A'; DEFAULT_TRANSPORT_READ_LIMIT + 1]).expect("write");

    let output = Command::new(binary)
        .arg(&path)
        .output()
        .expect("CLI should run");

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("transport.oversized_input"));
    let _ = std::fs::remove_file(path);
}

#[test]
fn cli_diagnostic_errors_escape_control_characters() {
    let binary = env!("CARGO_BIN_EXE_aprs-cli");
    let output = Command::new(binary)
        .arg("--bad\noption")
        .output()
        .expect("CLI should run");

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf8");
    assert!(stderr.contains("unknown option: \"--bad\\noption\""));
    assert!(!stderr.contains("unknown option: --bad\noption"));

    let output = Command::new(binary)
        .args(["--fail-on", "bad\rvalue"])
        .output()
        .expect("CLI should run");

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).expect("stderr should be utf8");
    assert!(stderr.contains("invalid --fail-on value: \"bad\\rvalue\""));
    assert!(!stderr.contains("invalid --fail-on value: bad\rvalue"));
}

#[test]
fn cli_validate_command_reports_validity() {
    let binary = env!("CARGO_BIN_EXE_aprs-cli");
    let output = Command::new(binary)
        .arg("validate")
        .output_with_stdin(b"N0CALL>APRS:>hello\n")
        .expect("CLI should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf8");
    assert!(stdout.contains("valid accepted=1 rejected=0 malformed=0"));
}

#[test]
fn cli_stats_command_prints_only_summary_to_stdout() {
    let binary = env!("CARGO_BIN_EXE_aprs-cli");
    let output = Command::new(binary)
        .arg("stats")
        .output_with_stdin(b"N0CALL>APRS:>hello\n")
        .expect("CLI should run");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf8");
    assert_eq!(stdout.trim(), "summary accepted=1 rejected=0 malformed=0");
}

#[test]
fn cli_explain_command_prints_stable_codes() {
    let binary = env!("CARGO_BIN_EXE_aprs-cli");
    let output = Command::new(binary)
        .arg("explain")
        .output_with_stdin(b"N0CALL>APRS:~opaque\n")
        .expect("CLI should run");

    assert!(!output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf8");
    assert!(stdout.contains("code=policy.unsupported_semantics"));
}

#[test]
fn cli_replay_command_preserves_accepted_packet_bytes() {
    let binary = env!("CARGO_BIN_EXE_aprs-cli");
    let output = Command::new(binary)
        .arg("replay")
        .arg("--permissive")
        .output_with_stdin(b"N0CALL>APRS:>\xff\nN1CALL>APRS:~opaque\n")
        .expect("CLI should run");

    assert!(output.status.success());
    assert_eq!(
        output.stdout,
        b"N0CALL>APRS:>\xff\nN1CALL>APRS:~opaque\n".to_vec()
    );
}

#[test]
fn cli_replay_command_does_not_mix_rejections_into_packet_stream() {
    let binary = env!("CARGO_BIN_EXE_aprs-cli");
    let output = Command::new(binary)
        .arg("replay")
        .output_with_stdin(b"N0CALL>APRS:>ok\nN1CALL>APRS:~opaque\n")
        .expect("CLI should run");

    assert!(!output.status.success());
    assert_eq!(output.stdout, b"N0CALL>APRS:>ok\n".to_vec());
}

#[test]
fn cli_support_matrix_json_is_machine_readable_without_packet_input() {
    let binary = env!("CARGO_BIN_EXE_aprs-cli");
    let output = Command::new(binary)
        .args(["support-matrix", "--json"])
        .output()
        .expect("CLI should run");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).expect("stdout should be utf8");
    assert!(stdout.contains("\"schema_version\":1"));
    assert!(stdout.contains("\"semantic_families\""));
    assert!(stdout.contains("\"kind\":\"status\""));
    assert!(stdout.contains("\"kind\":\"mic_e\""));
    assert!(stdout.contains("\"transport_adapters\""));
    assert!(stdout.contains("\"crate\":\"aprs-transport-kiss\""));
    assert!(stdout.contains("\"diagnostic_layers\""));
    assert!(stdout.contains("\"code\":\"parse\""));
    assert!(stdout.contains("\"code\":\"policy\""));
    assert!(stdout.contains("\"code\":\"transport\""));
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
