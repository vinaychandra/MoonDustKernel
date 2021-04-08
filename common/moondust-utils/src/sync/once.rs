use core::{
    pin::Pin,
    task::{Context, Poll, Waker},
};

use alloc::sync::Arc;
use crossbeam_queue::SegQueue;
use futures_lite::Future;

#[derive(Debug)]
pub struct AsyncOnce<T> {
    state: Arc<spin::Once<Arc<T>>>,
    wakers: Arc<SegQueue<Waker>>,
}

impl<T> Clone for AsyncOnce<T> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            wakers: self.wakers.clone(),
        }
    }
}

impl<T> AsyncOnce<T> {
    pub fn new() -> Self {
        Self {
            state: Arc::new(spin::Once::new()),
            wakers: Arc::new(SegQueue::new()),
        }
    }

    pub fn is_ready(&self) -> bool {
        self.state.is_completed()
    }

    pub fn try_set_result(&self, result: T) {
        self.state.call_once(|| Arc::new(result));
        for waker in self.wakers.pop() {
            waker.wake();
        }
    }
}

impl<T> Future for AsyncOnce<T> {
    type Output = Arc<T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.state.is_completed() {
            let result = self.state.get().unwrap().clone();
            return Poll::Ready(result);
        }

        self.wakers.push(cx.waker().clone());

        if self.state.is_completed() {
            let result = self.state.get().unwrap().clone();
            return Poll::Ready(result);
        } else {
            return Poll::Pending;
        }
    }
}
