use core::{panic, pin::Pin, task::Poll};

use moondust_sys::syscall::{Syscalls, Sysrets};

use crate::arch::process::{state::ThreadState, Thread};

impl Thread {
    pub fn process_syscall(mut self: Pin<&mut Self>) -> Poll<u8> {
        let state = &mut self.state;
        let syscall = match state {
            ThreadState::Syscall(a) => a,
            _ => panic!("Cannot process syscall when not in Syscall state"),
        };
        // TODO: sanitize/verify
        match syscall.syscall_info {
            Syscalls::Exit(val) => {
                info!("Thread with id {} exited with code {}", self.thread_id, val);
                return Poll::Ready(val);
            }
            Syscalls::Debug { data } => {
                info!("Test syscall with val {}", data);
                *syscall.return_data = Sysrets::NoVal;
                syscall.return_data_is_ready = true;
                syscall.waker.wake_by_ref();
                return Poll::Pending;
            }
        }
    }
}
