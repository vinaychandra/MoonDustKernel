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

#[cfg(not(test))]
use core::panic::PanicInfo;

#[allow(dead_code)]
#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
mod bootboot;

pub mod arch;
pub mod common;

// Required for -Z build-std flag.
extern crate rlibc;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate bitflags;
extern crate alloc;

#[thread_local]
pub static mut TEST: u8 = 9;

/// Entry point for the Operating System.
#[no_mangle] // don't mangle the name of this function
fn _start() -> ! {
    puts("MootDust Kernel: Pre-Init...");
    arch::initialize_architecture_bsp();
}

/// Main Function on bootstrap processor.
/// This function should not return.
pub fn main_bsp() -> ! {
    puts("MoonDust Kernel: Main function");
    arch::hlt_loop();
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    error!("Panic: {}", info);
    puts("====== KERNEL_PANIC ======");
    loop {}
}

#[alloc_error_handler]
#[cfg(not(test))]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

fn puts(string: &'static str) {
    use bootboot::*;
    unsafe {
        let font: *mut bootboot::psf2_t = &_binary_font_psf_start as *const u64 as *mut psf2_t;
        let (mut kx, mut line, mut mask, mut offs): (u32, u64, u64, u32);
        kx = 0;
        let bpl = ((*font).width + 7) / 8;

        for s in string.bytes() {
            let glyph_a: *mut u8 = (font as u64 + (*font).headersize as u64) as *mut u8;
            let mut glyph: *mut u8 = glyph_a.offset(
                (if s > 0 && (s as u32) < (*font).numglyph {
                    s as u32
                } else {
                    0
                } * ((*font).bytesperglyph)) as isize,
            );
            offs = kx * ((*font).width + 1) * 4;
            for _y in 0..(*font).height {
                line = offs as u64;
                mask = 1 << ((*font).width - 1);
                for _x in 0..(*font).width {
                    let target_location = (&bootboot::fb as *const u8 as u64 + line) as *mut u32;
                    let mut target_value: u32 = 0;
                    if (*glyph as u64) & (mask) > 0 {
                        target_value = 0xFFFFFF;
                    }
                    *target_location = target_value;
                    mask >>= 1;
                    line += 4;
                }
                let target_location = (&bootboot::fb as *const u8 as u64 + line) as *mut u32;
                *target_location = 0;
                glyph = glyph.offset(bpl as isize);
                offs += bootboot.fb_scanline;
            }
            kx += 1;
        }
    }
}
