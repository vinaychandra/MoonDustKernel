#![no_std]
#![no_main]

#[macro_use]
extern crate moondust_std;
extern crate alloc;

#[no_mangle]
pub fn main() {
    debug_print!("Syscall!");

    let a = alloc::boxed::Box::new(10u8);
    debug_print!("{}", a);
    let b = alloc::boxed::Box::new(99u8);
    debug_print!("{}", b);

    let _b: alloc::vec::Vec<u8> = alloc::vec::Vec::with_capacity(45 * 1024);
}
