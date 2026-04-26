use std::fs;

use aprs_transport_corpus::{read_corpus_packet_lines, read_corpus_packet_lines_with_limit};

#[test]
fn corpus_reader_reads_files_in_stable_order() {
    let dir = std::env::temp_dir().join(format!("libaprs-corpus-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir(&dir).expect("create dir");
    fs::write(dir.join("b.aprs"), b"N1CALL>APRS:>b\n").expect("write b");
    fs::write(dir.join("a.aprs"), b"N0CALL>APRS:>a\n").expect("write a");

    let packets = read_corpus_packet_lines(&dir).expect("read corpus");

    assert_eq!(
        packets,
        vec![b"N0CALL>APRS:>a".to_vec(), b"N1CALL>APRS:>b".to_vec()]
    );
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn corpus_reader_rejects_file_over_configured_limit() {
    let dir = std::env::temp_dir().join(format!("libaprs-corpus-limit-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir(&dir).expect("create dir");
    fs::write(dir.join("a.aprs"), b"N0CALL>APRS:>oversized\n").expect("write");

    let error = read_corpus_packet_lines_with_limit(&dir, 4).expect_err("oversized input fails");

    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    assert_eq!(error.to_string(), "transport.oversized_input");
    let _ = fs::remove_dir_all(dir);
}
