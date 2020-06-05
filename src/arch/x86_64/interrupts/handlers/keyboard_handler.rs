use crate::arch::interrupts::{pic::xapic::LAPIC, InterruptIndex};
use crate::*;
use x86_64::structures::idt::InterruptStackFrame;

pub extern "x86-interrupt" fn keyboard_handler(_stack_frame: &mut InterruptStackFrame) {
    use x86_64::instructions::port::Port;

    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };

    kernel_info!("Keyboard scan code {}", scancode);

    unsafe {
        let lapic = &LAPIC;
        if let Some(lapic_val) = lapic {
            lapic_val.send_eoi(InterruptIndex::Keyboard.as_u8());
        }
    }
}
