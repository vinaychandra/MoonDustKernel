use log::Level;

/// The location where all of memory is mapped to.
pub const MEM_MAP_OFFSET_LOCATION: u64 = 0xFFFF_FF00_0000_0000;

/// Size of a page in bytes.
pub const PAGE_SIZE: usize = 4096;

/// Bytes for stack alignment offset.
pub const STACK_ALIGN: usize = 128;

pub const BSP_STACK_SIZE_BYTES: usize = 4096 * 4;

pub const KERNEL_HEAP_START: usize = 0x_FFFF_FF80_0000_0000;
pub const KERNEL_HEAP_SIZE_INITIAL: usize = 30 * 1024 * 1024;
pub const KERNEL_HEAP_SIZE_TOTAL: usize = 10 * 1024 * 1024 * 1024;

pub const KERNEL_STACK_START: usize = 0x_FFFF_FF90_0000_1000;
pub const KERNEL_STACK_MAX_SIZE: usize = 10 * 1024 * 1024;
pub const KERNEL_STACK_GAP: usize = 0x100_0000;
pub const KERNEL_STACK_TOTAL_SIZE: usize = 10 * 1024 * 1024 * 1024;

/// Number of cores to suppoer
pub const MAX_CORE_COUNT: usize = 512;

/// Log settings
pub const DEFAULT_LOG_LEVEL: Level = Level::Info;
pub const EXTRA_LOGS: [&'static str; 1] = ["bootstrap"];
