use pc_keyboard::{layouts::Us104Key, HandleControl, Keyboard, ScancodeSet1};
use spin::Mutex;
use x86_64::structures::idt::InterruptStackFrame;

use super::InterruptIndex;

lazy_static! {
    static ref KEYBOARD: Mutex<Keyboard<Us104Key, ScancodeSet1>> =
        Mutex::new(Keyboard::new(Us104Key, ScancodeSet1, HandleControl::Ignore));
}

/// Handler than be used for non-standard faults.
pub extern "x86-interrupt" fn keyboard_handler(_stack_frame: &mut InterruptStackFrame) {
    unsafe {
        use x86_64::instructions::port::Port;
        let mut p1 = Port::<u8>::new(0x60);
        let scan_code = p1.read();

        let mut keyboard;
        {
            keyboard = KEYBOARD.lock();
            if let Ok(Some(key_event)) = keyboard.add_byte(scan_code) {
                crate::tasks::keyboard::add_scancode(key_event);
            }
        }

        let lapic = &super::super::devices::xapic::LAPIC;
        if let Some(lapic_val) = &*lapic {
            lapic_val.send_eoi(InterruptIndex::Keyboard as u8);
        }
    }
}
