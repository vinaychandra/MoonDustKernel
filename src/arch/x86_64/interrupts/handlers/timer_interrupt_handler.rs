use super::super::InterruptIndex;
use crate::*;
use x86_64::structures::idt::InterruptStackFrame;

pub extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
    unsafe {
        kernel_info!("TOCK");

        let lapic = &super::super::pic::xapic::LAPIC;
        if let Some(lapic_val) = &*lapic {
            lapic_val.send_eoi(InterruptIndex::Timer.as_u8());
        }

        x86_64::instructions::interrupts::enable();
    }
}

pub extern "x86-interrupt" fn hpet_timer_handler(_stack_frame: &mut InterruptStackFrame) {
    // Send notification.
    crate::devices::timer::add_notification();

    unsafe {
        let lapic = &super::super::pic::xapic::LAPIC;
        if let Some(lapic_val) = &*lapic {
            lapic_val.send_eoi(InterruptIndex::HpetTimer.as_u8());
        }
    }
}
