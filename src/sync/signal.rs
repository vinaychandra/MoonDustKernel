use core::{
    pin::Pin,
    task::{Context, Poll, Waker},
};

use crossbeam_queue::SegQueue;
use futures_lite::Future;

/// A signal for all waiting tasks.
pub struct Signal {
    wakers: SegQueue<Waker>,
}

impl Signal {
    pub const fn new() -> Signal {
        Signal {
            wakers: SegQueue::new(),
        }
    }

    /// Wait until a signal has been received. Must await
    /// to register the wait.
    pub async fn wait_async(&self) -> () {
        SignalFuture {
            signal: self,
            first_time: true,
        }
        .await
    }

    /// Signal all the waiting threads.
    pub fn signal(&self) {
        while let Some(v) = self.wakers.pop() {
            v.wake();
        }
    }
}

struct SignalFuture<'a> {
    signal: &'a Signal,
    first_time: bool,
}

impl<'a> Future for SignalFuture<'a> {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.first_time {
            self.first_time = false;
            self.signal.wakers.push(cx.waker().clone());
            return Poll::Pending;
        } else {
            return Poll::Ready(());
        }
    }
}
