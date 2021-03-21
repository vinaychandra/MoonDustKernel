#![no_std]
#![no_main]
#![feature(asm)]

use core::panic::PanicInfo;

use moondust_sys::syscall::SyscallInfo;

#[no_mangle] // don't mangle the name of this function
pub fn _start() {
    let a = SyscallInfo::Test { val: 10 };
    a.invoke();
}

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
