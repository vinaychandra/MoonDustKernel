//! Rust future that can be stopped at any point during execution.
//! The kernel has two stacks. One per-cpu and one per-task. This future
//! jumps between the two stacks and stashes the per-task stack if preempted
//! so that it can resume later.

use alloc::boxed::Box;
use core::{
    pin::Pin,
    ptr,
    task::{Context, Poll},
};

use futures_lite::{Future, FutureExt};

use crate::common::memory::stack::Stack;

/// The State of the process.
#[derive(Debug, Copy, Clone)]
enum ProcessState {
    /// The process is not running. Signals that it should be started.
    NotRunning,

    /// The process has yielded. Signals to make it NotRunning and return to the executor.
    Yielded,

    /// The process is now done. Signals the executor that the process finished.
    Complete(u8),

    /// The process has been preempted. Signals that the stack should be noted down
    /// and returned to the executor.
    Preempted(*const (), *const ()), // RSP RBP

    /// Signals that a pre-empted process has to be resumed.
    ResumePreemption(*const (), *const ()), // RSP RBP
}

pub struct PreemptableFuture {
    data: Data,
}

unsafe impl Send for PreemptableFuture {}
unsafe impl Sync for PreemptableFuture {}

struct Data {
    /// The stack on which this future is running.
    stack: Stack,

    original_future: Pin<Box<dyn Future<Output = u8>>>,

    state: ProcessState,
}

impl PreemptableFuture {
    /// Create a new preemptable task that runs the provided future on the provided stack.
    pub fn new(entry_point: impl Future<Output = u8> + 'static, stack: Stack) -> PreemptableFuture {
        PreemptableFuture {
            data: Data {
                stack,
                original_future: Box::pin(entry_point),
                state: ProcessState::NotRunning,
            },
        }
    }
}

impl Future for PreemptableFuture {
    type Output = u8;

    fn poll(mut self: core::pin::Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        unsafe {
            CUR_TASK = &mut self.data as *mut Data;
            CUR_CONTEXT = cx as *mut Context<'_> as *const () as *mut Context<'static>;

            // Because T1 is called by T2.. Its registers might have more changes than expeected.
            asm!("call {}", sym trampoline_1,
                lateout("rax") _, lateout("rdi") _, lateout("rsi") _, lateout("rdx") _, lateout("rcx") _,
                lateout("r8") _, lateout("r9") _, lateout("r10") _, lateout("r11") _);
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

// Run on original stack
#[inline(never)]
unsafe extern "C" fn trampoline_1() {
    asm!("trampoline_1_j:"); // Used to skip the stack setting..
    {
        // Store the current stack info.
        let rsp: u64;
        let rbp: u64;
        asm!("
                mov {0}, rsp
                mov {1}, rbp",
                 out(reg) rsp, out(reg) rbp);
        TRAMPOLINE_1_RSP_RBP = (rsp, rbp);
    }

    match (*CUR_TASK).state {
        ProcessState::NotRunning => {
            (*CUR_TASK).stack.switch_to();
            // This function doesn't return.
            asm!("jmp {0}", sym trampoline_2)
        }
        ProcessState::Yielded => {
            // we need to switch to not running next time.
            (*CUR_TASK).state = ProcessState::NotRunning;
            TRAMPOLINE_1_RETURN = Poll::Pending;
            return;
        }
        ProcessState::Complete(v) => {
            // Process done
            TRAMPOLINE_1_RETURN = Poll::Ready(v);
            return;
        }
        ProcessState::Preempted(rsp, rbp) => {
            // Preempted this time. Mark for resumption, wake and return.
            (*CUR_TASK).state = ProcessState::ResumePreemption(rsp, rbp);
            (*CUR_CONTEXT).waker().wake_by_ref();
            TRAMPOLINE_1_RETURN = Poll::Pending;
            return;
        }
        ProcessState::ResumePreemption(rsp, rbp) => asm!(
            "
                mov rsp, {0}
                mov rbp, {1}
                jmp preemptive_yield_j
            ",
            in(reg) rsp,
            in(reg) rbp,
        ),
    }

    // Should never come here
    trampoline_2(); // Force compile t2
    preemptive_yield(); // Force compile preemptive yield
    panic!("Unexpected call");
}

/// Run on final stack. This only works with yielded tasks.
#[inline(never)]
unsafe extern "C" fn trampoline_2() {
    let fut = &mut *(CUR_TASK as *mut Data);
    let cx = &mut *CUR_CONTEXT;
    let result = match fut.state {
        ProcessState::NotRunning => fut.original_future.poll(cx),
        ProcessState::Yielded => {
            panic!("Trampoline2 cannot be called when process state is yielded")
        }
        ProcessState::Complete(v) => Poll::Ready(v),
        ProcessState::Preempted(_, _) => {
            panic!("Trampoline2 cannot be called when process state is preempted")
        }
        ProcessState::ResumePreemption(_, _) => {
            panic!("Trampoline2 cannot be called when process state is preempted (resumepre)")
        }
    };

    match result {
        Poll::Ready(return_value) => (*CUR_TASK).state = ProcessState::Complete(return_value),
        Poll::Pending => {
            (*CUR_TASK).state = ProcessState::Yielded;
        }
    }

    let (rsp, rbp) = TRAMPOLINE_1_RSP_RBP;
    asm!("
            mov rsp, {0}
            mov rbp, {1}
            jmp trampoline_1_j", in(reg) rsp, in(reg) rbp,
            lateout("rax") _, lateout("rdi") _, lateout("rsi") _, lateout("rdx") _, lateout("rcx") _,
            lateout("r8") _, lateout("r9") _, lateout("r10") _, lateout("r11") _,
    );
}

/// Preempt this future. Stores the stack and returns to the executor.
#[inline(never)]
pub fn preemptive_yield() {
    unsafe {
        let rsp: *const ();
        let rbp: *const ();
        asm!("
                mov {0}, rsp
                mov {1}, rbp",
                 out(reg) rsp, out(reg) rbp);
        (*CUR_TASK).state = ProcessState::Preempted(rsp, rbp);

        // We jump to trampoline_1 here
        let (rsp, rbp) = TRAMPOLINE_1_RSP_RBP;
        asm!("
            mov rsp, {0}
            mov rbp, {1}
            jmp trampoline_1_j

            preemptive_yield_j:
        ", in(reg) rsp, in(reg) rbp,
            lateout("rax") _, lateout("rdi") _, lateout("rsi") _, lateout("rdx") _, lateout("rcx") _,
            lateout("r8") _, lateout("r9") _, lateout("r10") _, lateout("r11") _,
        );
    }
}
