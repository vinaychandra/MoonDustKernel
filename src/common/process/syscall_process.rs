//! Syscall processing logic for threads.

use core::{panic, task::Poll};

use moondust_sys::syscall::{HeapControl, ProcessControl, Syscalls, Sysrets};

use crate::arch::process::{
    state::{SyscallState, ThreadState},
    Thread,
};

impl Thread {
    /// Process the syscall requested by a thread.
    /// A return of [Poll::Pending] means that the syscall
    /// has been processed and the task can be but back in the list.
    /// In such a case, the waker is used to reschedule the task
    /// when appropriate.
    /// Return of a [Poll::Ready] means the thread exited.
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
                HeapControl::IncreaseHeapBy(size) => {
                    let sysret = {
                        let size_to_increase = *size;
                        let mut kpt = self.get_page_table().lock().await;
                        match kpt.map_more_user_heap(size_to_increase) {
                            Ok((a, b)) => Sysrets::SuccessWithVal2(a as _, b as _),
                            Err(_) => Sysrets::Fail,
                        }
                    };

                    let syscall = self.get_syscall();
                    *syscall.return_data = sysret;
                    syscall.return_data_is_ready = true;
                    syscall.waker.wake_by_ref();
                    Poll::Pending
                }
            },
            Syscalls::Process(ref process_control) => match process_control {
                ProcessControl::CreateThread {
                    ip,
                    stack_size,
                    extra_data,
                } => {
                    let ip = *ip as u64;
                    let extra_data = *extra_data;
                    let stack_size = *stack_size;
                    let mut thread = self.new_empty_thread(stack_size).await;
                    thread.setup_user_ip(ip);
                    thread.setup_user_custom_data(extra_data);

                    let thread_id = thread.thread_id;
                    crate::SCHEDULER
                        .spawn(2, crate::SPAWN_THREADS.get().unwrap().send((thread, 1)))
                        .detach();

                    let syscall = self.get_syscall();
                    *syscall.return_data = Sysrets::SuccessWithVal(thread_id as u64);
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
