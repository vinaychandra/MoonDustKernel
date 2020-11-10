use alloc::boxed::Box;
use core::{
    pin::Pin,
    ptr,
    task::{Context, Poll},
};

use futures_lite::{Future, FutureExt};

use crate::common::memory::stack::Stack;

#[derive(Debug, Copy, Clone)]
enum ProcessState {
    NotRunning,
    Yielded,
    Complete(u8),
    Preempted(*const (), *const ()),        // RSP RBP
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
            trampoline_1()
        }
    }
}

#[thread_local]
static mut CUR_TASK: *mut Data = ptr::null_mut();

#[thread_local]
static mut CUR_CONTEXT: *mut Context = ptr::null_mut();

#[thread_local]
static mut TRAMPOLINE_1_RSP_RBP: (u64, u64) = (0, 0);

// Run on original stack
#[inline(never)]
unsafe fn trampoline_1() -> Poll<u8> {
    asm!("trampoline_1_j:"); // Used to skip the stack setting..
    {
        let rsp: u64;
        let rbp: u64;
        asm!("
                mov {0}, rsp
                mov {1}, rbp",
                 out(reg) rsp, out(reg) rbp);

        // Store the current stack.
        TRAMPOLINE_1_RSP_RBP = (rsp, rbp);
    }

    match (*CUR_TASK).state {
        ProcessState::NotRunning => {
            (*CUR_TASK).stack.switch_to();
            asm!("jmp {0}", sym trampoline_2);
        }
        ProcessState::Yielded => {
            (*CUR_TASK).state = ProcessState::NotRunning;
            return Poll::Pending;
        }
        ProcessState::Complete(v) => return Poll::Ready(v),
        ProcessState::Preempted(rsp, rbp) => {
            (*CUR_TASK).state = ProcessState::ResumePreemption(rsp, rbp);
            (*CUR_CONTEXT).waker().wake_by_ref();
            return Poll::Pending;
        }
        ProcessState::ResumePreemption(rsp, rbp) => asm!(
            "
                mov rsp, {0}
                mov rbp, {1}
                jmp preemptive_yield_j
            ",
            in(reg) rsp,
            in(reg) rbp
        ),
    }

    // Should never come here
    trampoline_2(); // Force compile t2
    preemptive_yield(); // Force compile preemptive yield
    Poll::Pending
}

// Run on final stack
#[inline(never)]
unsafe fn trampoline_2() {
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
            jmp trampoline_1_j", in(reg) rsp, in(reg) rbp
    );
}

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
        ", in(reg) rsp, in(reg) rbp
        );
    }
}
