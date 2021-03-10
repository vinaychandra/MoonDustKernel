use crate::common::memory::{
    allocator::physical_memory_allocator::IPhysicalMemoryAllocator,
    paging::{IMemoryMapper, IPageTable, MapperPermissions},
};
use core::alloc::Layout;
use x86_64::{
    registers::control::Cr3,
    structures::paging::{
        page::PageRange, FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags,
        PhysFrame, Size4KiB, Translate,
    },
    PhysAddr, VirtAddr,
};

impl<'a> IMemoryMapper for OffsetPageTable<'a> {
    fn map(
        &mut self,
        phys_addr: *const u8,
        virt_addr: *const u8,
        size: usize,
        permissions: MapperPermissions,
        allocator: &dyn IPhysicalMemoryAllocator,
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

    fn map_with_alloc(
        &mut self,
        virt_addr: *const u8,
        size: usize,
        permissions: MapperPermissions,
        allocator: &dyn IPhysicalMemoryAllocator,
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
    allocator: &dyn IPhysicalMemoryAllocator,
) -> impl FrameAllocator<Size4KiB> + '_ {
    PhysicalMemoryAllocatorWrapper {
        allocator,
        zeroed: false,
        mem_map_offset: None,
    }
}

pub fn get_frame_allocator_zeroed(
    allocator: &dyn IPhysicalMemoryAllocator,
    mem_map_offset: usize,
) -> impl FrameAllocator<Size4KiB> + '_ {
    PhysicalMemoryAllocatorWrapper {
        allocator,
        zeroed: true,
        mem_map_offset: Some(mem_map_offset),
    }
}

pub fn activate_page_table(phys_addr: PhysAddr) {
    let frame =
        PhysFrame::from_start_address(phys_addr).expect("Physical address is not frame aligned.");
    let (_, flags) = Cr3::read();
    unsafe {
        Cr3::write(frame, flags);
    }
}

struct PhysicalMemoryAllocatorWrapper<'a> {
    pub allocator: &'a dyn IPhysicalMemoryAllocator,
    pub zeroed: bool,
    pub mem_map_offset: Option<usize>,
}

unsafe impl<'a> FrameAllocator<Size4KiB> for PhysicalMemoryAllocatorWrapper<'a> {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let layout = Layout::from_size_align(4096, 4096);
        let addr = self.allocator.allocate_physical_memory(layout.unwrap());

        if self.zeroed {
            // Clear the mem
            for i in 0..4096 {
                unsafe { *(addr.add(i).add(self.mem_map_offset.unwrap()) as *mut u8) = 0 };
            }
        }

        Some(PhysFrame::from_start_address(PhysAddr::new(addr as u64)).unwrap())
    }
}

impl IPageTable for PageTable {
    fn get_addr(&self) -> *const dyn IPageTable {
        self
    }
}
