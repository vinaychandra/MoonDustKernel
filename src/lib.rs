#![no_std]
#![feature(alloc_error_handler)]
#![feature(abi_x86_interrupt)]
#![feature(never_type)]
#![feature(asm)]
#![feature(const_fn)]

pub mod arch;
pub mod log;

/// Initialize logs for the kernel.
pub fn initialize_logging() {
    log::init_bootstrap_log();
}
