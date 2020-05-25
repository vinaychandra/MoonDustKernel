mod _console_vga;
pub mod log;

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
