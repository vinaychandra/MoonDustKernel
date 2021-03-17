use x86_64::structures::idt::InterruptStackFrame;

/// Handler than be used for timer
pub extern "x86-interrupt" fn timer_handler(_stack_frame: &mut InterruptStackFrame) {
    unsafe {
        info!("T1");
        let lapic = &mut crate::arch::LAPIC;
        let eoi = lapic.end_of_interrupt();
        eoi.signal();
    }
}
