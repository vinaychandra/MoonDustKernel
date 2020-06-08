use core::cmp::*;
use crossbeam_queue::SegQueue;
use spin::{Mutex, MutexGuard};

/// Struct to generate unused ids in a system.
pub struct IdGenerator {
    /// A list of unused values.
    unused_values: SegQueue<u32>,

    /// The step size to increase the size of the `unused_values`
    /// array.
    step_size: u32,

    /// Number of steps expanded until now. This is a mutex so that
    /// finishing one set will not cause a lot of sets to be created
    /// at once.
    steps_done: Mutex<u32>,

    /// The max value for the id.
    max_value: u32,
}

impl IdGenerator {
    /// Create a new id generator. Ids are generated from 1 to `max_value`.
    ///
    /// # Arguments
    /// * `max_value` - The max value for an id.
    /// * `step` - The step size to increase the capacity
    /// of the buffer used internally.
    ///
    /// # Notes
    /// - If `max_value` is less than `step`, `step` will be
    /// set to `max_value`.
    pub fn new(max_value: u32, step: u32) -> IdGenerator {
        // Set step to the minimum of max_value and step.
        let step = min(max_value, step);
        let vec = SegQueue::<u32>::new();

        IdGenerator {
            unused_values: vec,
            step_size: step,
            steps_done: Mutex::new(0),
            max_value,
        }
    }

    /// Get a new id that can be used.
    ///
    /// This returns `Some(value)` if an id is available.
    /// If the list is exhausted, `None` is returned.
    pub fn pop(&self) -> Option<u32> {
        if let Ok(val) = self.unused_values.pop() {
            return Some(val);
        }

        let steps_done = self.steps_done.lock();
        if let Ok(val) = self.unused_values.pop() {
            return Some(val);
        }

        if self.push_values(steps_done).is_err() {
            return None;
        }

        self.pop()
    }

    /// Push an value which will be marked as 'unused'
    /// and further can be reassigned.
    pub fn push(&self, value: u32) {
        self.unused_values.push(value);
    }

    /// Push the next set of values onto the unused list.
    /// The list is set as steps so that initial allocation will be fast.
    fn push_values(&self, mut steps_done: MutexGuard<u32>) -> Result<(), ()> {
        let range_start = *steps_done * self.step_size + 1;
        if range_start > self.max_value {
            return Err(());
        }

        let range_end = (*steps_done + 1) * self.step_size + 1;
        let range_end = min(range_end, self.max_value + 1);

        for i in range_start..range_end {
            self.unused_values.push(i);
        }

        *steps_done += 1;

        Ok(())
    }
}
