use async_task::Task;
use futures_lite::{future, prelude::*};

use super::executor::Executor;

/// Task priority.
#[repr(usize)]
#[derive(Debug, Clone, Copy)]
pub enum Priority {
    High = 0,
    Medium = 1,
    Low = 2,
}

/// An executor with task priorities.
///
/// Tasks with lower priorities only get polled when there are no tasks with higher priorities.
pub struct PriorityExecutor<'a> {
    ex: [Executor<'a>; 3],
}

impl<'a> PriorityExecutor<'a> {
    /// Creates a new executor.
    pub const fn new() -> PriorityExecutor<'a> {
        PriorityExecutor {
            ex: [Executor::new(), Executor::new(), Executor::new()],
        }
    }

    /// Spawns a task with the given priority.
    pub fn spawn<T: Send + 'a>(
        &self,
        priority: Priority,
        future: impl Future<Output = T> + Send + 'a,
    ) -> Task<T> {
        self.ex[priority as usize].spawn(future)
    }

    /// Runs the executor forever yielding every now and then..
    pub async fn run_simple(&self) {
        loop {
            for _ in 0..200 {
                let t0 = self.ex[0].tick();
                let t1 = self.ex[1].tick();
                let t2 = self.ex[2].tick();

                // Wait until one of the ticks completes, trying them in order from highest
                // priority to lowest priority.
                t0.or(t1).or(t2).await;
            }

            // Yield every now and then.
            future::yield_now().await;
        }
    }

    /// Runs the executor forever yielding every now and then..
    pub async fn run(&self) {
        loop {
            let t0 = self.ex[0].tick();
            let t1 = self.ex[1].tick();
            let t2 = self.ex[2].tick();

            // Wait until one of the ticks completes, trying them in order from highest
            // priority to lowest priority.
            t0.or(t1).or(t2).await;
        }
    }
}
