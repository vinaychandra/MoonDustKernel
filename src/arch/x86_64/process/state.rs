use core::task::Waker;

use moondust_sys::syscall::{Syscalls, Sysrets};

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

impl Registers {
    pub const fn new() -> Registers {
        Self {
            rcx: 0,
            rdx: 0,
            rsi: 0,
            rdi: 0,

            rax: 0,
            rbx: 0,
            r8: 0,
            r9: 0,
            r10: 0,
            r11: 0,
            r12: 0,
            r13: 0,
            r14: 0,
            r15: 0,

            rsp: 0,
            rbp: 0,

            rip: 0,
            rflags: 0,
        }
    }
}

#[derive(Debug)]
pub enum ThreadState {
    Running,

    /// This thread has syscall'ed.
    Syscall(SyscallState),

    /// Thread has not started
    NotStarted(Registers),
}

#[derive(Debug)]
pub struct SyscallState {
    pub registers: Registers,
    pub syscall_info: Syscalls,
    pub waker: Waker,

    pub return_data_is_ready: bool,
    pub return_data: &'static mut Sysrets,
}
