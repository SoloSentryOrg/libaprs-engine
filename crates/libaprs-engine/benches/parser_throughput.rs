use std::time::{Duration, Instant};

use libaprs_engine::parse_packet;

fn main() {
    let packet = b"N0CALL>APRS,WIDE1-1:>benchmark packet";
    let iterations = 100_000;
    let started = Instant::now();

    for _ in 0..iterations {
        parse_packet(packet).expect("benchmark packet should parse");
    }

    let elapsed = started.elapsed();
    print_summary(iterations, elapsed);
    enforce_threshold(iterations, elapsed);
}

fn print_summary(iterations: u32, elapsed: Duration) {
    let nanos = elapsed.as_nanos() / u128::from(iterations);
    println!("parsed {iterations} packets in {elapsed:?} ({nanos} ns/packet)");
}

fn enforce_threshold(iterations: u32, elapsed: Duration) {
    let Ok(limit) = std::env::var("LIBAPRS_MAX_NS_PER_PACKET") else {
        return;
    };
    let limit = limit
        .parse::<u128>()
        .expect("LIBAPRS_MAX_NS_PER_PACKET must be an integer");
    let nanos = elapsed.as_nanos() / u128::from(iterations);
    assert!(
        nanos <= limit,
        "parser throughput regression: {nanos} ns/packet exceeds configured limit {limit}"
    );
}
