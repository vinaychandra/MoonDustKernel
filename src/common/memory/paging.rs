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
    ) -> Result<(), &'static str>;

    /// Map a virtual address and the given size.
    fn map_with_alloc(
        &mut self,
        virt_addr: *const u8,
        size: usize,
        permissions: MapperPermissions,
    ) -> Result<(), &'static str>;

    /// Unmap a virtual address and return it's physical address and amount of data unmapped.
    fn unmap_range(&mut self, virt_addr: *const u8, size: usize) -> Result<(), &'static str>;

    /// Convert virtual address to physical address.
    fn virt_to_phys(&mut self, virt_addr: *const ()) -> Option<*const ()>;
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
