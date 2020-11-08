use super::{
    allocator::{
        fixed_size_block::FixedSizeBlockAllocator,
        physical_memory_allocator::IPhysicalMemoryAllocator,
    },
    paging::{IMemoryMapper, MapperPermissions},
    Locked,
};
use crate::arch::globals;

/// This is the multi-threaded heap allocater used by the kernel.
#[global_allocator]
static KERNEL_ALLOCATOR: Locked<FixedSizeBlockAllocator> =
    Locked::new(FixedSizeBlockAllocator::new());

/// Initialize the heap.
pub fn initialize_heap(
    mapper: &mut impl IMemoryMapper,
    frame_allocator: &mut dyn IPhysicalMemoryAllocator,
) -> Result<(), &'static str> {
    mapper.map_with_alloc(
        globals::KERNEL_HEAP_START as *const u8,
        globals::KERNEL_HEAP_SIZE_INITIAL,
        MapperPermissions::WRITE,
        frame_allocator,
    )?;

    info!(target: "heap", "Kernel heap initialized at {:x} with size {} MB and a max of {} MB", 
        globals::KERNEL_HEAP_START,
        globals::KERNEL_HEAP_SIZE_INITIAL / 1024 / 1024,
        globals::KERNEL_HEAP_SIZE_TOTAL / 1024 / 1024);

    unsafe {
        KERNEL_ALLOCATOR
            .lock()
            .init(globals::KERNEL_HEAP_START, globals::KERNEL_HEAP_SIZE_TOTAL);
    }

    Ok(())
}
