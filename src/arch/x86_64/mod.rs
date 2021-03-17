pub mod bootstrap;
pub mod gdt;
pub mod globals;
pub mod interrupts;
pub mod memory;
pub mod process;
pub mod serial;

use moondust_utils::buddy_system_allocator;

use self::serial::SerialLogger;
use crate::common::memory::fixed_size_block;

/// Logger that uses serial to output logs.
/// Architecture level logs for x86_64.
pub static LOGGER: SerialLogger = SerialLogger;

#[global_allocator]
static KERNEL_HEAP_ALLOCATOR: fixed_size_block::LockedHeap = fixed_size_block::LockedHeap::empty();

//TODO: Provide a better number than 40
pub static PHYSICAL_MEMORY_ALLOCATOR: buddy_system_allocator::LockedHeap<40> =
    buddy_system_allocator::LockedHeap::new();

pub use interrupts::apic::LAPIC;
pub use interrupts::apic::PROCESSOR_ID;
