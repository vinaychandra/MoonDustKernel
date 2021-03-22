use alloc::boxed::Box;
use x86_64::{
    registers::control::Cr3,
    structures::paging::{
        page::PageRange, FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags,
        PhysFrame, Size4KiB, Translate,
    },
    PhysAddr, VirtAddr,
};

use crate::{
    arch::globals,
    common::memory::paging::{IMemoryMapper, MapperPermissions},
};

#[derive(Debug)]
pub struct KernelPageTable {
    page_table: Box<PageTable>,
}

impl KernelPageTable {
    pub fn new(page_table: Box<PageTable>) -> Self {
        Self { page_table }
    }

    fn get_mapper(&mut self) -> impl IMemoryMapper + '_ {
        let offset = VirtAddr::new(globals::MEM_MAP_OFFSET_LOCATION);
        unsafe { OffsetPageTable::new(&mut self.page_table, offset) }
    }

    pub fn activate(&mut self) {
        let pt_vaddr = self.page_table.as_ref() as *const PageTable as *const ();
        let mut opt = self.get_mapper();
        let phys = opt
            .virt_to_phys(pt_vaddr)
            .expect("Cannot find phys mapping");
        let frame = PhysFrame::from_start_address(PhysAddr::new(phys as u64)).unwrap();
        let (_, flags) = Cr3::read();
        unsafe {
            Cr3::write(frame, flags);
        }
    }
}

impl<'a> IMemoryMapper for KernelPageTable {
    fn map(
        &mut self,
        phys_addr: *const u8,
        virt_addr: *const u8,
        size: usize,
        permissions: MapperPermissions,
    ) -> Result<(), &'static str> {
        self.get_mapper()
            .map(phys_addr, virt_addr, size, permissions)
    }

    fn map_with_alloc(
        &mut self,
        virt_addr: *const u8,
        size: usize,
        permissions: MapperPermissions,
    ) -> Result<(), &'static str> {
        self.get_mapper()
            .map_with_alloc(virt_addr, size, permissions)
    }

    fn unmap_range(&mut self, virt_addr: *const u8, size: usize) -> Result<(), &'static str> {
        self.get_mapper().unmap_range(virt_addr, size)
    }

    fn virt_to_phys(&mut self, virt_addr: *const ()) -> Option<*const ()> {
        self.get_mapper().virt_to_phys(virt_addr)
    }
}

impl Drop for KernelPageTable {
    fn drop(&mut self) {
        info!("dropping kpt");
    }
}

impl<'a> IMemoryMapper for OffsetPageTable<'a> {
    fn map(
        &mut self,
        phys_addr: *const u8,
        virt_addr: *const u8,
        size: usize,
        permissions: MapperPermissions,
    ) -> Result<(), &'static str> {
        debug_assert!(size % 4096 == 0, "Size must be page aligned");

        let mut perms: PageTableFlags = PageTableFlags::PRESENT;
        if !permissions.contains(MapperPermissions::EXECUTE) {
            perms = perms | PageTableFlags::NO_EXECUTE;
        }

        if permissions.contains(MapperPermissions::WRITE) {
            perms = perms | PageTableFlags::WRITABLE;
        }

        if permissions.contains(MapperPermissions::RING_3) {
            perms = perms | PageTableFlags::USER_ACCESSIBLE;
        }

        let page_range: PageRange = {
            let start_page = Page::<Size4KiB>::from_start_address(VirtAddr::from_ptr(virt_addr))
                .expect("start addr is no aligned");
            let end_page =
                Page::<Size4KiB>::from_start_address(VirtAddr::new(virt_addr as u64 + size as u64))
                    .unwrap();
            Page::range(start_page, end_page)
        };

        let mut frame_allocator = super::frame_allocator::get_frame_allocator();
        for (index, page) in page_range.into_iter().enumerate() {
            let frame = PhysFrame::<Size4KiB>::from_start_address(PhysAddr::new(
                phys_addr as u64 + (index as u64 * 4096),
            ))
            .expect("Physical Frame creation failed.");

            unsafe {
                self.map_to(page, frame, perms, &mut frame_allocator)
                    .expect("Mapping failed.")
                    .flush();
            }
        }

        Ok(())
    }

    fn map_with_alloc(
        &mut self,
        virt_addr: *const u8,
        size: usize,
        permissions: MapperPermissions,
    ) -> Result<(), &'static str> {
        debug_assert!(size % 4096 == 0, "Size must be page aligned");

        let mut perms: PageTableFlags = PageTableFlags::PRESENT;
        if !permissions.contains(MapperPermissions::EXECUTE) {
            perms |= PageTableFlags::NO_EXECUTE;
        }

        if permissions.contains(MapperPermissions::WRITE) {
            perms |= PageTableFlags::WRITABLE;
        }

        if permissions.contains(MapperPermissions::RING_3) {
            perms |= PageTableFlags::USER_ACCESSIBLE;
        }

        let page_range: PageRange = {
            let start_page = Page::<Size4KiB>::from_start_address(VirtAddr::from_ptr(virt_addr))
                .expect("start addr is not aligned");
            let end_page =
                Page::<Size4KiB>::from_start_address(VirtAddr::new(virt_addr as u64 + size as u64))
                    .unwrap();
            Page::range(start_page, end_page)
        };
        let _num_pages = page_range.count();

        let mut frame_allocator = super::frame_allocator::get_frame_allocator_zeroed();
        for page in page_range {
            let _s: *const u8 = page.start_address().as_ptr();
            let frame = frame_allocator
                .allocate_frame()
                .expect("Cannot allocate frame");
            unsafe {
                self.map_to(page, frame, perms, &mut frame_allocator)
                    .expect("Mapping failed.")
                    .flush();
            }
        }

        Ok(())
    }

    fn unmap_range(&mut self, virt_addr: *const u8, size: usize) -> Result<(), &'static str> {
        let page_range: PageRange = {
            let start_page = Page::<Size4KiB>::from_start_address(VirtAddr::from_ptr(virt_addr))
                .expect("start addr is no aligned");
            let end_page =
                Page::<Size4KiB>::from_start_address(VirtAddr::new(virt_addr as u64 + size as u64))
                    .unwrap();
            Page::range(start_page, end_page)
        };
        for page in page_range {
            self.unmap(page).map_or((), |v| v.1.flush());
        }

        Ok(())
    }

    fn virt_to_phys(&mut self, virt_addr: *const ()) -> Option<*const ()> {
        let virt_addr = VirtAddr::from_ptr(virt_addr);
        let phys_addr = self.translate_addr(virt_addr)?;
        Some(phys_addr.as_u64() as *const ())
    }
}
