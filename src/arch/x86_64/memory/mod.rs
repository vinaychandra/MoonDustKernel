pub mod cpu_local;
pub mod frame_allocator;
pub mod paging;

use x86_64::{
    structures::paging::{OffsetPageTable, PageTable},
    VirtAddr,
};

use super::globals;
use crate::common::memory::paging::IMemoryMapper;

/// Returns a mutable reference to the active level 4 table.
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
pub unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    unsafe { &mut *page_table_ptr } // unsafe
}

/// Returns a mutable reference to the active level 4 table.
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
pub unsafe fn active_level_4_table_default() -> &'static mut PageTable {
    unsafe { active_level_4_table(VirtAddr::new(globals::MEM_MAP_OFFSET_LOCATION)) }
}

/// Get the offsetpagetable for the default setup.
pub unsafe fn active_mapper_default() -> impl IMemoryMapper {
    let pt = unsafe { active_level_4_table_default() };
    unsafe { OffsetPageTable::new(pt, VirtAddr::new(globals::MEM_MAP_OFFSET_LOCATION)) }
}
