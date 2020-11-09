use core::{
    sync::atomic::AtomicBool,
    sync::atomic::Ordering,
    task::{Context, Poll, Waker},
};

use alloc::{sync::Arc, task::Wake};
use futures_lite::{pin, Future};

static SHOULD_WAKE: AtomicBool = AtomicBool::new(false);

pub fn block_on<T>(task: impl Future<Output = T>) {
    let waker = SchedulerWaker::new();
    pin!(task);
    loop {
        let mut context = Context::from_waker(&waker);
        match task.as_mut().poll(&mut context) {
            Poll::Ready(_) => {
                return;
            }
            Poll::Pending => {}
        };

        x86_64::instructions::interrupts::disable();
        if SHOULD_WAKE.swap(false, Ordering::SeqCst) {
            x86_64::instructions::interrupts::enable();
            continue;
        } else {
            info!("Sleeping");
            x86_64::instructions::interrupts::enable_interrupts_and_hlt();
        }
    }
}

struct SchedulerWaker {}

impl SchedulerWaker {
    pub fn new() -> Waker {
        Waker::from(Arc::new(SchedulerWaker {}))
    }
}

impl Wake for SchedulerWaker {
    fn wake(self: Arc<Self>) {
        SHOULD_WAKE.store(true, Ordering::SeqCst);
    }

    fn wake_by_ref(self: &Arc<Self>) {
        SHOULD_WAKE.store(true, Ordering::SeqCst);
    }
}
