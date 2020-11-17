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

unsafe impl Send for UserFuture {}
unsafe impl Sync for UserFuture {}

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

    YieldedWithSyscall(*const (), *const (), *const (), SyscallId),
    ResumeFrom(*const (), *const (), *const (), SyscallId),
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
            let us = &(*CUR_TASK).user_stack;
            let ptrs = us.get_stack_pointers();
            asm!("
            mov rsp, {0}
            mov rbp, {1}
            sysretq
            ", in(reg) ptrs.1, in(reg) ptrs.0, in("rcx") ep);
        }
        UserProcessState::YieldedWithSyscall(ep, a, b, id) => {
            (*CUR_TASK).state = UserProcessState::ResumeFrom(ep, a, b, id);
            TRAMPOLINE_1_RETURN = Poll::Pending;
            return;
        }
        UserProcessState::ResumeFrom(ep, rsp, rbp, _id) => {
            asm!("
            mov rcx, {0}
            mov rax, 0
            mov rsp, {1}
            mov rbp, {2}
            ", in(reg) ep, in(reg) rsp, in(reg) rbp);
            asm!("sysretq");
        }
    }
}

unsafe fn set_syscall_location(syscall_entry: *const ()) {
    x86_64::registers::model_specific::LStar::write(VirtAddr::new(syscall_entry as u64));
}

// Syscall: rcx -> rdi (IP) ... rdi -> info
#[inline(never)]
#[naked]
unsafe extern "C" fn syscall_entry_fn(
    _info: *const SyscallInfo,
    _b: u64,
    _c: u64,
    _stored_ip: u64,
) {
    // naked to retrieve the values and not corrupt stack.
    asm!("
        mov rsi, rsp
        mov rdx, rbp
        jmp {0}
    ", sym syscall_entry_fn_2);

    // (*CUR_TASK).kernel_stack.switch_to();
}

unsafe extern "C" fn syscall_entry_fn_2(
    info: *const SyscallInfo,
    rsp: *const (),
    rbp: *const (),
    stored_ip: *const (),
) {
    SYSCALL_RBP = rbp;
    SYSCALL_RSP = rsp;
    SYSCALL_RIP = stored_ip;
    SYSCALL_INFO = info;
    (*CUR_TASK).kernel_stack.switch_to();
    asm!("jmp {}", sym syscall_entry_fn_3)
}

#[thread_local]
static mut SYSCALL_INFO: *const SyscallInfo = core::ptr::null();

#[thread_local]
static mut SYSCALL_RSP: *const () = core::ptr::null();

#[thread_local]
static mut SYSCALL_RBP: *const () = core::ptr::null();

#[thread_local]
static mut SYSCALL_RIP: *const () = core::ptr::null();

unsafe fn syscall_entry_fn_3() {
    let info = SYSCALL_INFO;
    let stored_ip = SYSCALL_RIP;

    // We retrieved all the syscall data. Process it.
    crate::common::syscall::process_syscall(*info, (*CUR_CONTEXT).waker().clone());

    // After scheduling that, we yield this
    (*CUR_TASK).state =
        UserProcessState::YieldedWithSyscall(stored_ip, SYSCALL_RSP, SYSCALL_RBP, SyscallId(1));
    let (rsp, rbp) = TRAMPOLINE_1_RSP_RBP;
    asm!("
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
