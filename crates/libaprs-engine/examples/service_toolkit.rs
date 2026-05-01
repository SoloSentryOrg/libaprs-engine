use libaprs_engine::{
    parse_packet,
    service::{
        DuplicateDecision, DuplicateWindow, PacketRateBudget, RateLimitDecision, SemanticBlocklist,
        SemanticFamily,
    },
    Engine,
};

fn main() -> Result<(), libaprs_engine::ParseError> {
    let mut engine = Engine::default();
    let mut duplicates = DuplicateWindow::new(128);
    let mut rate = PacketRateBudget::new(100);
    let blocked = SemanticBlocklist::new(&[SemanticFamily::Unsupported, SemanticFamily::Malformed]);

    for packet_bytes in [b"N0CALL>APRS:>service".as_slice()] {
        if rate.try_consume() == RateLimitDecision::Limited {
            continue;
        }
        if duplicates.observe(packet_bytes) == DuplicateDecision::Duplicate {
            continue;
        }

        let packet = parse_packet(packet_bytes)?;
        if blocked.rejects(&packet.aprs_data()) {
            continue;
        }

        let _event = engine.process_event(packet.raw().as_bytes());
    }

    Ok(())
}
