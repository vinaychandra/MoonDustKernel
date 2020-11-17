#![no_std]
#![no_main]
#![feature(asm)]

#[cfg(not(test))]
use core::panic::PanicInfo;

#[no_mangle] // don't mangle the name of this function
pub fn _start() {
    let a = SyscallInfo::Test { val: 88 };

    unsafe {
        asm!(
        "
        nop
        syscall
        nop
        syscall
        ", in("rdi") &a,
        );
    }
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub enum SyscallInfo {
    Exit,
    Test { val: u8 },
}

/// This function is called on panic.
#[panic_handler]
#[cfg(not(test))]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
