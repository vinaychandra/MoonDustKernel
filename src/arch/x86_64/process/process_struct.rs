use alloc::boxed::Box;
use x86_64::{
    registers::control::Cr3,
    structures::paging::{OffsetPageTable, PageTable, PhysFrame},
    PhysAddr, VirtAddr,
};

use crate::{
    arch::{
        globals,
        memory::{active_level_4_table_default, active_mapper_default},
    },
    common::memory::paging::IMemoryMapper,
};

#[derive(Debug)]
pub struct Thread {
    page_table: Box<PageTable>,
}

impl Thread {
    pub fn new_empty_process() -> Self {
        Self {
            page_table: Self::create_new_kernel_only_pagetable_from_current(),
        }
    }

    pub fn activate(&self) {
        let pt_vaddr = self.page_table.as_ref() as *const PageTable as *const ();
        let opt = unsafe { active_mapper_default() };
        let phys = opt
            .virt_to_phys(pt_vaddr)
            .expect("Cannot find phys mapping");
        let frame = PhysFrame::from_start_address(PhysAddr::new(phys as u64)).unwrap();
        let (_, flags) = Cr3::read();
        unsafe {
            Cr3::write(frame, flags);
        }
    }

    pub fn get_mapper(&mut self) -> impl IMemoryMapper + '_ {
        let opt = unsafe {
            OffsetPageTable::new(
                &mut self.page_table,
                VirtAddr::new(globals::MEM_MAP_OFFSET_LOCATION),
            )
        };
        return opt;
    }

    fn create_new_kernel_only_pagetable_from_current() -> Box<PageTable> {
        let mut new_table = Box::new(PageTable::new());

        let table = unsafe { active_level_4_table_default() };

        // Copy kernel level entries
        new_table[510] = table[510].clone(); // Direct mapping data
        new_table[511] = table[511].clone(); // Everything else.

        new_table
    }
}

impl Drop for Thread {
    fn drop(&mut self) {
        let active_l4 = unsafe { active_level_4_table_default() } as *mut PageTable as usize;
        let my_page_table = self.page_table.as_ref() as *const PageTable as usize;
        assert_ne!(
            active_l4, my_page_table,
            "Should not drop pagetable while active"
        );
    }
}
