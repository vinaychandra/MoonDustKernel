mod memory_handler;

use acpi::search_for_rsdp_bios;
use acpi::Acpi;
use memory_handler::MemoryHandler;
use x86_64::VirtAddr;

/// Load the ACPI tables.
///
/// WARN: This only works when using BIOS to boot.
pub fn load_acpi(phys_mem_offset: VirtAddr) -> Acpi {
    let mut handler = MemoryHandler::new(phys_mem_offset);
    let rsdp_from_bios = unsafe { search_for_rsdp_bios(&mut handler).unwrap() };
    rsdp_from_bios
}
