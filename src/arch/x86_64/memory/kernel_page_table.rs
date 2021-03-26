use core::{
    alloc::{Layout, LayoutError},
    ops::Bound,
    ptr::NonNull,
};

use alloc::boxed::Box;
use moondust_utils::interval_tree::{Interval, IntervalTree};
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
    common::{
        align_up,
        memory::paging::{IMemoryMapper, MapperPermissions},
    },
};

#[derive(Debug)]
pub struct KernelPageTable {
    page_table: Box<PageTable>,
    vmem_allocated: usize,
    mem_areas: IntervalTree<u64>,

    heap_allocated: usize,
}

impl KernelPageTable {
    pub fn new(page_table: Box<PageTable>) -> Self {
        Self {
            page_table,
            vmem_allocated: 0,
            mem_areas: IntervalTree::new(),
            heap_allocated: 0,
        }
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

    pub fn get_user_heap_size(&self) -> usize {
        self.heap_allocated
    }

    pub fn map_user_heap(&mut self, size_to_increase: usize) -> Result<(), &'static str> {
        let size_to_increase = align_up(size_to_increase, globals::PAGE_SIZE);
        let final_size = self.heap_allocated + size_to_increase;

        // Max heap size reached.
        if final_size >= globals::USER_HEAP_END - globals::USER_HEAP_START {
            return Err("Max heap size reached");
        }

        let address_to_allocate_from = globals::USER_HEAP_START + self.heap_allocated;
        let val = self.map_with_alloc(
            address_to_allocate_from as _,
            size_to_increase,
            MapperPermissions::WRITE | MapperPermissions::RING_3 | MapperPermissions::READ,
        );
        if val.is_err() {
            // TODO: kill this process?
            return Err("Out of memory while allocating heap");
        }
        self.heap_allocated = final_size;
        Ok(())
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
        if !crate::arch::is_kernel_mode(virt_addr as u64) {
            self.vmem_allocated += size;
            self.mem_areas = self.mem_areas.insert(Interval::new(
                Bound::Included(virt_addr as u64),
                Bound::Excluded(virt_addr as u64 + size as u64),
            ));
        }

        self.get_mapper()
            .map(phys_addr, virt_addr, size, permissions)
    }

    fn map_with_alloc(
        &mut self,
        virt_addr: *const u8,
        size: usize,
        permissions: MapperPermissions,
    ) -> Result<(), &'static str> {
        if !crate::arch::is_kernel_mode(virt_addr as u64) {
            self.vmem_allocated += size;
            self.mem_areas = self.mem_areas.insert(Interval::new(
                Bound::Included(virt_addr as u64),
                Bound::Excluded(virt_addr as u64 + size as u64),
            ));
        }

        self.get_mapper()
            .map_with_alloc(virt_addr, size, permissions)
    }

    fn unmap_range(&mut self, virt_addr: *const u8, size: usize) -> Result<(), &'static str> {
        if !crate::arch::is_kernel_mode(virt_addr as u64) {
            self.vmem_allocated -= size;
        }

        self.get_mapper().unmap_range(virt_addr, size)
    }

    fn virt_to_phys(&mut self, virt_addr: *const ()) -> Option<*const ()> {
        self.get_mapper().virt_to_phys(virt_addr)
    }
}

impl Drop for KernelPageTable {
    fn drop(&mut self) {
        info!(
            "Dropping PageTable... (Currently allocated {} bytes)",
            self.vmem_allocated
        );

        let areas = self.mem_areas.clone();
        for interval in areas.iter() {
            let start_val = match interval.low() {
                Bound::Included(a) => *a,
                Bound::Excluded(a) => *a + 1,
                Bound::Unbounded => panic!("Cannot have unbounded interval!"),
            };

            let end_val = match interval.high() {
                Bound::Included(a) => *a,
                Bound::Excluded(a) => *a - 1,
                Bound::Unbounded => panic!("Cannot have unbounded interval!"),
            };

            let start = Page::<Size4KiB>::containing_address(VirtAddr::new(start_val));
            let end = Page::<Size4KiB>::containing_address(VirtAddr::new(end_val));
            let size = end.start_address() - start.start_address() + 4096;

            self.get_mapper()
                .unmap_range(start.start_address().as_ptr(), size as usize)
                .unwrap();
        }
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
                .map_err(|_| "start addr is no aligned")?;
            let end_page =
                Page::<Size4KiB>::from_start_address(VirtAddr::new(virt_addr as u64 + size as u64))
                    .map_err(|_| "start addr is no aligned")?;
            Page::range(start_page, end_page)
        };
        let locked_allocator = &crate::arch::PHYSICAL_MEMORY_ALLOCATOR;
        let mut allocator = locked_allocator.lock();
        for page in page_range {
            self.unmap(page).map_or((), |frame| {
                frame.1.flush();
                let phys_addr = frame.0.start_address();
                let virt_addr = phys_addr.as_u64() + globals::MEM_MAP_OFFSET_LOCATION;
                const LAYOUT: Result<Layout, LayoutError> = Layout::from_size_align(4096, 4096);
                allocator.dealloc(NonNull::new(virt_addr as *mut u8).unwrap(), LAYOUT.unwrap());
            });
        }

        Ok(())
    }

    fn virt_to_phys(&mut self, virt_addr: *const ()) -> Option<*const ()> {
        let virt_addr = VirtAddr::from_ptr(virt_addr);
        let phys_addr = self.translate_addr(virt_addr)?;
        Some(phys_addr.as_u64() as *const ())
    }
}
