use super::memory::{
    paging::{IMemoryMapper, IPageTable},
    stack::Stack,
};
use alloc::boxed::Box;

pub struct Process {
    pub mapper: Box<dyn IMemoryMapper>,
    pub kernel_stack: Stack,
    pub page_table: Box<dyn IPageTable>,
}

impl Process {}
