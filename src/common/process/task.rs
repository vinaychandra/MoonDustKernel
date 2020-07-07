use super::ProcessControlBlock;
use crate::common::memory::{allocator::physical_memory_allocator, stack::Stack};
use alloc::{boxed::Box, sync::Arc};
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use spin::RwLock;

pub struct Task {
    pub id: TaskId,
    future: Pin<Box<dyn Future<Output = ()>>>,

    pub process: Arc<RwLock<ProcessControlBlock>>,
    pub stack: Stack,
}

impl Task {
    pub fn new(
        stack_size: usize,
        future: impl Future<Output = ()> + 'static,
        process: Arc<RwLock<ProcessControlBlock>>,
    ) -> Task {
        let allocator = physical_memory_allocator::get_physical_memory_allocator();
        let stack = Stack::new_user_stack(
            stack_size,
             process.write().mapper.as_mut() ,
            allocator,
        );
        Task {
            id: TaskId::new(),
            future: Box::pin(future),
            process,
            stack,
        }
    }

    pub fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TaskId(u32);

impl TaskId {
    fn new() -> Self {
        TaskId(
            super::TASK_ID_GENERATOR
                .pop()
                .expect("Cannot allocate task id."),
        )
    }
}
