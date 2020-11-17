#![no_std]
#![no_main]
#![feature(asm)]

#[cfg(not(test))]
use core::panic::PanicInfo;

use mlibc::syscall::SyscallInfo;

#[no_mangle] // don't mangle the name of this function
pub fn _start() {
    let a = SyscallInfo::Test { val: 88 };
    call_syscall(a);

    let a = SyscallInfo::Test { val: 89 };
    call_syscall(a);

    let a = SyscallInfo::Exit;
    call_syscall(a);
}

/// This function is called on panic.
#[panic_handler]
#[cfg(not(test))]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[inline(always)]
pub fn call_syscall(info: SyscallInfo) {
    unsafe {
        asm!(
            "syscall",
            in("rdi") &info,
            // All caller-saved registers must be marked as clobberred
            out("rax") _, out("rcx") _, out("rdx") _, out("rsi") _,
            out("r8") _, out("r9") _, out("r10") _, out("r11") _,
            // out("xmm0") _, out("xmm1") _, out("xmm2") _, out("xmm3") _,
            // out("xmm4") _, out("xmm5") _, out("xmm6") _, out("xmm7") _,
            // out("xmm8") _, out("xmm9") _, out("xmm10") _, out("xmm11") _,
            // out("xmm12") _, out("xmm13") _, out("xmm14") _, out("xmm15") _,
        )
    }
}
