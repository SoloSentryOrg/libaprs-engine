#![cfg(feature = "serde")]

use libaprs_engine::{parse_packet, serde_support::PacketDiagnostic};

#[test]
fn serde_diagnostic_preserves_non_utf8_bytes() {
    let packet = parse_packet(b"N0CALL>APRS:>\xff").expect("packet should parse");
    let diagnostic = PacketDiagnostic::from_packet(&packet);
    let explicit_diagnostic = packet.to_diagnostic();

    assert_eq!(diagnostic.raw, b"N0CALL>APRS:>\xff");
    assert_eq!(explicit_diagnostic, diagnostic);
    assert_eq!(diagnostic.semantic, "status");
}
