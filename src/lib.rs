#![no_std]
#![feature(alloc_error_handler)]
#![feature(abi_x86_interrupt)]
#![feature(never_type)]
#![feature(asm)]
#![feature(const_fn)]
#![feature(const_in_array_repeat_expressions)]
#![feature(thread_local)]
#![feature(new_uninit)]
#![feature(wake_trait)]
#![feature(const_btree_new)]
#![feature(map_first_last)]
#![feature(duration_constants)]

extern crate alloc;

pub mod arch;
pub mod devices;
pub mod log;
pub mod memory;
pub mod tasks;

#[macro_use]
extern crate lazy_static;

/// Initialize logs for the kernel.
pub fn initialize_logging() {
    log::init_bootstrap_log();
}

#[alloc_error_handler]
#[cfg(not(test))]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}
