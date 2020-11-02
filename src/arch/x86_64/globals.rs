/// The location where all of memory is mapped to.
pub const MEM_MAP_LOCATION: u64 = 0xFFFF_FF00_0000_0000;

/// Size of a page in bytes.
pub const PAGE_SIZE: usize = 4096;

/// Bytes for stack alignment offset.
pub const STACK_ALIGN: usize = 128;

pub const KERNEL_HEAP_START: usize = 0x_FFFF_FF80_0000_0000;
pub const KERNEL_HEAP_SIZE_INITIAL: usize = 1 * 1024 * 1024; // 1 MB
pub const KERNEL_HEAP_SIZE_TOTAL: usize = 10 * 1024 * 1024 * 1024; // 10 GB

pub const KERNEL_STACK_BSP: usize = 0x_FFFF_FF90_0000_0000;
pub const KERNEL_STACK_BSP_SIZE: usize = 1 * 1024 * 1024; // 1 MB
pub const KERNEL_STACK_PRE_ALLOCATED: usize = 10 * 1024 * 1024; // 10 MB
pub const KERNEL_STACK_PER_PROCESS: usize = 100 * 1024; // 100 KiB
pub const KERNEL_STACK_TOTAL_SIZE: usize = 10 * 1024 * 1024 * 1024; // 10 GiB

pub const USER_STACK_END: usize = 0x6FFF_FFFF_FFFF;
pub const USER_STACK_DEFAULT_SIZE: usize = 4 * 1024 * 1024; // 4 MB
