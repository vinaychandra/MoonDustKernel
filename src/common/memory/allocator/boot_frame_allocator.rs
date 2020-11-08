//! Frame allocator. Used to allocate frames. This can run without
//! pre-allocated frames.

use super::physical_memory_allocator::IPhysicalMemoryAllocator;
use crate::{bootboot::MMapEnt, bootboot2::MMapEntType};
use core::cell::UnsafeCell;
use linked_list_allocator::align_up;

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
    memory_iter: &'static [MMapEnt],

    /// The current memory region being allocated from.
    /// This is an index into memory_iter
    current_memory_region: usize,

    /// The next memory region being allocated from.
    /// This is an index into memory_iter
    next_memory_region: usize,

    /// Next physical address to allocate.
    next_address_to_allocate: usize,
}

#[allow(dead_code)]
impl BootFrameAllocator {
    /// Create a new boot frame allocator with memory map entries.
    pub fn new(memory_iter: &'static [MMapEnt]) -> BootFrameAllocator {
        let mut first_memory_region: MMapEnt;
        let mut index = 0;
        loop {
            first_memory_region = memory_iter[index];
            if first_memory_region.is_free() {
                break;
            }

            index += 1;
        }

        // Potential bug: No usable memory
        BootFrameAllocator {
            memory_iter,
            current_memory_region: index,
            next_address_to_allocate: first_memory_region.ptr() as usize,
            next_memory_region: index + 1,
        }
    }

    /// Get contiguous area for the memory instead of a page at a time.
    pub fn get_contiguous_area(&mut self) -> Option<MMapEnt> {
        let mut to_return: Option<MMapEnt>;

        loop {
            if self.next_memory_region >= self.memory_iter.len() {
                to_return = None;
                break;
            }

            to_return = Some(self.memory_iter[self.next_memory_region]);
            if to_return.unwrap().get_type() == MMapEntType::Free {
                break;
            }

            self.next_memory_region += 1;
        }

        if let Some(_) = to_return {
            return to_return;
        }

        // Go to current memory region only after exhausting everything.
        if self.current_memory_region >= self.memory_iter.len() {
            return None;
        }

        let next_mem_region = self.memory_iter[self.current_memory_region];
        self.current_memory_region = self.memory_iter.len();

        let mut area_size: usize = 0xF;

        while area_size & 0xF != 0 {
            // Mmap doesn't support smaller than 0xF aligned sizes.
            // Portential bug: final value > max
            self.next_address_to_allocate += 1;
            area_size =
                next_mem_region.size() - (self.next_address_to_allocate - next_mem_region.ptr());
        }

        let mut ent = MMapEnt {
            ptr: self.next_address_to_allocate as u64,
            size: area_size as u64,
        };
        ent.set_type(MMapEntType::Free);
        return Some(ent);
    }

    pub fn alloc(&mut self, size: usize, alignment: usize) -> Option<usize> {
        if self.current_memory_region >= self.memory_iter.len() {
            return None;
        }

        let mut address_to_allocate = self.next_address_to_allocate;
        address_to_allocate = align_up(address_to_allocate as usize, alignment);

        if address_to_allocate + size
            <= self.memory_iter[self.current_memory_region].end_address() as usize
        {
            self.next_address_to_allocate = address_to_allocate + size;

            // TODO: Is this required?
            // Zero from "address_to_allocate" to "address_to_allocate" +size
            // for addr in address_to_allocate..address_to_allocate + size {
            //     unsafe { *(addr as *mut u8) = 0 };
            // }

            return Some(address_to_allocate as usize);
        }

        // Cannot allocate in the current region. Skip current region. We might leak a small region.
        loop {
            self.current_memory_region = self.next_memory_region;
            if self.current_memory_region >= self.memory_iter.len()
                || self.memory_iter[self.current_memory_region].is_free()
            {
                break;
            } else {
                self.next_memory_region += 1;
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

impl IPhysicalMemoryAllocator for UnsafeCell<BootFrameAllocator> {
    fn allocate_physical_memory(&self, layout: core::alloc::Layout) -> *mut u8 {
        let allocator = self.get();
        let bfa = unsafe { allocator.as_mut().unwrap() };
        bfa.alloc(layout.size(), layout.align())
            .map_or_else(|| core::ptr::null_mut(), |a| a as *mut u8)
    }
    fn deallocate_physical_memory(&self, _layout: core::alloc::Layout, _memory: *mut u8) {
        unimplemented!()
    }
}
