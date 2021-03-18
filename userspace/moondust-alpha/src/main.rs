#![no_std]
#![no_main]
#![feature(asm)]

#[cfg(not(test))]
use core::panic::PanicInfo;

#[no_mangle] // don't mangle the name of this function
pub fn _start() {}

/// This function is called on panic.
#[panic_handler]
#[cfg(not(test))]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
