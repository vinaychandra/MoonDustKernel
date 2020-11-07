//! Common module.
//! This module presents common shared code between multiple architectures.

pub mod devices;
pub mod graphics;
pub mod memory;
pub mod process;
pub mod ramdisk;
pub mod syscall;

/// Align value downwards.
///
/// Returns the greatest x with alignment `align` so that x <= addr. The alignment must be
///  a power of 2.
#[inline]
pub fn align_down(addr: u64, align: u64) -> u64 {
    assert!(align.is_power_of_two(), "`align` must be a power of two");
    addr & !(align - 1)
}

/// Align value upwards.
///
/// Returns the smallest x with alignment `align` so that x >= addr. The alignment must be
/// a power of 2.
#[inline]
pub fn align_up(value: u64, align: u64) -> u64 {
    assert!(align.is_power_of_two(), "`align` must be a power of two");
    let align_mask = align - 1;
    if value & align_mask == 0 {
        value // already aligned
    } else {
        (value | align_mask) + 1
    }
}

pub unsafe fn extend_lifetime<'b, T>(r: &'b T) -> &'static T {
    core::mem::transmute::<&'b T, &'static T>(r)
}
