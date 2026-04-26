use aprs_transport_mqtt::{packet_from_publish_payload, topic_matches};

#[test]
fn mqtt_payload_helper_preserves_bytes_and_matches_topics() {
    assert_eq!(
        packet_from_publish_payload(b"N0CALL>APRS:>\xff"),
        b"N0CALL>APRS:>\xff".to_vec()
    );
    assert!(topic_matches("aprs/+/packet", "aprs/N0CALL/packet"));
    assert!(!topic_matches("aprs/+/packet", "aprs/N0CALL/status"));
}
