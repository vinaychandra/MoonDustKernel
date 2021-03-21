#[derive(Debug, Clone)]
#[repr(C)]
pub enum SyscallInfo {
    Exit { val: u8 },
    Test { val: u8 },
}

#[derive(Debug, Clone)]
#[repr(C)]
pub enum SysretInfo {
    NoVal,
}
