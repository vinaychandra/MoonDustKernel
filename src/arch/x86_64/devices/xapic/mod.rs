mod ioapic;
mod lapic;

use lapic::LApic;
use x86_64::VirtAddr;

#[thread_local]
pub static mut LAPIC: Option<LApic> = None;

pub fn initialize_apic(phys_mem_offset: VirtAddr, ioapic_addr: u64) {
    info!(
        "Initialize APIC: Offset {:x} IOApic Addr: {:x} LapicAddr: {:x}",
        phys_mem_offset.as_u64(),
        ioapic_addr,
        unsafe { LApic::read_base().as_u64() }
    );
    let lapic_instance = unsafe {
        LApic::new(VirtAddr::new(
            LApic::read_base().as_u64() + phys_mem_offset.as_u64(),
        ))
    };

    let ioapic_instance =
        ioapic::IOApic::new(VirtAddr::new(ioapic_addr + phys_mem_offset.as_u64()));

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
