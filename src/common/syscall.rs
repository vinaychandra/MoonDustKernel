use core::task::Waker;

use crate::arch::process::user_future::SyscallInfo;

pub fn process_syscall(info: SyscallInfo, waker: Waker) {
    match info {
        SyscallInfo::Exit => {}
        SyscallInfo::Test { val } => {
            info!("Testing syscall with value: {}", val);
            waker.wake();
        }
    }
}
