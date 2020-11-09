//! Async mutex implementation
//! Example usage
//!    let a = Arc::new(Mutex::new(1));
//!    let x = arch::process::block_on(a.lock());
//!    let b = a.clone();
//!    exec.spawn(Priority::Medium, async {
//!        {
//!            {
//!                let mut m = b.lock().await;
//!                *m = 2;
//!                info!("M VAL is {}", *m);
//!            }
//!            {
//!                let mut m = b.lock().await;
//!                *m = 3;
//!                info!("M VAL is {}", *m);
//!            }
//!        }
//!    })
//!    .detach();
//!    exec.spawn(Priority::Medium, async {
//!        {
//!            drop(x);
//!            {
//!                let mut m = a.lock().await;
//!                *m = 4;
//!                info!("M VAL is {}", *m);
//!            }
//!            {
//!                let mut m = a.lock().await;
//!                *m = 5;
//!                info!("M VAL is {}", *m);
//!            }
//!        }
//!    })
//!    .detach();
//! Output gets "4 5 2 3"

use core::{
    cell::UnsafeCell,
    future::Future,
    ops::{Deref, DerefMut},
    pin::Pin,
    sync::atomic::{AtomicBool, Ordering},
    task::{Context, Poll, Waker},
};

use crossbeam_queue::SegQueue;

/// A mutual exclusion primitive for protecting shared data.
pub struct Mutex<T> {
    blocked: AtomicBool,
    waiting: SegQueue<Waker>,
    value: UnsafeCell<T>,
}

unsafe impl<T: Send> Send for Mutex<T> {}
unsafe impl<T: Send> Sync for Mutex<T> {}

impl<T> Mutex<T> {
    /// Creates a new Mutex.
    pub fn new(t: T) -> Mutex<T> {
        Mutex {
            blocked: AtomicBool::new(false),
            waiting: SegQueue::new(),
            value: UnsafeCell::new(t),
        }
    }

    pub async fn lock(&self) -> MutexGuard<'_, T> {
        LockFuture { mutex: self }.await
    }
}

/// A guard that releases the lock when dropped.
pub struct MutexGuard<'a, T>(&'a Mutex<T>);

unsafe impl<T: Send> Send for MutexGuard<'_, T> {}
unsafe impl<T: Sync> Sync for MutexGuard<'_, T> {}

impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        self.0.blocked.store(false, Ordering::SeqCst);

        // TODO optimize. We wake everything expecting somone will be ready.
        while let Some(waker) = self.0.waiting.pop() {
            waker.wake();
        }
    }
}

impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.0.value.get() }
    }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.0.value.get() }
    }
}

struct LockFuture<'a, T> {
    mutex: &'a Mutex<T>,
}

impl<'a, T> Future for LockFuture<'a, T> {
    type Output = MutexGuard<'a, T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if !self.mutex.blocked.swap(true, Ordering::SeqCst) {
            // Acquired the mutex
            return Poll::Ready(MutexGuard(self.mutex));
        }

        self.mutex.waiting.push(cx.waker().clone());

        // We retry to lock to prevent race conditions.
        if !self.mutex.blocked.swap(true, Ordering::SeqCst) {
            // Acquired the mutex
            return Poll::Ready(MutexGuard(self.mutex));
        }

        return Poll::Pending;
    }
}
