use super::{globals, memory::tables};
use crate::common::{
    memory::{allocator::physical_memory_allocator, stack::Stack},
    process::Process,
};
use alloc::boxed::Box;
use x86_64::{
    structures::paging::{OffsetPageTable, PageTable},
    VirtAddr,
};

impl Process {
    pub fn new() -> Self {
        let pt = tables::create_new_kernel_only_table_from_current();
        let page_table = Box::into_raw(pt) as *mut PageTable;

        unsafe {
            let mut mapper = OffsetPageTable::new(
                page_table.as_mut().unwrap(),
                VirtAddr::new(globals::MEM_MAP_LOCATION),
            );
            let allocator = physical_memory_allocator::get_physical_memory_allocator();

            let kernel_stack =
                Stack::new_kernel_stack(globals::KERNEL_STACK_PER_PROCESS, &mut mapper, allocator);
            let mapper = Box::new(mapper);
            Process {
                mapper,
                kernel_stack,
                page_table: Box::from_raw(page_table),
            }
        }
    }
}
