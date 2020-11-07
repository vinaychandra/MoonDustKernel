use super::globals;
use crate::common::memory;
use memory::allocator::physical_memory_allocator;
use physical_memory_allocator::IPhysicalMemoryAllocator;
use x86_64::{
    structures::paging::{
        Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame, Size1GiB,
    },
    PhysAddr, VirtAddr,
};

pub mod paging;
pub mod stack;
pub mod tables;

/// Initialize a new OffsetPageTable for bootstrap processor.
/// This moves the existing mem map to a arch dependant location as well.
pub fn init_bsp(allocator: &mut dyn IPhysicalMemoryAllocator) -> OffsetPageTable<'static> {
    unsafe {
        let original_offset = VirtAddr::new(0x0);
        let bsp_page_table = active_level_4_table(original_offset);

        let mut mapper = OffsetPageTable::new(bsp_page_table, original_offset);
        let mut allocator = paging::get_frame_allocator_zeroed(allocator, 0);
        // 512 GB memory map.
        for i in 0..512 {
            let page = Page::from_start_address(VirtAddr::new(
                (i * 1024 * 1024 * 1024) + globals::MEM_MAP_LOCATION,
            ))
            .unwrap();

            let frame =
                PhysFrame::<Size1GiB>::from_start_address(PhysAddr::new(i * 1024 * 1024 * 1024))
                    .unwrap();
            mapper
                .map_to(
                    page,
                    frame,
                    PageTableFlags::PRESENT | PageTableFlags::WRITABLE,
                    &mut allocator,
                )
                .unwrap()
                .flush();
        }

        let final_offset = VirtAddr::new(globals::MEM_MAP_LOCATION);
        let bsp_page_table = active_level_4_table(final_offset);
        let org = OffsetPageTable::new(bsp_page_table, final_offset);

        org
    }
}

/// Initialize CPU local store for kernel.
pub fn initialize_tls() {
    let total_size;
    let tls_ptr = unsafe {
        let tdata_size =
            &__tdata_end as *const usize as usize - &__tdata_start as *const usize as usize;
        total_size = &__tbss_end as *const usize as usize - &__tdata_start as *const usize as usize;
        memory::cpu_local::load_tls_data(
            &__tdata_start as *const usize as *const u8,
            tdata_size,
            total_size + 8, // Add 8 bytes to store TCB pointer.
        )
    };
    info!(target: "initialize_tls", "TLS data loaded. Setting fs");
    let fs_ptr = ((tls_ptr as *const u8 as u64) + (total_size as u64)) as *mut u64;
    x86_64::registers::model_specific::FsBase::write(VirtAddr::from_ptr(fs_ptr));
    unsafe {
        *fs_ptr = fs_ptr as u64;
    }
    info!(target: "initialize_tls", "TLS Pointer is set to {:x?}. Size is {:?} bytes", fs_ptr, total_size);
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
