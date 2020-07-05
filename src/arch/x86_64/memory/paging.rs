use crate::common::memory::{
    allocator::physical_memory_allocator::IPhysicalMemoryAllocator,
    paging::{MapperPermissions, MemoryMapper},
};
use core::alloc::Layout;
use x86_64::{
    structures::paging::{
        page::PageRange, FrameAllocator, Mapper, MapperAllSizes, OffsetPageTable, Page,
        PageTableFlags, PhysFrame, Size4KiB,
    },
    PhysAddr, VirtAddr,
};

impl<'a> MemoryMapper for OffsetPageTable<'a> {
    fn map<AllocatorType: IPhysicalMemoryAllocator>(
        &mut self,
        phys_addr: *const u8,
        virt_addr: *const u8,
        size: usize,
        permissions: MapperPermissions,
        allocator: &AllocatorType,
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

        let mut frame_allocator = get_frame_allocator(allocator);
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

    fn map_with_alloc<AllocatorType: IPhysicalMemoryAllocator>(
        &mut self,
        virt_addr: *const u8,
        size: usize,
        permissions: MapperPermissions,
        allocator: &AllocatorType,
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
                .expect("start addr is no aligned");
            let end_page =
                Page::<Size4KiB>::from_start_address(VirtAddr::new(virt_addr as u64 + size as u64))
                    .unwrap();
            Page::range(start_page, end_page)
        };

        let mut frame_allocator = get_frame_allocator(allocator);
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

    fn virt_to_phys(&self, virt_addr: *const u8) -> Option<*const u8> {
        let virt_addr = VirtAddr::from_ptr(virt_addr);
        let phys_addr = self.translate_addr(virt_addr)?;
        Some(phys_addr.as_u64() as *const u8)
    }

    fn get_page_table(&self) -> *const u8 {
        todo!()
    }
}

pub fn get_frame_allocator(
    allocator: &impl IPhysicalMemoryAllocator,
) -> impl FrameAllocator<Size4KiB> + '_ {
    PhysicalMemoryAllocatorWrapper { allocator }
}

struct PhysicalMemoryAllocatorWrapper<'a> {
    pub allocator: &'a dyn IPhysicalMemoryAllocator,
}

unsafe impl<'a> FrameAllocator<Size4KiB> for PhysicalMemoryAllocatorWrapper<'a> {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let layout = Layout::from_size_align(4096, 4096);
        let addr = self.allocator.allocate_physical_memory(layout.unwrap());
        Some(PhysFrame::from_start_address(PhysAddr::new(addr as u64)).unwrap())
    }
}
