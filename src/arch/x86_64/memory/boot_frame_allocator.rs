//! Frame allocator. Used to allocate frames. This can run without
//! pre-allocated frames.

use bootloader::bootinfo::{FrameRange, MemoryRegion, MemoryRegionType};
use x86_64::{
    structures::paging::{FrameAllocator, PhysFrame, Size4KiB},
    PhysAddr,
};

const PAGE_SIZE: u64 = 4096;

/// Structure for a frame allocator.
///
/// This allocator can run without pre-allocated frames. This allocator
/// does not support deallocation of frames.
/// This allocator works on memory regions with provided page sizes.
///
/// The frame allocator uses a group of regions. There is an active region from
/// which simple allocations are done. Once a region is complete, the next region
/// is used for allocation.
pub struct BootFrameAllocator {
    /// Iterator for different Memory regions.
    memory_iter: &'static [MemoryRegion],

    /// The current memory region being allocated from.
    /// This is an index into memory_iter
    current_memory_region: usize,

    /// The next memory region being allocated from.
    /// This is an index into memory_iter
    next_memory_region: usize,

    /// Next physical frame to allocate.
    next_frame_to_allocate: u64,
}

#[allow(dead_code)]
impl BootFrameAllocator {
    /// Create a new frame allocator with the provided memory area iterator.
    pub fn new(memory_iter: &'static [MemoryRegion]) -> BootFrameAllocator {
        let mut first_memory_region: MemoryRegion;
        let mut index = 0;
        loop {
            first_memory_region = memory_iter[index];
            if first_memory_region.region_type == MemoryRegionType::Usable {
                break;
            }

            index += 1;
        }

        // Potential bug: No usable memory
        BootFrameAllocator {
            memory_iter,
            current_memory_region: index,
            next_frame_to_allocate: first_memory_region.range.start_frame_number,
            next_memory_region: index + 1,
        }
    }

    /// Get contiguous area for the memory instead of a page at a time.
    pub fn get_contiguous_area(&mut self) -> Option<MemoryRegion> {
        let mut to_return: Option<MemoryRegion>;

        loop {
            if self.next_memory_region >= self.memory_iter.len() {
                to_return = None;
                break;
            }

            to_return = Some(self.memory_iter[self.next_memory_region]);
            if to_return.unwrap().region_type == MemoryRegionType::Usable {
                break;
            }

            self.next_memory_region += 1;
        }

        if to_return != None {
            return to_return;
        }

        // Go to current memory region only after exhausting everything.
        if self.current_memory_region >= self.memory_iter.len() {
            return None;
        }

        let next_mem_region = self.memory_iter[self.current_memory_region];
        self.current_memory_region = self.memory_iter.len();

        return Some(MemoryRegion {
            range: FrameRange {
                start_frame_number: self.next_frame_to_allocate,
                end_frame_number: next_mem_region.range.end_frame_number,
            },
            region_type: MemoryRegionType::Usable,
        });
    }

    fn alloc(&mut self, size: usize, alignment: usize) -> Option<usize> {
        if self.current_memory_region >= self.memory_iter.len() {
            return None;
        }

        let size64: u64 = size as u64;

        let frame_to_allocate = self.next_frame_to_allocate;
        let address_to_allocate: u64 = frame_to_allocate * PAGE_SIZE;

        if address_to_allocate + size64
            <= self.memory_iter[self.current_memory_region]
                .range
                .end_addr()
        {
            self.next_frame_to_allocate = ((address_to_allocate + size64) / PAGE_SIZE) + 1;
            return Some(address_to_allocate as usize);
        }

        // Cannot allocate in the current region. Skip current region. We might leak a small region.
        loop {
            self.current_memory_region = self.next_memory_region;
            if self.current_memory_region >= self.memory_iter.len()
                || self.memory_iter[self.current_memory_region].region_type
                    == MemoryRegionType::Usable
            {
                break;
            }
        }

        self.next_memory_region = self.current_memory_region + 1;

        if self.current_memory_region >= self.memory_iter.len() {
            return None;
        }

        return self.alloc(size, alignment);
    }

    fn dealloc(&mut self, _addr: u64, _size: usize, _alignment: usize) {
        unimplemented!()
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let address_to_return = self.alloc(4096, 4096);
        let frame: Option<PhysFrame> = address_to_return
            .map(|addr| PhysFrame::from_start_address(PhysAddr::new(addr as u64)).unwrap());

        frame
    }
}
