use core::{
    pin::Pin,
    task::{Context, Poll},
};

use futures_lite::Future;
use moondust_sys::syscall::{SyscallInfo, SysretInfo};
use x86_64::{registers::model_specific::LStar, VirtAddr};

use super::{
    state::{Registers, ThreadState},
    Thread,
};

impl Future for Thread {
    type Output = u8;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // TODO: Activate thread?
        unsafe {
            {
                // Store the current stack info
                let rsp: u64;
                let rbp: u64;
                let rip: u64;
                asm!("
                        mov {0}, rsp
                        mov {1}, rbp
                        mov {2}, user_future_resume_point",
                         out(reg) rsp, out(reg) rbp, out(reg) rip);
                TRAMPOLINE_1_RSP_RBP_RIP = (rsp, rbp, rip);
            }
        }

        // TODO: we only need to set this once.
        set_syscall_location(syscall_entry_fn as *const ());

        match &mut self.state {
            ThreadState::Running => {
                panic!("Thread is already in running state!");
            }
            ThreadState::Syscall {
                registers,
                syscall_info,
                sysret_data,
            } => {
                if let Some(_scall) = syscall_info {
                    match sysret_data {
                        Some(sysret) => {
                            let info = &sysret.info;
                            match info {
                                SysretInfo::NoVal => unsafe {
                                    asm!("
                                            mov rsp, rdi
                                            mov rbp, rsi
                                            sysretq
                                        ", in("rdi") registers.rsp, in("rsi") registers.rbp, in("rax") registers.rax,
                                        in("rbx") registers.rbx, in("rcx") registers.rip, in("rdx") registers.rdx,
                                        in("r8") registers.r8, in("r9") registers.r9, in("r10") registers.r10,
                                        in("r11") registers.r11, in("r12") registers.r12, in("r13") registers.r13,
                                        in("r14") registers.r14, in("r15") registers.r15);
                                },
                            }
                        }
                        None => return Poll::Pending,
                    }
                } else {
                    // No syscall, we can continue this.
                    // Although this is a "noreturn", we do not add the option so that rust doesn't
                    // optimize out the remaning statements.
                    unsafe {
                        asm!("
                            mov rsp, rdi
                            mov rbp, rsi
                            sysretq
                        ", in("rdi") registers.rsp, in("rsi") registers.rbp, in("rax") registers.rax,
                        in("rbx") registers.rbx, in("rcx") registers.rip, in("rdx") registers.rdx,
                        in("r8") registers.r8, in("r9") registers.r9, in("r10") registers.r10,
                        in("r11") registers.r11, in("r12") registers.r12, in("r13") registers.r13,
                        in("r14") registers.r14, in("r15") registers.r15);
                    }
                }
            }
        }

        let mut regs: Registers = Default::default();
        let syscall: SyscallInfo;
        unsafe {
            let retrieved_syscall: *const SyscallInfo;
            asm!(
                "user_future_resume_point:
                nop
                ",
                out("rax") _, out("rbx") _, out("rcx") regs.rip, out("rdx") regs.rbp, out("rsi") regs.rsp,
                out("rdi") retrieved_syscall, out("r8") _, out("r9") _, out("r10") _, out("r11") _, out("r12") _,
                out("r13") _, out("r14") _, out("r15") _,
            );
            syscall = (&*retrieved_syscall).clone();
        }

        // User's syscall reaches here. Now process it.
        return self.process_syscall(cx, syscall, regs);
    }
}

#[thread_local]
static mut TRAMPOLINE_1_RSP_RBP_RIP: (u64, u64, u64) = (0, 0, 0);

fn set_syscall_location(syscall_entry: *const ()) {
    LStar::write(VirtAddr::new(syscall_entry as u64));
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
    unsafe {
        asm!("
        mov rsi, rsp
        mov rdx, rbp
        jmp {0}
    ", sym syscall_entry_fn_2, options(noreturn));
    }
}

unsafe extern "C" fn syscall_entry_fn_2(
    info: *const SyscallInfo,
    user_rsp: *const (),
    user_rbp: *const (),
    user_stored_ip: *const (),
) {
    unsafe {
        let (rsp, rbp, rip) = TRAMPOLINE_1_RSP_RBP_RIP;
        asm!(
            "
            mov rbp, {0}
            mov rsp, {1}
            jmp {2}
        ", in(reg) rsp, in(reg) rbp, in(reg) rip, in("rdi") info, in("rsi") user_rsp,
        in("rdx") user_rbp, in("rcx") user_stored_ip
        );
    }
}
