mod _console_vga;
mod gdt;
pub mod interrupts;
pub mod log;

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

/// Architecture level initialization.
pub fn init() {
    gdt::init();

    interrupts::init_idt();
}
