mod _console_vga;
mod _gdt;
pub mod log;

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

/// Architecture level initialization.
pub fn init() {
    _gdt::init();
}
