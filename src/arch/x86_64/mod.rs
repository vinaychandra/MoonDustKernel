pub mod serial;

use core::alloc::GlobalAlloc;

use self::serial::SerialLogger;

/// Logger that uses serial to output logs.
/// Architecture level logs for x86_64.
pub static LOGGER: SerialLogger = SerialLogger;

#[global_allocator]
static ALLOC: PanicAlloc = PanicAlloc;

struct PanicAlloc;

unsafe impl GlobalAlloc for PanicAlloc {
    unsafe fn alloc(&self, _layout: core::alloc::Layout) -> *mut u8 {
        panic!("Alloc requested");
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: core::alloc::Layout) {
        panic!("Dealloc requested");
    }
}
