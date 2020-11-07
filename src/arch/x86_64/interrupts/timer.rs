use x86_64::structures::idt::InterruptStackFrame;

pub extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: &mut InterruptStackFrame) {
    unsafe {
        info!("TOCK");

        let lapic = &super::LAPIC;
        if let Some(lapic_val) = &*lapic {
            lapic_val.end_of_interrupt();
        }
    }
}

pub extern "x86-interrupt" fn hpet_timer_handler(_stack_frame: &mut InterruptStackFrame) {
    info!("TICK");

    unsafe {
        let lapic = &super::LAPIC;
        if let Some(lapic_val) = &*lapic {
            lapic_val.end_of_interrupt();
        }
    }
}
