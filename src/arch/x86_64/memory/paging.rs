use x86_64::{
    structures::paging::{
        page::PageRange, FrameAllocator, Mapper, OffsetPageTable, Page, PageTableFlags, PhysFrame,
        Size4KiB, Translate,
    },
    PhysAddr, VirtAddr,
};

use crate::common::memory::paging::{IMemoryMapper, MapperPermissions};

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

        let mut frame_allocator = super::frame_allocator::get_frame_allocator();
        for page in page_range {
            let _s: *const u8 = page.start_address().as_ptr();
            let frame = frame_allocator
                .allocate_frame()
                .expect("Cannot allocate frame");
            debug!(target:"paging", "alloc {:x} to {:?}", _s as usize, frame);
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

    fn virt_to_phys(&self, virt_addr: *const u8) -> Option<*const u8> {
        let virt_addr = VirtAddr::from_ptr(virt_addr);
        let phys_addr = self.translate_addr(virt_addr)?;
        Some(phys_addr.as_u64() as *const u8)
    }

    fn get_page_table(&self) -> *const u8 {
        todo!()
    }
}
