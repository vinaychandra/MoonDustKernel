//! Project Moondust - Kernel Enty Point.
//! This file is the main entry point for the kernel.

#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
#![feature(alloc_error_handler)]
#![feature(asm)]
#![feature(const_fn)]
#![feature(const_in_array_repeat_expressions)]
#![feature(naked_functions)]
#![feature(new_uninit)]
#![feature(thread_local)]
#![feature(wake_trait)]
#![feature(const_mut_refs)]
#![feature(const_fn_fn_ptr_basics)]
#![feature(future_poll_fn)]
#![feature(const_btree_new)]

use alloc::string::String;
use arch::{globals, process::preemptable_future::PreemptableFuture};
use common::{
    executor::priority::{Priority, PriorityExecutor},
    graphics,
    memory::stack::Stack,
    ramdisk,
};
use core::{
    panic::PanicInfo,
    sync::atomic::{AtomicU8, Ordering},
};
use logging::UnifiedLogger;

// BOOTBOOT is autogenerated. So, we ignore a bunch of warnings.
#[allow(dead_code)]
#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
mod bootboot;
mod bootboot2;

pub mod arch;
pub mod common;
pub mod logging;
pub mod sync;
pub mod tasks;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate alloc;

/// Logger used by the kernel everywhere. This logger is activated by the architecture
/// level startup once the memory is ready.
pub static KERNEL_LOGGER: UnifiedLogger = UnifiedLogger::new();

static PROCESSOR_COUNT: AtomicU8 = AtomicU8::new(0);

/// Entry point for the Operating System. This calls into the bootstrap
/// of architecture. This is not expected to return because the architecture
/// calls into [main_bsp] and [main_app].
#[no_mangle]
fn _start() -> ! {
    info!("MootDust Kernel: Pre-Init...");

    let this_val = PROCESSOR_COUNT.fetch_add(1, Ordering::SeqCst);

    // We only run bsp on the first processor.
    if this_val == 0 {
        arch::initialize_architecture_bsp();
    } else {
        main_app()
    }
}

/// Main function on AP Processor.
pub fn main_app() -> ! {
    arch::hlt_loop();
}

/// Main Function on bootstrap processor.
/// This function should not return.
pub fn main_bsp() -> ! {
    info!("MoonDust Kernel: Main function");

    unsafe {
        let ramdisk = ramdisk::ustar::UStarArchive::new(
            bootboot::bootboot.initrd_ptr as *const u8,
            bootboot::bootboot.initrd_size as usize,
        );

        info!(target: "main", "initrd image is {}", ramdisk);
    }

    load_graphics().unwrap();

    // Setup interrupts
    unsafe {
        crate::arch::interrupts::load_interrupts().unwrap();
    }

    info!("Run completed");

    let exec = PriorityExecutor::new();
    let f = tasks::keyboard::print_keypresses();
    let new_stack = Stack::new_kernel_stack(globals::PAGE_SIZE * 10);
    let f2 = PreemptableFuture::new(f, new_stack);

    exec.spawn(Priority::Medium, f2).detach();

    info!(
        "Startup duration is {:?}",
        crate::common::time::get_uptime()
    );

    info!(
        "Current time is {}",
        crate::common::time::get_current_time()
    );

    crate::arch::process::block_on(exec.run());
    arch::hlt_loop();
}

fn load_graphics() -> Result<(), String> {
    let display;
    unsafe {
        let fb_raw = &bootboot::fb as *const u8 as *mut u32;
        let b = bootboot::bootboot;
        assert!(
            b.fb_scanline == b.fb_width * 4,
            "Scanline must be the same size as width * 4. Not implemented the non equal scenario."
        );
        let fb = core::slice::from_raw_parts_mut(
            fb_raw,
            (bootboot::bootboot.fb_height * bootboot::bootboot.fb_width) as usize,
        );
        display = common::graphics::fb::FrameBrufferDisplay::new(
            fb,
            b.fb_width as u16,
            b.fb_height as u16,
        );
    }

    info!("Initializing the UI");
    let terminal = tui::Terminal::new(display).unwrap();
    info!("Terminal created");
    graphics::gui::initialize(terminal);
    info!("Switching to GUI Logging");

    // Initialize GUI logging.
    KERNEL_LOGGER.enable_gui_logger();
    info!("Project Thunderstorm");

    Ok(())
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error!("Panic: {}", info);
    info!("====== KERNEL_PANIC ======");
    loop {}
}

#[alloc_error_handler]
#[cfg(not(test))]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

// TODO: Missing fminf in compiler-builtins for soft-float
// BODY: See https://github.com/rust-lang/rust/issues/62729.
// BODY:
// BODY: As a workaround, we include the functions in libuser for now.
/// Workaround rust-lang/rust#62729
#[no_mangle]
#[doc(hidden)]
pub unsafe extern "C" fn fminf(x: f32, y: f32) -> f32 {
    libm::fminf(x, y)
}

// TODO: Missing fmaxf in compiler-builtins for soft-float
// BODY: See https://github.com/rust-lang/rust/issues/62729.
// BODY:
// BODY: As a workaround, we include the functions in libuser for now.
/// Workaround rust-lang/rust#62729
#[no_mangle]
#[doc(hidden)]
pub unsafe extern "C" fn fmaxf(x: f32, y: f32) -> f32 {
    libm::fmaxf(x, y)
}
