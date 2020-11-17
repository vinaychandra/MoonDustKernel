#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub enum SyscallInfo {
    Exit,
    Test { val: u8 },
}
