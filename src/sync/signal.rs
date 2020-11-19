use core::{
    pin::Pin,
    sync::atomic::{AtomicU64, Ordering},
    task::{Context, Poll, Waker},
};

use crossbeam_queue::SegQueue;
use futures_lite::Future;

/// A signal for all waiting tasks.
pub struct Signal {
    wakers: SegQueue<Waker>,
    generation_count: AtomicU64,
}

impl Signal {
    pub const fn new() -> Signal {
        Signal {
            wakers: SegQueue::new(),
            generation_count: AtomicU64::new(0),
        }
    }

    /// Wait until a signal has been received. Must await
    /// to register the wait.
    pub async fn wait_async(&self) -> () {
        SignalFuture {
            signal: self,
            this_gen_count: self.generation_count.load(Ordering::SeqCst),
        }
        .await
    }

    /// Signal all the waiting threads.
    pub fn signal(&self) {
        self.generation_count.fetch_add(1, Ordering::SeqCst);
        while let Some(v) = self.wakers.pop() {
            v.wake();
        }
    }
}

struct SignalFuture<'a> {
    signal: &'a Signal,
    this_gen_count: u64,
}

impl<'a> Future for SignalFuture<'a> {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.this_gen_count >= self.signal.generation_count.load(Ordering::SeqCst) {
            self.signal.wakers.push(cx.waker().clone());
            return Poll::Pending;
        } else {
            return Poll::Ready(());
        }
    }
}
