use crate::arch::globals;
use alloc::boxed::Box;
use x86_64::{structures::paging::PageTable, VirtAddr};

/// Uses current page table to create a new page table. Needs a mapping at
/// 0xFFFF_FF00_0000_0000.
pub fn create_new_kernel_only_table_from_current() -> Box<PageTable> {
    let mut new_table = Box::new(PageTable::new());

    let table = unsafe { super::active_level_4_table(VirtAddr::new(globals::MEM_MAP_LOCATION)) };

    // Copy kernel level entries
    new_table[510] = table[510].clone(); // Direct mapping data
    new_table[511] = table[511].clone(); // Everything else.

    new_table
}
