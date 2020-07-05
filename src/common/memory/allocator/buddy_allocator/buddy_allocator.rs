//! Generic Buddy allocator.
//!
//! # Notes
//! https://en.wikipedia.org/wiki/Buddy_memory_allocation
//! https://nfil.dev/kernel/rust/coding/rust-buddy-allocator/

use alloc::vec::Vec;
use core::cmp;

/// A buddy allocator structure. This buddy allocator only
/// manages a memory of size that is a power of 2.
pub struct BuddyAllocator {
    /// The address at which this instance's managed
    /// memory starts.
    start_address: u64,

    /// The address at which this instance's managed
    /// memory ends.
    end_address: u64,

    /// The number of non-leaf levels for this instance.
    /// L0 is the largest block. L0 breaks into two L1s
    /// and so on.
    num_levels: u8,

    /// The size of each block on the leaf level. This is
    /// the minimum size of a memory block that we can return.
    block_size: u16,

    /// A free_list for a level is the list of blocks in that
    /// level that are not in use.
    free_lists: Vec<Vec<u32>>,
}

impl core::fmt::Display for BuddyAllocator {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut res = writeln!(
            f,
            "  Start: {} / End: {} / Levels: {} / Block size: {} / Max alloc: {}",
            self.start_address,
            self.end_address,
            self.num_levels + 1,
            self.block_size,
            (self.block_size as usize) << (self.num_levels as usize),
        );
        res = res.and_then(|_| write!(f, "  Free lists: "));
        for i in 0usize..(self.num_levels as usize + 1) {
            res = res.and_then(|_| write!(f, "{} in L{} / ", self.free_lists[i].len(), i));
        }
        res
    }
}

impl BuddyAllocator {
    /// Max size that can be supported by this buddy allocator.
    fn max_size(&self) -> usize {
        (self.block_size as usize) << (self.num_levels as usize)
    }

    /// Returns whether a given physical address belongs to this allocator.
    pub fn contains(&self, addr: u64) -> bool {
        addr >= self.start_address && addr < self.end_address
    }

    /// Create a new buddy allocator. The start and end areas difference must be a
    /// power of 2. The `block_size` is the size of a block on the leaf level.
    pub fn new(start_address: u64, end_address: u64, block_size: u16) -> BuddyAllocator {
        // number of levels excluding the leaf level
        let mut num_levels: u8 = 0;
        while ((block_size as u64) << num_levels as u64) < end_address - start_address {
            num_levels += 1;
        }
        // vector of free lists
        let mut free_lists: Vec<Vec<u32>> = Vec::with_capacity((num_levels + 1) as usize);
        // Initialize each free list with a small capacity (in order to use the current allocator
        // at least for the first few items and not the one that will be in use when we're actually
        // using this as the allocator as this might lead to this allocator using itself and locking)
        for _ in 0..(num_levels + 1) {
            free_lists.push(Vec::with_capacity(4));
        }
        // The top-most block is (the only) free for now!
        free_lists[0].push(0);
        // We need 1<<levels bits to store which blocks are split (so 1<<(levels-3) bytes)
        BuddyAllocator {
            start_address,
            end_address,
            num_levels,
            block_size,
            free_lists,
        }
    }
}

// Allocation support.
impl BuddyAllocator {
    /// Find the level of this allocator than can accommodate the required memory size.
    fn req_size_to_level(&self, size: usize) -> Option<usize> {
        let max_size = self.max_size();
        if size > max_size {
            // can't allocate more than the maximum size for this allocator!
            None
        } else {
            // find the largest block level that can support this size
            let mut next_level = 1;
            while (max_size >> next_level) >= size {
                next_level += 1;
            }
            // ...but not larger than the max level!
            let req_level = cmp::min(next_level - 1, self.num_levels as usize);
            Some(req_level)
        }
    }

    /// Get a block from the free list at this level or split a block above and
    /// return one of the splitted blocks.
    fn get_free_block(&mut self, level: usize) -> Option<u32> {
        self.free_lists[level]
            .pop()
            .or_else(|| self.split_level(level))
    }

    fn split_level(&mut self, level: usize) -> Option<u32> {
        // We reached the maximum level, we can't split anymore! We can't support this allocation.
        if level == 0 {
            None
        } else {
            self.get_free_block(level - 1).map(|block| {
                // Get a block from 1 level above us and split it.
                // We push the second of the splitted blocks to the current free list
                // and we return the other one as we now have a block for this allocation!
                self.free_lists[level].push(block * 2 + 1);
                block * 2
            })
        }
    }

    /// Allocate a new memory area with a size and alignment.
    pub fn alloc(&mut self, size: usize, alignment: usize) -> Option<u64> {
        // We should always be aligned due to how the buddy allocator works
        // (everything will be aligned to block_size bytes).
        // If we need in some case that we are aligned to a greater size,
        // allocate a memory block of (alignment) bytes.
        let size = cmp::max(size, alignment);
        // find which level of this allocator can accommodate this amount of memory (if any)
        self.req_size_to_level(size).and_then(|req_level| {
            // We can accommodate it! Now to check if we actually have / can make a free block
            // or we're too full.
            self.get_free_block(req_level).map(|block| {
                // We got a free block!
                // get_free_block gives us the index of the block in the given level
                // so we need to find the size of each block in that level and multiply by the index
                // to get the offset of the memory that was allocated.
                let offset = block as u64 * (self.max_size() >> req_level as usize) as u64;
                // Add the base address of this buddy allocator's block and return
                self.start_address + offset
            })
        })
    }
}

// Deallocation support.
impl BuddyAllocator {
    fn merge_buddies(&mut self, level: usize, block_num: u32) {
        // toggle last bit to get buddy block
        let buddy_block = block_num ^ 1;
        // if buddy block in free list
        if let Some(buddy_idx) = self.free_lists[level]
            .iter()
            .position(|blk| *blk == buddy_block)
        {
            // remove current block (in last place)
            self.free_lists[level].pop();
            // remove buddy block
            self.free_lists[level].remove(buddy_idx);
            // add free block to free list 1 level above
            self.free_lists[level - 1].push(block_num / 2);
            // repeat the process!
            self.merge_buddies(level - 1, block_num / 2)
        }
    }

    /// Deallocate an area at the given physical address with size and aligment.
    pub fn dealloc(&mut self, addr: u64, size: usize, alignment: usize) {
        // As above, find which size was used for this allocation so that we can find the level
        // that gave us this memory block.
        let size = cmp::max(size, alignment);
        // find which level of this allocator was used for this memory request
        if let Some(req_level) = self.req_size_to_level(size) {
            // find size of each block at this level
            let level_block_size = self.max_size() >> req_level;
            // calculate which # block was just freed by using the start address and block size
            let block_num = ((addr - self.start_address) as usize / level_block_size) as u32;
            // push freed block to the free list so we can reuse it
            self.free_lists[req_level].push(block_num);
            // try merging buddy blocks now that we might have some to merge
            self.merge_buddies(req_level, block_num);
        }
    }
}
