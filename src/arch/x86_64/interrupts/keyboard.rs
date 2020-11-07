use x86_64::structures::idt::InterruptStackFrame;

use super::InterruptIndex;

/// Handler than be used for non-standard faults.
pub extern "x86-interrupt" fn keyboard_handler(_stack_frame: &mut InterruptStackFrame) {
    unsafe {
        use x86_64::instructions::port::Port;
        let mut p1 = Port::<u8>::new(0x60);
        let byte_read = p1.read();

        let mut p2 = Port::<u8>::new(0x61);
        let mut a = p2.read();
        a |= 0x82;
        p2.write(a);
        a &= 0x7f;
        p2.write(a);

        warn!("Scan Code: {}", byte_read);
        let lapic = &super::super::devices::xapic::LAPIC;
        if let Some(lapic_val) = &*lapic {
            lapic_val.send_eoi(InterruptIndex::Keyboard as u8);
        }
    }
}
