#![no_std]
#![feature(alloc_error_handler)]
#![feature(asm)]

use core::panic::PanicInfo;

use moondust_sys::syscall::{heap::Heap, Syscalls};
use moondust_utils::buddy_system_allocator::{self, LockedHeapWithRescue};

pub mod debug;

extern crate alloc;

// TODO: Is it 20?
#[global_allocator]
pub static HEAP: LockedHeapWithRescue<20> = LockedHeapWithRescue::new(expand_heap);

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout);
}

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    debug_print!("User PANIC: {}", info);

    let exit_call = Syscalls::Exit(1);
    exit_call.invoke();
    unreachable!()
}

#[no_mangle]
fn _start() -> ! {
    let current_heap_size = Heap::get_current_heap_size();
    unsafe {
        HEAP.lock()
            .add_to_heap(0x4000_0000_0000, 0x4000_0000_0000 + current_heap_size);
    }

    unsafe { asm!("call main") };
    let exit_call = Syscalls::Exit(0);
    exit_call.invoke();
    unreachable!()
}

fn expand_heap(heap: &mut buddy_system_allocator::Heap<20>) {
    let added_heap = Heap::expand_heap_by(100 * 1024); // TODO: Better number here?
    unsafe {
        heap.add_to_heap(added_heap.0 as _, added_heap.1 as _);
    }
}
