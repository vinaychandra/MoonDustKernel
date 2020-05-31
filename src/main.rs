#![no_std]
#![no_main]

use alloc::boxed::Box;
use bootloader::{entry_point, BootInfo};
#[cfg(not(test))]
use core::panic::PanicInfo;
use moondust_kernel::*;

extern crate alloc;

entry_point!(kernel_main);

/// Entry point for the Operating System.
#[no_mangle] // don't mangle the name of this function
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    // Initialize logging so that data can be seen on screen
    moondust_kernel::initialize_logging();

    // Initialize architecture
    arch::init(boot_info);

    // test box
    let _test = Box::new(10u64);

    x86_64::instructions::interrupts::enable();

    kernel_error!("kernel loop ended.");
    arch::hlt_loop()
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kernel_error!("PANIC: {}", info);
    arch::hlt_loop()
}
