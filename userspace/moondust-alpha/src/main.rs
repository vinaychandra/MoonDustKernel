#![no_std]
#![no_main]

#[macro_use]
extern crate moondust_std as std;
#[macro_use]
extern crate alloc;

#[no_mangle]
pub fn main() {
    debug_print!("Syscall!");

    let a = alloc::boxed::Box::new(10u8);
    debug_print!("Test val: {}", a);

    let _thread2 = std::thread::spawn(|| {
        debug_print!("This is from the other thread");
    });

    let b = alloc::boxed::Box::new(99u8);
    debug_print!("Alloc after spawn: {}", b);
}
