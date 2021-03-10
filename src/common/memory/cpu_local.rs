//! CPU Local support
//!
//! In the kernel, we use `#[thread_local]` attribute for CPU local variables.
//! The data is loaded into `tbss` and `tdata` sections.

use alloc::boxed::Box;
use core::ptr;

/// Load TLS data into memory and return its physical address.
/// All sizes are in bytes.
/// # Arguments
/// - `start_addr`: Starting virtual address for TLS segment.
/// - `tdata_size`: The number of data bytes in the template. Corresponds to
///         the length of the `.tdata` section.
/// - `total_size`: The total number of bytes that the TLS segment should have in memory.
///         Generally corresponds to the combined length of the `.tdata` and `.tbss` sections.
/// # Returns
/// Virtual address of the target pointer. The data will be of size `total_size`
#[cfg(target_arch = "x86_64")]
pub unsafe fn load_tls_data(
    start_addr: *const u8,
    tdata_size: usize,
    total_size: usize,
) -> *mut [u8] {
    // We add 8 bytes to have storage to store fs pointer.
    let mut values = Box::<[u8]>::new_uninit_slice(total_size);
    ptr::copy(start_addr, values.as_mut_ptr() as *mut u8, tdata_size);
    ptr::write_bytes(
        ((values.as_mut_ptr() as usize) + tdata_size) as *mut u8,
        0,
        total_size - tdata_size,
    );
    let final_val = values.assume_init();
    Box::into_raw(final_val)
}
