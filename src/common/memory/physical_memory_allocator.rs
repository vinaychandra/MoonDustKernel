use core::alloc::{GlobalAlloc, Layout};

use buddy_system_allocator::LockedHeap;

use crate::arch::globals::{self, MEM_MAP_OFFSET_LOCATION};

pub trait IPhysicalMemoryAllocator {
    /// Allocation for physical memory. Returns virtual address in mmap region.
    fn allocate_physical_memory(&self, layout: Layout) -> *mut u8;

    /// Deallocate physical memory. Returns virtual address in mmap region.
    fn deallocate_physical_memory(&self, layout: Layout, memory: *mut u8);

    fn allocate_physical_memory_pe(&self, layout: Layout) -> u64 {
        let addr = self.allocate_physical_memory(layout);
        let offset = globals::MEM_MAP_OFFSET_LOCATION;
        addr as u64 - offset
    }
}

impl IPhysicalMemoryAllocator for LockedHeap {
    fn allocate_physical_memory(&self, layout: Layout) -> *mut u8 {
        unsafe { self.alloc(layout) }
    }

    fn deallocate_physical_memory(&self, layout: Layout, memory: *mut u8) {
        unsafe { self.dealloc(memory, layout) }
    }
}
