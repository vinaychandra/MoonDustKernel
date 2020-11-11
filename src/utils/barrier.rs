use core::{
    pin::Pin,
    task::{Context, Poll, Waker},
};

use crossbeam_queue::ArrayQueue;
use futures_lite::Future;

/// Sync for multiple threads. Will unpause all after reaching threshold.
pub struct Barrier {
    wakers: ArrayQueue<Waker>,
}

impl Barrier {
    pub fn new(n: usize) -> Barrier {
        Barrier {
            wakers: ArrayQueue::new(n - 1),
        }
    }

    pub async fn wait_async(&self) -> () {
        BarrierFuture {
            barrier: self,
            first_time: true,
        }
        .await
    }
}

struct BarrierFuture<'a> {
    barrier: &'a Barrier,
    first_time: bool,
}

impl<'a> Future for BarrierFuture<'a> {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // TODO Check for race conditions here
        if self.first_time {
            self.first_time = false;
            if self.barrier.wakers.push(cx.waker().clone()).is_ok() {
                return Poll::Pending;
            } else {
                while let Some(waker) = self.barrier.wakers.pop() {
                    waker.wake();
                }

                return Poll::Ready(());
            }
        } else {
            return Poll::Ready(());
        }
    }
}
