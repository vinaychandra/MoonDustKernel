use core::task::Waker;

use moondust_sys::syscall::{SyscallInfo, SysretInfo};

#[derive(Default, Debug)]
pub struct Registers {
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,

    pub rax: u64,
    pub rbx: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,

    pub rsp: u64,
    pub rbp: u64,

    pub rip: u64,
    pub rflags: u64,
}

#[derive(Debug)]
pub enum ThreadState {
    Running,

    /// This thread has syscall'ed.
    Syscall {
        registers: Registers,
        syscall_info: Option<SyscallInfo>,
        sysret_data: Option<SysretWrapper>,
    },
}

#[derive(Debug)]
pub struct SysretWrapper {
    pub info: SysretInfo,
    pub waker: Waker,
}
