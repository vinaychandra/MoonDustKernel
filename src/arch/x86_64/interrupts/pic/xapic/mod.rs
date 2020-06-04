//! APIC ("Advanced Programmable Interrupt Controller") is the updated Intel standard for
//! the older PIC. It is used in multiprocessor systems and is an integral part of all
//! recent Intel (and compatible) processors. The APIC is used for sophisticated interrupt
//! redirection, and for sending interrupts between processors.

mod ioapic;
mod lapic;

use lapic::LApic;
use x86_64::VirtAddr;

/// Local APIC instance. This is a hardware available per-CPU.
pub static mut LAPIC: Option<LApic> = None;

/// Initialize the APIC
/// ## Arguments
/// * `phys_mem_offset` - The offset at which all of physical memory is mapped to.
/// * `ioapic_address` - The physical address at which IOAPIC is mapped to. This value
/// has to be detected from ACPI tables. For Qemu, this is 0xfec0000
pub fn initialize_apic(phys_mem_offset: VirtAddr, ioapic_address: u32) {
    let lapic_instance = unsafe {
        LApic::new(VirtAddr::new(
            LApic::read_base().as_u64() + phys_mem_offset.as_u64(),
        ))
    };

    let ioapic_instance = ioapic::IOApic::new(VirtAddr::new(
        ioapic_address as u64 + phys_mem_offset.as_u64(),
    ));

    // Enable local APIC
    lapic_instance.enable();

    // Get the APIC ID
    let apic_id = lapic_instance.read(lapic::APIC_ID_REGISTER);

    // Reroute IOApic's IRQs
    ioapic_instance.set_irq(1, apic_id, 33);
    // ioapic_instance.set_irq(2, apic_id, 32);
    ioapic_instance.set_irq(8, apic_id, 36);

    // Store LAPIC
    unsafe {
        LAPIC.replace(lapic_instance);
    }
}
