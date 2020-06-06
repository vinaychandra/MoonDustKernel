use crate::*;
use x86_64::structures::idt::{InterruptStackFrame, PageFaultErrorCode};

pub extern "x86-interrupt" fn page_fault_handler(
    stack_frame: &mut InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    let addr = x86_64::registers::control::Cr2::read();

    kernel_info!("EXCEPTION: PAGE FAULT");
    kernel_info!("Accessed Address: {:?}", addr);
    kernel_info!("Error Code: {:?}", error_code);
    kernel_info!("{:#?}", stack_frame);
    arch::hlt_loop();
}
