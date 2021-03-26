#![no_std]
#![no_main]
#![feature(asm)]

use core::panic::PanicInfo;

use moondust_sys::syscall::{heap::Heap, Syscalls};

#[macro_use]
extern crate moondust_std;

#[no_mangle] // don't mangle the name of this function
pub fn _start() {
    unsafe { asm!("nop") };
    unsafe { asm!("nop") };
    unsafe { asm!("nop") };
    debug_print!("Syscall!");

    let current_heap_size = Heap::get_current_heap_size();
    debug_print!("Current heap size is {} bytes", current_heap_size);

    let a = Syscalls::Exit(10);
    a.invoke();
}

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
