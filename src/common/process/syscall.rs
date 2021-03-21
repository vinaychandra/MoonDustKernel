use core::{
    pin::Pin,
    task::{Context, Poll},
};

use moondust_sys::syscall::{SyscallInfo, SysretInfo};

use crate::arch::process::state::{Registers, SysretWrapper, ThreadState};

pub struct UserFuture<'a> {
    pub thread_id: usize,
    pub thread_state: &'a mut ThreadState,
}

impl<'a> UserFuture<'a> {
    pub fn new(thread_id: usize, thread_state: &'a mut ThreadState) -> Self {
        Self {
            thread_id,
            thread_state,
        }
    }

    pub fn process_syscall(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        syscall: SyscallInfo,
        registers: Registers,
    ) -> Poll<u8> {
        match syscall {
            SyscallInfo::Exit { val } => return Poll::Ready(val),
            SyscallInfo::Test { val } => {
                info!("Test syscall with val {}", val);
                *self.thread_state = ThreadState::Syscall {
                    registers,
                    syscall_info: Some(syscall),
                    sysret_data: Some(SysretWrapper {
                        waker: cx.waker().clone(),
                        info: SysretInfo::NoVal,
                    }),
                };
                return Poll::Pending;
            }
        }
    }
}
