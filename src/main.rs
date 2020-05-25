#![no_std]
#![no_main]

use bootloader::{entry_point, BootInfo};
#[cfg(not(test))]
use core::panic::PanicInfo;
use moondust_kernel::*;

entry_point!(kernel_main);

/// Entry point for the Operating System.
#[no_mangle] // don't mangle the name of this function
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    moondust_kernel::initialize_logging();

    kernel_error!("{:X}", boot_info.physical_memory_offset);

    let a = panic as *const () as u64;
    kernel_info!("{:X}", a);

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
