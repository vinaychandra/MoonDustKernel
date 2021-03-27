use core::{
    alloc::{GlobalAlloc, Layout, LayoutError},
    ptr,
};

use x86_64::{
    structures::paging::{FrameAllocator, FrameDeallocator, PhysFrame, Size4KiB},
    PhysAddr,
};

use crate::arch::{globals, PHYSICAL_MEMORY_ALLOCATOR};

pub fn get_frame_allocator() -> impl FrameAllocator<Size4KiB> {
    PhysicalMemoryAllocatorWrapper { zeroed: false }
}

pub fn get_frame_deallocator() -> impl FrameDeallocator<Size4KiB> {
    PhysicalMemoryAllocatorWrapper { zeroed: false }
}

pub fn get_frame_allocator_zeroed() -> impl FrameAllocator<Size4KiB> {
    PhysicalMemoryAllocatorWrapper { zeroed: true }
}

struct PhysicalMemoryAllocatorWrapper {
    zeroed: bool,
}

unsafe impl FrameAllocator<Size4KiB> for PhysicalMemoryAllocatorWrapper {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let layout = Layout::from_size_align(4096, 4096).unwrap();
        let mem;
        if self.zeroed {
            mem = unsafe { PHYSICAL_MEMORY_ALLOCATOR.alloc_zeroed(layout) };
        } else {
            mem = unsafe { PHYSICAL_MEMORY_ALLOCATOR.alloc(layout) };
        }

        if mem == ptr::null_mut() {
            return None;
        }

        let phys_addr = PhysAddr::new((mem as u64) - globals::MEM_MAP_OFFSET_LOCATION);
        return Some(PhysFrame::from_start_address(phys_addr).unwrap());
    }
}

impl FrameDeallocator<Size4KiB> for PhysicalMemoryAllocatorWrapper {
    unsafe fn deallocate_frame(&mut self, frame: PhysFrame<Size4KiB>) {
        const LAYOUT: Result<Layout, LayoutError> = Layout::from_size_align(4096, 4096);

        let phys_addr = frame.start_address();
        let virt_addr = phys_addr.as_u64() + globals::MEM_MAP_OFFSET_LOCATION;

        unsafe {
            PHYSICAL_MEMORY_ALLOCATOR.dealloc(virt_addr as *mut u8, LAYOUT.unwrap());
        }
    }
}
