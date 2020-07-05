//! Physicam memory allocation support.

use super::{
    boot_frame_allocator::BootFrameAllocator, buddy_allocator::manager::BuddyAllocatorManager,
};
use conquer_once::spin::OnceCell;
use core::alloc::GlobalAlloc;
use core::alloc::Layout;

pub trait IPhysicalMemoryAllocator {
    /// Allocation for physical memory.
    fn allocate_physical_memory(&self, layout: Layout) -> *mut u8;

    /// Deallocate physical memory.
    fn deallocate_physical_memory(&self, layout: Layout, memory: *mut u8);
}

///////////////////////////
// PHYSICAL MEMORY SUPPORT
///////////////////////////

static PHYSICAL_MEMORY_PROVIDER: OnceCell<PhysicalMemoryAllocator> = OnceCell::uninit();

struct PhysicalMemoryAllocator {
    manager: BuddyAllocatorManager,
}

impl PhysicalMemoryAllocator {
    pub fn new(frame_alloc: &mut BootFrameAllocator, frame_size: usize) -> PhysicalMemoryAllocator {
        let frame_size = frame_size as u64;
        let buddy_manager = BuddyAllocatorManager::new();

        // Get our current buddy allocator

        // Allocate increasingly large memory areas.
        // The previously created buddy allocator (which uses a single page) will be used to back
        // the first of these areas' internal structures to avoid the area having to use itself.
        // Then the first two areas will be used to support the third, etc.
        // Until we can support 1GiB buddy allocators (the final type) which need a big
        // amount of continuous backing memory (some MiB for the is_split bitmap plus
        // several Vecs for the free lists).
        buddy_manager.add_mem_area_with_size(
            frame_alloc,
            frame_size * 8,
            frame_size as u16,
            frame_size,
        );
        buddy_manager.add_mem_area_with_size(
            frame_alloc,
            frame_size * 64,
            frame_size as u16,
            frame_size,
        );
        buddy_manager.add_mem_area_with_size(frame_alloc, 1 << 24, frame_size as u16, frame_size);
        while buddy_manager.add_mem_area_with_size(
            frame_alloc,
            1 << 30,
            frame_size as u16,
            frame_size,
        ) {}
        info!(target: "BuddyAllocationManager", "{}", buddy_manager);
        PhysicalMemoryAllocator {
            manager: buddy_manager,
        }
    }
}

impl IPhysicalMemoryAllocator for PhysicalMemoryAllocator {
    fn allocate_physical_memory(&self, layout: Layout) -> *mut u8 {
        unsafe { self.manager.alloc(layout) }
    }
    fn deallocate_physical_memory(&self, layout: Layout, memory: *mut u8) {
        unsafe { self.manager.dealloc(memory, layout) }
    }
}

pub fn initialize_physical_memory_allocator(
    frame_alloc: &mut BootFrameAllocator,
    frame_size: usize,
) {
    PHYSICAL_MEMORY_PROVIDER.init_once(|| PhysicalMemoryAllocator::new(frame_alloc, frame_size));
}

pub fn get_physical_memory_allocator() -> &'static impl IPhysicalMemoryAllocator {
    PHYSICAL_MEMORY_PROVIDER.get().unwrap()
}
