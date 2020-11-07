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

use alloc::string::String;
use common::{graphics, process::Process, ramdisk};
#[cfg(not(test))]
use core::panic::PanicInfo;
use elfloader::ElfBinary;
use logging::UnifiedLogger;
use ramdisk::ustar::UStarArchive;
use tui::layout::Rect;

#[allow(dead_code)]
#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
mod bootboot;
mod bootboot2;

pub mod arch;
pub mod common;
pub mod logging;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate alloc;

extern crate rusttype;
extern crate tui;

#[thread_local]
pub static mut TEST: u8 = 9;

pub static KERNEL_LOGGER: UnifiedLogger = UnifiedLogger::new();

/// Entry point for the Operating System.
#[no_mangle] // don't mangle the name of this function
fn _start() -> ! {
    info!("MootDust Kernel: Pre-Init...");
    arch::initialize_architecture_bsp();
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
    arch::hlt_loop();
}

fn _load_sigma_space(ramdisk: UStarArchive) {
    let file_name = "./userspace/sigma_space";
    let file = ramdisk.lookup(file_name).expect("File not found");

    let process = Process::new();
    let binary = ElfBinary::new(file_name, file).expect("Bad ELF");
    process.load_elf(0x0, binary).unwrap();
}

fn load_graphics() -> Result<(), String> {
    let display;
    let size;
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
        size = Rect::new(0, 0, 80, 50);
        display = common::graphics::fb::FrameBrufferDisplay::new(fb, size, b.fb_width);
    }

    info!("Initializing the UI");
    let terminal = tui::Terminal::new(display).unwrap();
    info!("Terminal created");
    graphics::gui::initialize(terminal);
    info!("Switching to GUI Logging");

    // Initialize GUI logging.
    KERNEL_LOGGER.enable_gui_logger();
    info!("Project Thunderstorm");

    // Setup interrupts
    unsafe {
        crate::arch::interrupts::load_interrupts().unwrap();
    }

    info!("Run completed");
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
