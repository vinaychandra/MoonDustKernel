use core::{
    pin::Pin,
    task::{Context, Poll},
};

use moondust_sys::syscall::{SyscallInfo, SysretInfo};

use crate::arch::process::{
    state::{Registers, SysretWrapper, ThreadState},
    Thread,
};

impl Thread {
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
                self.state = ThreadState::Syscall {
                    registers,
                    syscall_info: Some(syscall),
                    sysret_data: Some(SysretWrapper {
                        waker: cx.waker().clone(),
                        info: SysretInfo::NoVal,
                    }),
                };
                cx.waker().wake_by_ref();
                return Poll::Pending;
            }
        }
    }
}
