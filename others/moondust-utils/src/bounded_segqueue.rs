//! Bound wrapper for crossbeam SegQueue
use core::ops::Deref;

use crossbeam_queue::SegQueue;

#[derive(Debug)]
pub struct BoundedSegQueue<T> {
    inner: SegQueue<T>,

    /// Approximate capacity.
    capacity: usize,
}

impl<T> Deref for BoundedSegQueue<T> {
    type Target = SegQueue<T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> BoundedSegQueue<T> {
    pub fn unbounded() -> Self {
        BoundedSegQueue {
            inner: SegQueue::new(),
            capacity: 0,
        }
    }

    pub fn bounded(capacity: usize) -> Self {
        BoundedSegQueue {
            inner: SegQueue::new(),
            capacity,
        }
    }

    pub fn push(&self, value: T) -> Result<(), ()> {
        if self.capacity > 0 && self.inner.len() > self.capacity {
            return Err(());
        }

        self.inner.push(value);
        Ok(())
    }

    pub fn capacity(&self) -> Option<usize> {
        if self.capacity > 0 {
            Some(self.capacity)
        } else {
            None
        }
    }
}
