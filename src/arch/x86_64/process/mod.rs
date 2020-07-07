pub mod task;

use super::{
    gdt, globals,
    memory::{paging, tables},
};
use crate::common::{
    memory::{allocator::physical_memory_allocator, stack::Stack},
    process::{Process, ProcessControlBlock},
};
use alloc::boxed::Box;
use x86_64::{
    structures::paging::{OffsetPageTable, PageTable},
    PhysAddr, VirtAddr,
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
            let pcb = ProcessControlBlock {
                mapper,
                kernel_stack,
                page_table: Box::from_raw(page_table),
            };

            Process::new_with_pcb(pcb)
        }
    }

    /// Activate this process as needed.
    pub fn activate(&self) {
        let pcb = &self.pcb.read();
        let page_table = pcb.page_table.get_addr();
        let phys = pcb
            .mapper
            .virt_to_phys(page_table as *const u8)
            .expect("Cannot find Physical Address for page table to activate.");
        paging::activate_page_table(PhysAddr::new(phys as u64));

        gdt::setup_usermode();
    }
}
