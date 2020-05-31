use crate::*;
use x86_64::structures::idt::InterruptStackFrame;

pub extern "x86-interrupt" fn double_fault_handler(
    stack_frame: &mut InterruptStackFrame,
    error_code: u64,
) -> ! {
    kernel_error!(
        "EXCEPTION: DOUBLE FAULT\n{:#?}\nError Code: {}",
        stack_frame,
        error_code
    );
    arch::hlt_loop()
}
