use super::allocator::physical_memory_allocator::IPhysicalMemoryAllocator;

/// A memory mapper used to map virtual memory.
pub trait MemoryMapper {
    /// Map a virtual address to physical address and the given size.
    fn map<AllocatorType: IPhysicalMemoryAllocator>(
        &mut self,
        phys_addr: *const u8,
        virt_addr: *const u8,
        size: usize,
        permissions: MapperPermissions,
        allocator: &AllocatorType,
    ) -> Result<(), &'static str>;

    /// Map a virtual address and the given size.
    fn map_with_alloc<AllocatorType: IPhysicalMemoryAllocator>(
        &mut self,
        virt_addr: *const u8,
        size: usize,
        permissions: MapperPermissions,
        allocator: &AllocatorType,
    ) -> Result<(), &'static str>;

    /// Unmap a virtual address and return it's physical address and amount of data unmapped.
    fn unmap_range(&mut self, virt_addr: *const u8, size: usize) -> Result<(), &'static str>;

    /// Convert virtual address to physical address.
    fn virt_to_phys(&self, virt_addr: *const u8) -> Option<*const u8>;

    /// Get the root page table that is denoted by this mapper.
    fn get_page_table(&self) -> *const u8;
}

bitflags! {
    pub struct MapperPermissions : u8 {
        // const READ      = 0b0000_0001;
        const WRITE     = 0b0000_0010;
        const EXECUTE   = 0b0000_0100;
        const RING_3    = 0b0000_1000;
    }
}
