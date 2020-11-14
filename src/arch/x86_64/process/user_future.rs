//! Rust future that can run user-mode code.

use core::{
    ptr,
    task::{Context, Poll},
};

use futures_lite::Future;
use x86_64::VirtAddr;

use crate::common::memory::stack::Stack;

pub struct UserFuture {
    data: Data,
}

struct Data {
    user_stack: Stack,

    kernel_stack: Stack,

    state: UserProcessState,
}

#[derive(Debug, Copy, Clone)]
struct SyscallId(u64);

#[derive(Debug, Copy, Clone)]
enum UserProcessState {
    NotStarted(*const ()),

    YieldedWithSyscall(*const (), SyscallId),
    ResumeFrom(*const (), SyscallId),
}

impl UserFuture {
    pub fn new(entry_point: *const (), user_stack: Stack, kernel_stack: Stack) -> UserFuture {
        UserFuture {
            data: Data {
                user_stack,
                kernel_stack,
                state: UserProcessState::NotStarted(entry_point),
            },
        }
    }
}

impl Future for UserFuture {
    type Output = u8;

    fn poll(mut self: core::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        unsafe {
            CUR_TASK = &mut self.data as *mut Data;
            CUR_CONTEXT = cx as *mut Context<'_> as *const () as *mut Context<'static>;

            asm!("call {}", sym trampoline_1);
            TRAMPOLINE_1_RETURN
        }
    }
}

#[thread_local]
static mut CUR_TASK: *mut Data = ptr::null_mut();

#[thread_local]
static mut CUR_CONTEXT: *mut Context = ptr::null_mut();

#[thread_local]
static mut TRAMPOLINE_1_RSP_RBP: (u64, u64) = (0, 0);

#[thread_local]
static mut TRAMPOLINE_1_RETURN: Poll<u8> = Poll::Pending;

#[inline(never)]
unsafe extern "C" fn trampoline_1() {
    asm!("uf_trampoline_1_j:"); // Used to skip the stack setting..
    {
        // Store the current stack info
        let rsp: u64;
        let rbp: u64;
        asm!("
                mov {0}, rsp
                mov {1}, rbp",
                 out(reg) rsp, out(reg) rbp);
        TRAMPOLINE_1_RSP_RBP = (rsp, rbp);
    }

    set_syscall_location(syscall_entry_fn as *const ());

    match (*CUR_TASK).state {
        UserProcessState::NotStarted(ep) => {
            // First time run. This will never return
            asm!("
            mov rcx, {0}
            ", in(reg) ep);
            (*CUR_TASK).user_stack.switch_to();
            asm!("sysret");
        }
        UserProcessState::YieldedWithSyscall(ep, id) => {
            (*CUR_TASK).state = UserProcessState::ResumeFrom(ep, id);
            TRAMPOLINE_1_RETURN = Poll::Pending;
            return;
        }
        UserProcessState::ResumeFrom(ep, _id) => {
            asm!("
            mov rcx, {0}
            mov rax, 0
            ", in(reg) ep);
            (*CUR_TASK).user_stack.switch_to();
            asm!("sysret");
        }
    }
}

unsafe fn set_syscall_location(syscall_entry: *const ()) {
    x86_64::registers::model_specific::LStar::write(VirtAddr::new(syscall_entry as u64));
}

#[naked]
unsafe extern "C" fn syscall_entry_fn() {
    (*CUR_TASK).kernel_stack.switch_to();
    asm!("jmp {0}", sym syscall_entry_fn_2);
}

unsafe fn syscall_entry_fn_2() {
    let stored_ip: *const ();
    let info: *const SyscallInfo;
    asm!("", out("ecx") stored_ip, out("rdi") info);

    let info: &'static SyscallInfo = &*info;

    // We retrieved all the syscall data. Process it.
    let _info2: SyscallInfo = *info;

    // After scheduling that, we yield this
    (*CUR_TASK).state = UserProcessState::YieldedWithSyscall(stored_ip, SyscallId(1));
    let (rsp, rbp) = TRAMPOLINE_1_RSP_RBP;
    asm!(
        "
        mov rsp, {0}
        mov rbp, {1}
        jmp uf_trampoline_1_j",
        in(reg) rsp, in(reg) rbp
    );
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub enum SyscallInfo {
    Exit,
    Test { val: u8 },
}
