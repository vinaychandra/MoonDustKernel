use core::ptr::NonNull;

use acpi::{AcpiHandler, PhysicalMapping};

/// Memory handler used by ACPI to access mapping
/// regions of physical memory.
/// # Notes
/// The ACPI crate requires this handler so that it can
/// map physical memory onto the current address space. Because,
/// we already map everything into the address space, we just
/// return the mapped values.
#[derive(Debug, Clone)]
pub struct MemoryHandler {
    /// The location where all of physical memory is mapped to.
    phys_mem_offset: usize,
}

impl MemoryHandler {
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
            handler: self.clone(),
        }
    }

    fn unmap_physical_region<T>(&self, _region: &PhysicalMapping<Self, T>) {
        // We don't need to unmap any physical region.
        debug!(target:"acpi", "ACPI requested an unmap.");
    }
}
