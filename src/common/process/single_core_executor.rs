use super::task::{Task, TaskId};
use crate::arch;
use alloc::{collections::BTreeMap, sync::Arc, task::Wake};
use core::task::{Context, Poll, Waker};
use crossbeam_queue::SegQueue;

/// Global task executor / scheduler.
/// Single Core implementation.
pub struct SimpleExecutor {
    /// The list of tasks running in the system.
    tasks: BTreeMap<TaskId, Task>,

    /// The task queue to run.
    task_queue: Arc<SegQueue<TaskId>>,

    /// Cache for wakers.
    waker_cache: BTreeMap<TaskId, Waker>,

    /// List of tasks not yet run even once. These
    /// would be moved to `task_queue` once setup in
    /// all the data structures.
    new_task_list: Arc<SegQueue<Task>>,
}

impl SimpleExecutor {
    pub fn new() -> Self {
        Self {
            tasks: BTreeMap::new(),
            task_queue: Arc::new(SegQueue::new()),
            waker_cache: BTreeMap::new(),
            new_task_list: Arc::new(SegQueue::new()),
        }
    }

    /// Get the spawner that is responsible for creating new tasks.
    pub fn get_spawner(&self) -> SimpleSpawner {
        SimpleSpawner::new(self.new_task_list.clone())
    }

    /// Run the executor main loop.
    pub fn run(&mut self) -> ! {
        loop {
            self.spawn_new_tasks();
            self.run_ready_tasks();
            self.sleep_if_idle();
        }
    }

    fn spawn_new_tasks(&mut self) {
        while let Some(task) = self.new_task_list.pop() {
            self.spawn_internal(task)
        }
    }

    fn spawn_internal(&mut self, task: Task) {
        let task_id = task.id;
        if self.tasks.insert(task.id, task).is_some() {
            panic!("task with same ID already in tasks");
        }
        self.task_queue.push(task_id);
    }

    fn run_ready_tasks(&mut self) {
        // destructure `self` to avoid borrow checker errors
        #[allow(unused_variables)]
        let Self {
            tasks,
            task_queue,
            waker_cache,
            new_task_list,
        } = self;

        while let Some(task_id) = task_queue.pop() {
            let task = match tasks.get_mut(&task_id) {
                Some(task) => task,
                None => continue, // task no longer exists
            };
            let waker = waker_cache
                .entry(task_id)
                .or_insert_with(|| TaskWaker::new(task_id, task_queue.clone()));
            let mut context = Context::from_waker(waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => {
                    // task done -> remove it and its cached waker
                    tasks.remove(&task_id);
                    waker_cache.remove(&task_id);
                }
                Poll::Pending => {
                    // Goto next task.
                }
            }
        }
    }

    fn sleep_if_idle(&self) {
        arch::disable_interrupts();
        if self.task_queue.is_empty() && self.new_task_list.is_empty() {
            arch::enable_interrupts_and_halt();
        } else {
            arch::enable_interrupts();
        }
    }
}

struct TaskWaker {
    task_id: TaskId,
    task_queue: Arc<SegQueue<TaskId>>,
}

impl TaskWaker {
    fn new(task_id: TaskId, task_queue: Arc<SegQueue<TaskId>>) -> Waker {
        Waker::from(Arc::new(TaskWaker {
            task_id,
            task_queue,
        }))
    }

    fn wake_task(&self) {
        self.task_queue.push(self.task_id);
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    }
}

/// A struct that provides a way to spawn new tasks.
pub struct SimpleSpawner {
    /// List of tasks not yet run even once. These
    /// would be moved to `task_queue` once setup in
    /// all the data structures.
    new_task_list: Arc<SegQueue<Task>>,
}

impl SimpleSpawner {
    fn new(new_task_list: Arc<SegQueue<Task>>) -> Self {
        Self { new_task_list }
    }

    /// Spawn a new task onto the executor's list.
    pub fn spawn(&self, task: Task) {
        self.new_task_list.push(task);
    }
}
