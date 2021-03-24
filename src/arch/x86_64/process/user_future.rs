use core::{
    pin::Pin,
    task::{Context, Poll},
};

use futures_lite::Future;
use moondust_sys::syscall::SyscallWrapper;
use x86_64::{registers::model_specific::LStar, VirtAddr};

use super::{
    state::{Registers, SyscallState, ThreadState},
    Thread,
};
use crate::arch::cpu_locals;

impl Future for Thread {
    type Output = u8;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        user_switching_fn(self, cx)
    }
}

fn user_switching_fn(mut thread: Pin<&mut Thread>, cx: &mut Context<'_>) -> Poll<u8> {
    // TODO: This can cause busy wait when there is nothing else to run.
    if !thread.try_activate() {
        cx.waker().wake_by_ref();
        return Poll::Pending;
    }

    cpu_locals::CURRENT_THREAD_ID.set(thread.thread_id);
    let thread_id = thread.thread_id;

    debug!(target: "user_future", "[CPU:{}][Thread:{}] Resuming thread", cpu_locals::PROCESSOR_ID.get(), thread_id);
    unsafe {
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
    }

    // TODO: we only need to set this once.
    set_syscall_location(syscall_entry_fn as *const ());

    match &mut thread.state {
        ThreadState::Running => {
            panic!("Thread is already in running state!");
        }
        ThreadState::NotStarted(registers) => {
            // No syscall, we can continue this.
            // Although this is a "noreturn", we do not add the option so that rust doesn't
            // optimize out the remaning statements.
            debug!(target: "user_future",
                "[CPU:{}][Thread:{}] Thread state was not started. Starting now.",
                cpu_locals::PROCESSOR_ID.get(),
                thread_id);
            unsafe {
                asm!("
                        cli
                        mov rsp, rdi
                        mov rbp, rsi
                        sysretq
                    ", in("rdi") registers.rsp, in("rsi") registers.rbp, in("rax") registers.rax,
                    in("rbx") registers.rbx, in("rcx") registers.rip, in("rdx") registers.rdx,
                    in("r8") registers.r8, in("r9") registers.r9, in("r10") registers.r10,
                    in("r11") registers.rflags, in("r12") registers.r12, in("r13") registers.r13,
                    in("r14") registers.r14, in("r15") registers.r15);
            }
        }
        ThreadState::Syscall(state) => {
            if !state.return_data_is_ready {
                debug!(target: "user_future",
                    "[CPU:{}][Thread:{}] Thread state was syscall but data is not ready.",
                    cpu_locals::PROCESSOR_ID.get(),
                    thread_id);
                return Poll::Pending;
            }

            debug!(target: "user_future",
                "[CPU:{}][Thread:{}] Thread state was syscall and returning to user.",
                cpu_locals::PROCESSOR_ID.get(),
                thread_id);
            let registers = &mut state.registers;
            unsafe {
                asm!("
                        cli
                        mov rsp, rdi
                        mov rbp, rsi
                        sysretq
                    ", in("rdi") registers.rsp, in("rsi") registers.rbp, in("rax") registers.rax,
                    in("rbx") registers.rbx, in("rcx") registers.rip, in("rdx") registers.rdx,
                    in("r8") registers.r8, in("r9") registers.r9, in("r10") registers.r10,
                    in("r11") registers.rflags, in("r12") registers.r12, in("r13") registers.r13,
                    in("r14") registers.r14, in("r15") registers.r15);
            }
        }
    }

    let regs: Registers;
    let syscall_state: SyscallState;
    unsafe {
        let retrieved_syscall: *mut SyscallWrapper;
        asm!(
            "user_future_resume_point:
            nop
            sti
            ",
            out("rax") _, out("rbx") _, out("rcx") _, out("rdx") _, out("rsi") _,
            out("rdi") retrieved_syscall, out("r8") _, out("r9") _, out("r10") _, out("r11") _, out("r12") _,
            out("r13") _, out("r14") _, out("r15") _,
        );
        regs = REGISTERS.take().expect("Expected REGISTERS after sysret");
        debug!(target: "user_future",
            "[CPU:{}][Thread:{}] Thread returned from usermode by making a syscall.",
            cpu_locals::PROCESSOR_ID.get(),
            thread_id);
        syscall_state = SyscallState {
            registers: regs,
            syscall_info: (*retrieved_syscall).call_info.clone(),
            waker: cx.waker().clone(),
            return_data_is_ready: false,
            return_data: &mut (*retrieved_syscall).return_info,
        };
    }

    thread.state = ThreadState::Syscall(syscall_state);

    // User's syscall reaches here. Now process it.
    return thread.process_syscall();
}

#[thread_local]
static mut TRAMPOLINE_1_RSP_RBP: (u64, u64) = (0, 0);

fn set_syscall_location(syscall_entry: *const ()) {
    LStar::write(VirtAddr::new(syscall_entry as u64));
}

// Syscall: rcx -> rdi (IP) ... rdi -> info
#[inline(never)]
#[naked]
unsafe extern "C" fn syscall_entry_fn(
    _info: *const SyscallWrapper,
    _b: u64,
    _c: u64,
    _stored_ip: u64,
) {
    // naked to retrieve the values and not corrupt stack.
    unsafe {
        asm!("
        mov rsi, rsp
        mov rdx, rbp
        jmp {0}
    ", sym syscall_entry_fn_2, options(noreturn));
    }
}

#[thread_local]
static mut REGISTERS: Option<Registers> = None;

unsafe extern "C" fn syscall_entry_fn_2(
    info: *const SyscallWrapper,
    user_rsp: *const (),
    user_rbp: *const (),
    user_stored_ip: *const (),
) {
    unsafe {
        let rbx: u64;
        let r12: u64;
        let r13: u64;
        let r14: u64;
        let r15: u64;
        let rflags: u64;
        asm!("nop", 
            out("rbx") rbx, out("r12") r12, out("r13") r13,
            out("r14") r14, out("r15") r15, out("r11") rflags);

        let mut regs = Registers::new();
        regs.rsp = user_rsp as u64;
        regs.rbp = user_rbp as u64;
        regs.rip = user_stored_ip as u64;
        regs.rbx = rbx;
        regs.r12 = r12;
        regs.r13 = r13;
        regs.r14 = r14;
        regs.r15 = r15;
        regs.rflags = rflags;
        REGISTERS = Some(regs);

        let (rsp, rbp) = TRAMPOLINE_1_RSP_RBP;
        asm!(
            "
            mov rbp, {1}
            mov rsp, {0}
            jmp user_future_resume_point
        ", in(reg) rsp, in(reg) rbp,  in("rdi") info);
    }
}
