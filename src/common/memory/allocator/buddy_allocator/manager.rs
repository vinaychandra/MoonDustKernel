//! Functionality to generate and manage multiple buddy allocators.

use super::buddy_allocator::BuddyAllocator;
use crate::common::memory::allocator::boot_frame_allocator::BootFrameAllocator;
use alloc::alloc::{GlobalAlloc, Layout};
use alloc::vec::Vec;
use core::ptr::null_mut;
use spin::{Mutex, RwLock};

/// Result from a request of memory area.
enum MemAreaRequest {
    /// Able to return a memory area with the given size.
    Success((u64, u64)),

    /// Unable to satisfy the memory request. So, return
    /// multiple values of smaller sizes.
    SmallerThanReq((u64, u64), Option<(u64, u64)>),

    /// Failed to return the memory area for the request.
    /// Can happen when there are no more pages left.
    Fail,
}

/// Buddy allocation manager.
pub struct BuddyAllocatorManager {
    /// List of buddy allocaters. Each one is wrapped
    /// in a Mutex so that multiple processors can access
    /// them separately without any race conditions.
    buddy_allocators: RwLock<Vec<Mutex<BuddyAllocator>>>,
}

impl BuddyAllocatorManager {
    fn get_mem_area_with_size(
        frame_alloc: &mut BootFrameAllocator,
        mem_size: u64,
        frame_size: u64,
    ) -> MemAreaRequest {
        // This function tries to find a continuous memory area as big as the one requested by
        // pulling pages from the frame allocator. If it doesn't find an area big enough immediately,
        // it might return one or two smaller ones (so that we don't leave memory unused for no reason
        // if it doesn't fit our purposes).
        if let Some(first_page) = frame_alloc.alloc(frame_size as usize, frame_size as usize) {
            let first_page = first_page as u64;
            let first_addr = first_page;
            let mut last_addr = first_addr + frame_size;
            // Keep pulling pages from the frame allocator until we hit the required memory size
            // or until we run out of memory or we get a block that is not after the previous block received.
            while let Some(next_page) = frame_alloc.alloc(frame_size as usize, frame_size as usize)
            {
                if next_page == last_addr as usize {
                    last_addr += frame_size;
                } else {
                    // TODO: A page can be leaked here because we do not return that
                    // page in case of smaller size.
                    break;
                }
                if last_addr - first_addr == mem_size {
                    break;
                }
            }
            // If we found a memory area big enough, great! Return it.
            if last_addr - first_addr == mem_size {
                MemAreaRequest::Success((first_addr, last_addr))
            } else {
                // If we found a smaller memory block, get the largest piece that is a power of 2
                // and also greater than a page size. We can use that to make a smaller buddy allocator.
                if let Some(first_memarea) =
                    Self::get_largest_page_multiple(first_addr, last_addr, frame_size)
                {
                    // Try to form a second such block with the left-over memory to not waste it.
                    let second_memarea =
                        Self::get_largest_page_multiple(first_memarea.1, last_addr, frame_size);

                    // TODO: Add more blocks
                    MemAreaRequest::SmallerThanReq(first_memarea, second_memarea)
                } else {
                    // This should never happen but let's be safe
                    MemAreaRequest::Fail
                }
            }
        } else {
            // Couldn't even pull a single page from the frame allocator :(
            MemAreaRequest::Fail
        }
    }

    fn get_largest_page_multiple(start: u64, end: u64, frame_size: u64) -> Option<(u64, u64)> {
        // Given a start and end address, try to find the largest memory size that can fit into that
        // area that is also a left shift of a FRAME_SIZE (ie. 4096, 8192, 16384 etc.)
        // We need this because our buddy allocator needs a memory area whose size is a power of 2
        // in order to be able to split it cleanly and efficiently.
        // Also, the smallest size of that memory area will be the FRAME_SIZE.
        let mem_len = end - start;
        if mem_len == 0 {
            None
        } else {
            // double page_mult while it still fits in this mem area
            let mut page_mult = frame_size;
            while page_mult <= mem_len {
                page_mult <<= 1;
            }
            // we went over the limit so divide by two
            page_mult >>= 1;
            let start_addr = start;
            let end_addr = start + page_mult;
            Some((start_addr, end_addr))
        }
    }
}

impl BuddyAllocatorManager {
    /// Create an empty buddy allocator list.
    pub fn new() -> BuddyAllocatorManager {
        let buddy_allocators = RwLock::new(Vec::with_capacity(32));
        BuddyAllocatorManager { buddy_allocators }
    }

    /// Add a new buddy allocator to the list with these specs.
    pub fn add_memory_area(&self, start_addr: u64, end_addr: u64, block_size: u16) {
        // As each one has some dynamic internal structures, we try to make it so that none of these
        // has to use itself when allocating these.
        let new_buddy_alloc = Mutex::new(BuddyAllocator::new(start_addr, end_addr, block_size));
        // On creation the buddy allocator constructor might lock the list of buddy allocators
        // due to the fact that it allocates memory for its internal structures (except for the very
        // first buddy allocator which still uses the previous, dumb allocator).
        // Therefore we first create it and then we lock the list in order to push the new
        // buddy allocator to the list.
        self.buddy_allocators.write().push(new_buddy_alloc);
    }

    /// Find and create a buddy allocator with the memory area requested.
    pub fn add_mem_area_with_size(
        &self,
        frame_alloc: &mut BootFrameAllocator,
        mem_size: u64,
        block_size: u16,
        frame_size: u64,
    ) -> bool {
        // We use get_mem_area_with_size first to find the memory area.
        // That function might instead find one (or two) smaller memory areas if the current
        // memory block that we're pulling memory from isn't big enough.
        // In that case add these smaller ones but keep looping until we get a memory block
        // as big as the one requested.
        // If we run out of memory, we simply return false.
        loop {
            match Self::get_mem_area_with_size(frame_alloc, mem_size, frame_size) {
                // Success! Found a memory area big enough for our purposes.
                MemAreaRequest::Success((mem_start, mem_end)) => {
                    debug!(
                        target: "memory",
                        "[BAM] Adding requested mem area to BuddyAlloc: {} to {} ({} KB)\n",
                        mem_start,
                        mem_end,
                        (mem_end - mem_start) / 1024
                    );
                    self.add_memory_area(mem_start, mem_end, block_size);
                    return true;
                }
                // Found one or two smaller memory areas instead, insert them and keep looking.
                MemAreaRequest::SmallerThanReq((mem_start, mem_end), second_area) => {
                    self.add_memory_area(mem_start, mem_end, block_size);
                    debug!(
                        target: "memory",
                        "[BAM] Adding smaller mem area to BuddyAlloc: {} to {} ({} KB)\n",
                        mem_start,
                        mem_end,
                        (mem_end - mem_start) / 1024
                    );
                    if let Some((mem_start, mem_end)) = second_area {
                        self.add_memory_area(mem_start, mem_end, block_size);
                        debug!(
                            target: "memory",
                            "[BAM] Adding smaller mem area to BuddyAlloc: {} to {} ({} KB)\n",
                            mem_start,
                            mem_end,
                            (mem_end - mem_start) / 1024
                        );
                    }
                }
                // Ran out of memory! Return false.
                MemAreaRequest::Fail => {
                    debug!(
                        target: "memory",
                        "[BAM] Failed to find mem area big enough for BuddyAlloc: {}\n",
                        mem_size
                    );
                    return false;
                }
            }
        }
    }
}

unsafe impl GlobalAlloc for BuddyAllocatorManager {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // Loop through the list of buddy allocators until we can find one that can give us
        // the requested memory.
        // TODO: Figure out what happens if all are locked
        let allocation =
            self.buddy_allocators
                .read()
                .iter()
                .enumerate()
                .find_map(|(i, allocator)| {
                    // for each allocator
                    allocator.try_lock().and_then(|mut allocator| {
                        allocator
                            .alloc(layout.size(), layout.align())
                            .map(|allocation| {
                                // try allocating until one succeeds and return this allocation
                                trace!(
                                    target: "memory",
                                    "[BAM] - BuddyAllocator #{} allocated {} bytes\n",
                                    i,
                                    layout.size()
                                );
                                trace!(target: "memory", "{}\n", *allocator);
                                allocation
                            })
                    })
                });
        allocation.map(|phys| phys as *mut u8).unwrap_or(null_mut())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let phys_addr = ptr as u64;

        for (i, allocator_mtx) in self.buddy_allocators.read().iter().enumerate() {
            // for each allocator
            if let Some(mut allocator) = allocator_mtx.try_lock() {
                // find the one whose memory range contains this address
                if allocator.contains(phys_addr) {
                    // deallocate using this allocator!
                    allocator.dealloc(phys_addr, layout.size(), layout.align());
                    trace!(
                        target: "memory",
                        "[BAM] - BuddyAllocator #{} de-allocated {} bytes\n",
                        i,
                        layout.size()
                    );
                    trace!(target: "memory", "[BAM] {}\n", *allocator);
                    return;
                }
            }
        }
        info!(
            target: "memory",
            "[BAM]! Could not de-allocate virtual address: {} / Memory lost",
            phys_addr
        );
    }
}

impl core::fmt::Display for BuddyAllocatorManager {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(
            f,
            "BuddyAllocationManager: Count: {}",
            self.buddy_allocators.read().len()
        )?;

        for alloc in self.buddy_allocators.read().iter() {
            writeln!(f, "{}", *alloc.lock())?;
        }

        Ok(())
    }
}
