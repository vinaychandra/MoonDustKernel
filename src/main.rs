//! Project Moondust - Kernel Enty Point.
//! This file is the main entry point for the kernel.

#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(asm)]
#![feature(async_closure)]
#![feature(const_btree_new)]
#![feature(const_fn_fn_ptr_basics)]
#![feature(const_fn)]
#![feature(const_mut_refs)]
#![feature(const_raw_ptr_to_usize_cast)]
#![feature(crate_visibility_modifier)]
#![feature(future_poll_fn)]
#![feature(map_first_last)]
#![feature(naked_functions)]
#![feature(new_uninit)]
#![feature(result_into_ok_or_err)]
#![feature(thread_local)]
#![deny(unsafe_op_in_unsafe_fn)]

use core::{
    panic::PanicInfo,
    sync::atomic::{AtomicUsize, Ordering},
};
use logging::UnifiedLogger;
use moondust_utils::executor::priority_executor::PriorityExecutor;

// BOOTBOOT is autogenerated. So, we ignore a bunch of warnings.
#[allow(dead_code)]
#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
mod bootboot;
#[allow(dead_code)]
mod bootboot2;

pub mod arch;
pub mod common;
pub mod logging;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate alloc;
#[macro_use]
extern crate const_fn_assert;
#[macro_use]
extern crate bitflags;

/// Logger used by the kernel everywhere. This logger is activated by the architecture
/// level startup once the memory is ready.
pub static KERNEL_LOGGER: UnifiedLogger = UnifiedLogger::new();

static PROCESSOR_COUNT: AtomicUsize = AtomicUsize::new(0);

/// Entry point for the Operating System. This calls into the bootstrap
/// of architecture. This is not expected to return because the architecture
/// calls into [main_bsp] and [main_app].
#[no_mangle]
fn _start() -> ! {
    let this_val = PROCESSOR_COUNT.fetch_add(1, Ordering::SeqCst);

    // We only run bsp on the first processor.
    if this_val == 0 {
        crate::arch::bootstrap::initialize_bootstrap_core();
    } else {
        loop {} // Do not run AP Core.
        crate::arch::bootstrap::initialize_ap_core(this_val);
    }
}

pub static SCHEDULER: PriorityExecutor<5> = PriorityExecutor::const_new();

/// Main function on AP Processor.
pub fn main_app() -> ! {
    x86_64::instructions::interrupts::enable();
    loop {}
}

/// Main Function on bootstrap processor.
/// This function should not return.
pub fn main_bsp() -> ! {
    x86_64::instructions::interrupts::enable();
    loop {}
}

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error!("Panic: {}", info);
    info!("====== KERNEL_PANIC ======");
    loop {}
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}
