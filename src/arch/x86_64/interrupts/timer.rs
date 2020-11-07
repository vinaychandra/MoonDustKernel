use x86_64::structures::idt::InterruptStackFrame;

use super::InterruptIndex;

pub extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
    unsafe {
        info!("TOCK");

        let lapic = &super::super::devices::xapic::LAPIC;
        if let Some(lapic_val) = &*lapic {
            lapic_val.send_eoi(InterruptIndex::Timer as u8);
        }
    }
}

pub extern "x86-interrupt" fn hpet_timer_handler(_stack_frame: &mut InterruptStackFrame) {
    info!("TICK");

    unsafe {
        let lapic = &super::super::devices::xapic::LAPIC;
        if let Some(lapic_val) = &*lapic {
            lapic_val.send_eoi(InterruptIndex::HpetTimer as u8);
        }
    }
}
