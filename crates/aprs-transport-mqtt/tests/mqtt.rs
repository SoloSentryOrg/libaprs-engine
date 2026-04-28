use aprs_transport_mqtt::{
    packet_from_publish_payload, packet_from_publish_payload_with_limit, topic_matches,
};

#[test]
fn mqtt_payload_helper_preserves_bytes_and_matches_topics() {
    assert_eq!(
        packet_from_publish_payload(b"N0CALL>APRS:>\xff"),
        b"N0CALL>APRS:>\xff".to_vec()
    );
    assert!(topic_matches("aprs/+/packet", "aprs/N0CALL/packet"));
    assert!(!topic_matches("aprs/+/packet", "aprs/N0CALL/status"));
}

#[test]
fn mqtt_payload_helper_rejects_payload_over_configured_limit() {
    let error = packet_from_publish_payload_with_limit(b"N0CALL>APRS:>too-long", 4)
        .expect_err("oversized payload must fail");

    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    assert_eq!(error.to_string(), "transport.oversized_input");
}
