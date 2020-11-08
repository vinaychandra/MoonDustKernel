use super::allocator::physical_memory_allocator::IPhysicalMemoryAllocator;

/// A memory mapper used to map virtual memory.
/// This usually wraps around page table logic to map addresses.
pub trait IMemoryMapper {
    /// Map a virtual address to physical address and the given size.
    fn map(
        &mut self,
        phys_addr: *const u8,
        virt_addr: *const u8,
        size: usize,
        permissions: MapperPermissions,
        allocator: &dyn IPhysicalMemoryAllocator,
    ) -> Result<(), &'static str>;

    /// Map a virtual address and the given size.
    fn map_with_alloc(
        &mut self,
        virt_addr: *const u8,
        size: usize,
        permissions: MapperPermissions,
        allocator: &dyn IPhysicalMemoryAllocator,
    ) -> Result<(), &'static str>;

    /// Unmap a virtual address and return it's physical address and amount of data unmapped.
    fn unmap_range(&mut self, virt_addr: *const u8, size: usize) -> Result<(), &'static str>;

    /// Convert virtual address to physical address.
    fn virt_to_phys(&self, virt_addr: *const u8) -> Option<*const u8>;

    /// Get the root page table that is denoted by this mapper.
    fn get_page_table(&self) -> *const u8;
}

pub trait IPageTable {
    /// Get the address of the current page table.
    fn get_addr(&self) -> *const dyn IPageTable;
}

bitflags! {
    /// Permissions for the current page.
    pub struct MapperPermissions : u8 {
        const READ      = 0b0000_0000;
        const WRITE     = 0b0000_0010;
        const EXECUTE   = 0b0000_0100;
        const RING_3    = 0b0000_1000;
    }
}
