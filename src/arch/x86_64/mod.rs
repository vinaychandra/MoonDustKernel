pub mod bootstrap;
pub mod gdt;
pub mod globals;
pub mod interrupts;
pub mod memory;
pub mod process;
pub mod serial;

use moondust_utils::buddy_system_allocator;
use x86_64::VirtAddr;

use self::serial::SerialLogger;
use crate::common::memory::fixed_size_block;

/// Logger that uses serial to output logs.
/// Architecture level logs for x86_64.
pub static LOGGER: SerialLogger = SerialLogger;

/// Heap for the kernel.
#[global_allocator]
static KERNEL_HEAP_ALLOCATOR: fixed_size_block::LockedHeap = fixed_size_block::LockedHeap::empty();

//TODO: Provide a better number than 40
/// The global physical memory allocator for the environment.
pub static PHYSICAL_MEMORY_ALLOCATOR: buddy_system_allocator::LockedHeap<40> =
    buddy_system_allocator::LockedHeap::new();

pub mod cpu_locals {
    use core::cell::{Cell, RefCell};

    use alloc::sync::Arc;
    use moondust_utils::sync::mutex::Mutex;

    pub use super::interrupts::apic::LAPIC;
    pub use super::interrupts::apic::PROCESSOR_ID;
    use super::memory::kernel_page_table::KernelPageTable;

    #[thread_local]
    pub static CURRENT_PAGE_TABLE: RefCell<Option<Arc<Mutex<KernelPageTable>>>> =
        RefCell::new(None);

    #[thread_local]
    pub static CURRENT_THREAD_ID: Cell<usize> = Cell::new(0);
}

pub fn is_kernel_mode(addr: u64) -> bool {
    if VirtAddr::try_new(addr).is_err() {
        return false;
    }

    (addr & (1 << 62)) > 0
}
