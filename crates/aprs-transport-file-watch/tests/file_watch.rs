use std::fs;

use aprs_transport_file_watch::{
    read_appended_packet_lines, read_appended_packet_lines_with_limit,
};

#[test]
fn file_watch_reads_only_appended_bytes() {
    let path = std::env::temp_dir().join(format!("libaprs-file-watch-{}", std::process::id()));
    fs::write(&path, b"N0CALL>APRS:>one\n").expect("write");

    let first = read_appended_packet_lines(&path, 0).expect("first read");
    assert_eq!(first.next_offset, b"N0CALL>APRS:>one\n".len() as u64);
    assert_eq!(first.packets, vec![b"N0CALL>APRS:>one".to_vec()]);

    fs::write(&path, b"N0CALL>APRS:>one\nN1CALL>APRS:>\xff\n").expect("append simulation");
    let second = read_appended_packet_lines(&path, first.next_offset).expect("second read");

    assert_eq!(second.packets, vec![b"N1CALL>APRS:>\xff".to_vec()]);
    let _ = fs::remove_file(path);
}

#[test]
fn file_watch_rejects_appended_bytes_over_configured_limit() {
    let path =
        std::env::temp_dir().join(format!("libaprs-file-watch-limit-{}", std::process::id()));
    fs::write(&path, b"N0CALL>APRS:>oversized\n").expect("write");

    let error =
        read_appended_packet_lines_with_limit(&path, 0, 4).expect_err("oversized input fails");

    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    assert_eq!(error.to_string(), "transport.oversized_input");
    let _ = fs::remove_file(path);
}
