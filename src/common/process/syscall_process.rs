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
        // TODO: sanitize/verify
        match syscall {
            SyscallInfo::Exit { val } => {
                info!("Thread with id {} exited with code {}", self.thread_id, val);
                return Poll::Ready(val);
            }
            SyscallInfo::Debug { ptr, len } => {
                let slice = unsafe { core::slice::from_raw_parts(ptr as *const u8, len as usize) };
                let s = core::str::from_utf8(slice).expect("//TODO! NOT A STR");
                info!("Test syscall with val {}", s);
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
