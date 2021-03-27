use async_task::Task;
use futures_lite::prelude::*;

use super::async_executor::Executor;

/// An executor with task priorities.
///
/// Tasks with lower priorities only get polled when there are no tasks with higher priorities.
pub struct PriorityExecutor<'a, const PRIORTY_COUNT: usize> {
    ex: [Executor<'a>; PRIORTY_COUNT],
}

// TODO: Remove the where clause once Default supports arbitarary length.
impl<'a, const PRIORTY_COUNT: usize> PriorityExecutor<'a, PRIORTY_COUNT>
where
    [Executor<'a>; PRIORTY_COUNT]: Default,
{
    pub const fn const_new() -> PriorityExecutor<'static, PRIORTY_COUNT> {
        const VAL: Executor<'static> = Executor::new();
        PriorityExecutor {
            ex: [VAL; PRIORTY_COUNT],
        }
    }

    /// Creates a new executor.
    pub fn new() -> Self {
        PriorityExecutor {
            ex: Default::default(),
        }
    }

    /// Spawns a task with the given priority. Lower numbers have better priority.
    pub fn spawn<T: Send + 'a>(
        &self,
        priority: usize,
        future: impl Future<Output = T> + Send + 'a,
    ) -> Task<T> {
        assert!(priority < PRIORTY_COUNT);
        self.ex[priority as usize].spawn(future)
    }

    /// Runs the executor forever.
    pub async fn run(&self) -> ! {
        //TODO: parametrize over PRIORITY_COUNT
        loop {
            let t0 = self.ex[0].run_n_loops(1);
            let t1 = self.ex[1].run_n_loops(1);
            let t2 = self.ex[2].run_n_loops(1);
            let t3 = self.ex[3].run_n_loops(1);
            let t4 = self.ex[4].run_n_loops(1);

            // Wait until one of the ticks completes, trying them in order from highest
            // priority to lowest priority.
            t0.or(t1).or(t2).or(t3).or(t4).await;
        }
    }
}
