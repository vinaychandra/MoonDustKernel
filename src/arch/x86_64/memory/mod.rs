use super::globals;
use crate::common::memory;
use x86_64::{
    structures::paging::{OffsetPageTable, PageTable},
    VirtAddr,
};

pub mod paging;
pub mod stack;
pub mod tables;

/// Initialize a new OffsetPageTable for bootstrap processor.
/// This moves the existing mem map to a arch dependant location as well.
pub fn init_bsp() -> OffsetPageTable<'static> {
    unsafe {
        let original_offset = VirtAddr::new(0x0);
        let bsp_page_table = active_level_4_table(original_offset);
        bsp_page_table[510] = bsp_page_table[0].clone(); // Copy the original identity mapping to 0xFFFF_FF00_0000_0000

        let final_offset = VirtAddr::new(globals::MEM_MAP_LOCATION);
        let bsp_page_table = active_level_4_table(final_offset);
        let org = OffsetPageTable::new(bsp_page_table, final_offset);

        org
    }
}

/// Initialize CPU local store for kernel.
pub fn initialize_tls() {
    let tls_ptr = unsafe {
        let tdata_size =
            &__tdata_end as *const usize as usize - &__tdata_start as *const usize as usize;
        let total_size =
            &__tbss_end as *const usize as usize - &__tdata_start as *const usize as usize;
        memory::cpu_local::load_tls_data(
            &__tdata_start as *const usize as *const u8,
            tdata_size,
            total_size + 8, // Add 8 bytes to store TCB pointer.
        )
    };
    info!(target: "initialize_tls", "TLS data loaded. Setting fs");
    let fs_ptr = unsafe {
        ((tls_ptr as *const u8 as u64)
            + (&__tbss_end as *const usize as u64 - &__tdata_start as *const usize as u64))
            as *mut u64
    };
    x86_64::registers::model_specific::FsBase::write(VirtAddr::from_ptr(fs_ptr));
    unsafe {
        *fs_ptr = fs_ptr as u64;
    }
    info!(target: "initialize_tls", "TLS Pointer is set to {:x?}", fs_ptr);
}

extern "C" {
    static mut __tdata_start: usize;
    static mut __tdata_end: usize;
    static mut __tbss_start: usize;
    static mut __tbss_end: usize;
}

/// Returns a mutable reference to the active level 4 table.
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr // unsafe
}
