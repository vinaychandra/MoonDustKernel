use core::{
    sync::atomic::{AtomicUsize, Ordering},
    usize,
};

use crossbeam_queue::SegQueue;

pub struct IdGenerator {
    pool_start: AtomicUsize,
    returned_value_heap: SegQueue<usize>, // TODO: Move these to pool_start
}

impl IdGenerator {
    pub const fn new() -> IdGenerator {
        IdGenerator {
            pool_start: AtomicUsize::new(1),
            returned_value_heap: SegQueue::new(),
        }
    }

    pub fn get_value(&self) -> usize {
        if let Some(val) = self.returned_value_heap.pop() {
            val
        } else {
            self.pool_start.fetch_add(1, Ordering::Relaxed)
        }
    }

    pub fn return_value(&self, value: usize) {
        self.returned_value_heap.push(value);
    }
}
