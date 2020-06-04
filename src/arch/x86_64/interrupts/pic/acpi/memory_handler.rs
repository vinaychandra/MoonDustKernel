use acpi::handler::{AcpiHandler, PhysicalMapping};
use core::ptr::NonNull;
use x86_64::VirtAddr;

/// Memory handler used by ACPI to access mapping
/// regions of physical memory.
pub struct MemoryHandler {
    /// The location where all of physical memory is mapped to.
    phys_mem_offset: VirtAddr,
}

impl MemoryHandler {
    pub const fn new(phys_mem_offset: VirtAddr) -> MemoryHandler {
        MemoryHandler { phys_mem_offset }
    }
}

impl AcpiHandler for MemoryHandler {
    unsafe fn map_physical_region<T>(
        &mut self,
        physical_address: usize,
        size: usize,
    ) -> PhysicalMapping<T> {
        // Because all of memory is already mapped, we just use that.
        let target_virtual_address: usize =
            physical_address + self.phys_mem_offset.as_u64() as usize;

        PhysicalMapping {
            physical_start: physical_address,
            virtual_start: NonNull::new(target_virtual_address as *mut T).unwrap(),
            region_length: size,
            mapped_length: size,
        }
    }

    fn unmap_physical_region<T>(&mut self, _region: PhysicalMapping<T>) {
        // We don't need to unmap any physical region.
    }
}
