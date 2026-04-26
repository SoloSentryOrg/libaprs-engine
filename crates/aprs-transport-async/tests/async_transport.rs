use std::future::Future;
use std::pin::pin;
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};

use aprs_transport_async::split_packet_lines;

#[test]
fn async_split_preserves_packet_bytes() {
    let packets = block_on(split_packet_lines(b"N0CALL>APRS:>\xff\n"));

    assert_eq!(packets, vec![b"N0CALL>APRS:>\xff".to_vec()]);
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
