use core::{panic, task::Poll};

use moondust_sys::syscall::{HeapControl, Syscalls, Sysrets};

use crate::arch::process::{
    state::{SyscallState, ThreadState},
    Thread,
};

impl Thread {
    pub async fn process_syscall(&mut self) -> Poll<u8> {
        let state = &mut self.state;
        let syscall = match state {
            ThreadState::Syscall(a) => a,
            _ => panic!("Cannot process syscall when not in Syscall state"),
        };
        // TODO: sanitize/verify
        match syscall.syscall_info {
            Syscalls::Exit(val) => {
                info!("Thread with id {} exited with code {}", self.thread_id, val);
                Poll::Ready(val)
            }
            Syscalls::Debug { data } => {
                info!("Test syscall with val {}", data);
                *syscall.return_data = Sysrets::NoVal;
                syscall.return_data_is_ready = true;
                syscall.waker.wake_by_ref();
                Poll::Pending
            }
            Syscalls::Heap(ref heap_control) => match heap_control {
                HeapControl::GetCurrentHeapSize => {
                    let heap_size: usize;
                    {
                        let kpt = self.get_page_table().lock().await;
                        heap_size = kpt.get_user_heap_size();
                    }

                    let syscall = self.get_syscall();
                    *syscall.return_data = Sysrets::SuccessWithVal(heap_size as _);
                    syscall.return_data_is_ready = true;
                    syscall.waker.wake_by_ref();
                    Poll::Pending
                }
            },
        }
    }

    fn get_syscall(&mut self) -> &mut SyscallState {
        let state = &mut self.state;
        match state {
            ThreadState::Syscall(a) => a,
            _ => panic!("Cannot process syscall when not in Syscall state"),
        }
    }
}
