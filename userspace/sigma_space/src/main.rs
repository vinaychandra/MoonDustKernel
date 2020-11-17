#![no_std]
#![no_main]
#![feature(asm)]

#[cfg(not(test))]
use core::panic::PanicInfo;

use mlibc::syscall::SyscallInfo;

#[no_mangle] // don't mangle the name of this function
pub fn _start() {
    let a = SyscallInfo::Test { val: 88 };
    a.invoke();

    let a = SyscallInfo::Test { val: 89 };
    a.invoke();

    let a = SyscallInfo::Exit;
    a.invoke();
}

/// This function is called on panic.
#[panic_handler]
#[cfg(not(test))]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
