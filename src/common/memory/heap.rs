use super::{
    allocator::{
        fixed_size_block::FixedSizeBlockAllocator,
        physical_memory_allocator::IPhysicalMemoryAllocator,
    },
    paging::{MapperPermissions, MemoryMapper},
    Locked,
};
use crate::arch::globals;

#[global_allocator]
static KERNEL_ALLOCATOR: Locked<FixedSizeBlockAllocator> =
    Locked::new(FixedSizeBlockAllocator::new());

/// Initialize the heap.
pub fn initialize_heap(
    mapper: &mut impl MemoryMapper,
    frame_allocator: &mut impl IPhysicalMemoryAllocator,
) -> Result<(), &'static str> {
    mapper.map_with_alloc(
        globals::KERNEL_HEAP_START as *const u8,
        globals::KERNEL_HEAP_SIZE_INITIAL,
        MapperPermissions::WRITE,
        frame_allocator,
    )?;

    unsafe {
        KERNEL_ALLOCATOR
            .lock()
            .init(globals::KERNEL_HEAP_START, globals::KERNEL_HEAP_SIZE_TOTAL);
    }

    Ok(())
}
