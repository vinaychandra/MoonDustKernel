pub mod memory;

/// Align value upwards.
///
/// Returns the smallest x with alignment `align` so that x >= addr. The alignment must be
/// a power of 2.
#[inline]
pub const fn align_up(value: usize, align: usize) -> usize {
    cfn_assert!(align.is_power_of_two());
    let align_mask = align - 1;
    if value & align_mask == 0 {
        value // already aligned
    } else {
        (value | align_mask) + 1
    }
}

/// Align value downwards.
///
/// Returns the greatest x with alignment `align` so that x <= addr. The alignment must be
///  a power of 2.
#[inline]
pub const fn align_down(addr: usize, align: usize) -> usize {
    cfn_assert!(align.is_power_of_two());
    addr & !(align - 1)
}
