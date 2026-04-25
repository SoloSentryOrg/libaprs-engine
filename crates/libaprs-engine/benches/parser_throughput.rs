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
}

fn print_summary(iterations: u32, elapsed: Duration) {
    let nanos = elapsed.as_nanos() / u128::from(iterations);
    println!("parsed {iterations} packets in {elapsed:?} ({nanos} ns/packet)");
}
