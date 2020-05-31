use crate::memory::{fixed_size_block::FixedSizeBlockAllocator, Locked};
use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
    },
    VirtAddr,
};

const KERNEL_HEAP_START: usize = 0x_FFFF_C000_0000_0000;
const KERNEL_HEAP_SIZE: usize = 1024 * 1024; // 1 MB

#[global_allocator]
static KERNEL_ALLOCATOR: Locked<FixedSizeBlockAllocator> =
    Locked::new(FixedSizeBlockAllocator::new());

pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let heap_start = VirtAddr::new(KERNEL_HEAP_START as u64);
        let heap_end = heap_start + KERNEL_HEAP_SIZE - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe { mapper.map_to(page, frame, flags, frame_allocator)?.flush() };
    }

    unsafe {
        KERNEL_ALLOCATOR
            .lock()
            .init(KERNEL_HEAP_START, KERNEL_HEAP_SIZE);
    }

    Ok(())
}
