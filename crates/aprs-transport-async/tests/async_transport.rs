use std::future::Future;
use std::pin::pin;
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};

use aprs_transport_async::{split_packet_lines, split_packet_lines_with_limit};

#[test]
fn async_split_preserves_packet_bytes() {
    let packets = block_on(split_packet_lines(b"N0CALL>APRS:>\xff\n"));

    assert_eq!(packets, vec![b"N0CALL>APRS:>\xff".to_vec()]);
}

#[test]
fn async_split_can_reject_packet_lines_over_configured_limit() {
    let error = block_on(split_packet_lines_with_limit(b"N0CALL>APRS:>too-long\n", 4))
        .expect_err("oversized packet line must fail");

    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    assert_eq!(error.to_string(), "transport.oversized_input");
}

fn block_on<F: Future>(future: F) -> F::Output {
    struct NoopWake;

    impl Wake for NoopWake {
        fn wake(self: Arc<Self>) {}
    }

    let waker = Waker::from(Arc::new(NoopWake));
    let mut context = Context::from_waker(&waker);
    let mut future = pin!(future);

    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(output) => return output,
            Poll::Pending => std::thread::yield_now(),
        }
    }
}
