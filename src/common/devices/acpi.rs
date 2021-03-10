use core::ptr::NonNull;

use acpi::{AcpiHandler, PhysicalMapping};

/// Memory handler used by ACPI to access mapping
/// regions of physical memory.
#[derive(Debug, Copy, Clone)]
pub struct MemoryHandler {
    /// The location where all of physical memory is mapped to.
    phys_mem_offset: usize,
}

impl MemoryHandler {
    /// Create a memory handler for ACPI that works when memory is mapped
    /// at the [`phys_mem_offset`]
    pub const fn new(phys_mem_offset: usize) -> MemoryHandler {
        MemoryHandler { phys_mem_offset }
    }
}

impl AcpiHandler for MemoryHandler {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> PhysicalMapping<Self, T> {
        let target_virtual_address: usize = physical_address + self.phys_mem_offset;
        debug!(target:"acpi", "Mappping region at address {:x} with size {} bytes", target_virtual_address, size);

        PhysicalMapping {
            physical_start: physical_address,
            virtual_start: NonNull::new(target_virtual_address as *mut T).unwrap(),
            region_length: size,
            mapped_length: size,
            handler: *self,
        }
    }

    fn unmap_physical_region<T>(&self, _region: &PhysicalMapping<Self, T>) {
        // We don't need to unmap any physical region.
        debug!(target:"acpi", "ACPI requested an unmap.");
    }
}
