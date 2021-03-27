//! LAPIC and IOAPIC support for x86_64 architecture.

use core::{cell::Cell, ptr::null_mut, usize};

use acpi::platform::Apic;
use apic::{io_apic::IoApicBase, registers::TimerDivideConfigurationValue, ApicBase};
use x86_64::{registers::model_specific::Msr, PhysAddr};

use crate::arch::globals;

/// Local APIC data.
#[thread_local]
pub static mut LAPIC: ApicBase = unsafe { ApicBase::new(null_mut()) };

/// Processor ID of the current processor.
#[thread_local]
pub static PROCESSOR_ID: Cell<usize> = Cell::new(0);

/// Initialize the current LAPIC. This is run on each Processor to turn them on and
/// set the interrupts correctly.
pub fn initialize_lapic() {
    let lapic_base = read_lapic_base();
    let lapic_mem = lapic_base.as_u64() + globals::MEM_MAP_OFFSET_LOCATION;
    let lapic_mem = lapic_mem as *mut ();

    let mut lapic_instance = unsafe { ApicBase::new(lapic_mem) };

    //TODO: Make sure the TPR (Task Priority Register) is set (so it won't block/postpone lower priority IRQs)
    // Enable local apic
    {
        let spurios_vector = lapic_instance.spurious_interrupt_vector();
        let mut val = spurios_vector.read();
        val.enable_apic_software(true);
    }

    // Enable timer
    {
        lapic_instance
            .timer_divide_configuration()
            .update(|t| t.set(TimerDivideConfigurationValue::Divide64));
        lapic_instance.timer_local_vector_table_entry().update(|t| {
            t.set_vector(super::InterruptIndex::Timer.as_u8());
            t.set_timer_mode(true);
            t.set_mask(false);
        });
        lapic_instance.timer_initial_count().update(|t| {
            t.set(123456);
        });
    }

    PROCESSOR_ID.replace(lapic_instance.id().read().id() as usize);

    unsafe {
        LAPIC = lapic_instance;
    }
}

/// Startup the IOApic. This is usually run on only one of the processor because IOApic is
/// shared among multiple cores.
pub fn initialize_ioapic(apic: Apic) {
    let count = apic.io_apics.len();
    info!(target: "apic", "IOApics found: {}", count);

    let first_apic = &apic.io_apics[0];
    let address = first_apic.address as u64;
    let address = (address + globals::MEM_MAP_OFFSET_LOCATION) as *mut u8;
    let mut ioapic = unsafe { IoApicBase::new(address) };

    let lapic = unsafe { &mut LAPIC };
    let id = lapic.id().read().id();

    ioapic.update_redirection_table_entry(1, |entry| {
        entry.set_destination(id);
        entry.set_masked(false);
        entry.set_vector(super::InterruptIndex::Keyboard.as_u8());
    });
}

/// Get the LApic Base address.
/// This function reads from `IA32_APIC_BASE`.
fn read_lapic_base() -> PhysAddr {
    const IA32_APIC_BASE: u32 = 0x1B;
    unsafe { PhysAddr::new(Msr::new(IA32_APIC_BASE).read() & 0xFFFFFF000 as u64) }
}
