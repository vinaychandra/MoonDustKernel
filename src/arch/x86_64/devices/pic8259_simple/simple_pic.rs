use super::ChainedPics;
use spin;

/// Offsets for PIC raised interrupts. 32 is the first value available
/// after the inbuilt CPU exceptions. This is for the main PIC.
const PIC_1_OFFSET: u8 = 32;

/// Offsets for PIC raised interrupts. 32 is the first value available
/// after the inbuilt CPU exceptions. This is for the secondary PIC.
const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

pub fn initialize_pic() {
    unsafe { PICS.lock().initialize(None, None) };
}

pub fn disable_pic() {
    unsafe { PICS.lock().initialize(Some(0xff), Some(0xff)) };
}
