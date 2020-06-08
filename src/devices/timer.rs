use crate::*;
use alloc::{collections::BTreeMap, sync::Arc};
use conquer_once::spin::OnceCell;
use core::{
    convert::TryFrom,
    future::Future,
    pin::Pin,
    task::{Context, Poll, Waker},
    time::Duration,
};
use crossbeam_queue::ArrayQueue;
use futures_util::{stream::Stream, task::AtomicWaker, StreamExt};
use spin::Mutex;

/// A future that can be used to pause a thread.
pub struct TimerFuture {
    /// A state that is shared between the calling thread and the target
    /// timer thread is responsible for waking up the calling thread.
    shared_state: Arc<Mutex<TimerSharedState>>,
}

/// Shared state between the future and the waiting thread
struct TimerSharedState {
    /// Whether or not the sleep time has elapsed
    completed: bool,

    /// The waker for the task that `TimerFuture` is running on.
    /// The thread can use this after setting `completed = true` to tell
    /// `TimerFuture`'s task to wake up, see that `completed = true`, and
    /// move forward.
    waker: Option<Waker>,
}

impl Future for TimerFuture {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Look at the shared state to see if the timer has already completed.
        let mut shared_state = self.shared_state.lock();
        if shared_state.completed {
            Poll::Ready(())
        } else {
            // Set waker so that the thread can wake up the current task
            // when the timer has completed, ensuring that the future is polled
            // again and sees that `completed = true`.
            //
            // It's tempting to do this once rather than repeatedly cloning
            // the waker each time. However, the `TimerFuture` can move between
            // tasks on the executor, which could cause a stale waker pointing
            // to the wrong task, preventing `TimerFuture` from waking up
            // correctly.
            //
            // N.B. it's possible to check for this using the `Waker::will_wake`
            // function, but we omit that here to keep things simple.
            shared_state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

impl TimerFuture {
    /// Create a new `TimerFuture` which will complete after the provided
    /// timeout.
    pub fn new(duration: Duration) -> Self {
        let shared_state = Arc::new(Mutex::new(TimerSharedState {
            completed: false,
            waker: None,
        }));

        // Convert the given time into nano seconds.
        // HPET Currently only supports u64 ticks as max value.
        let nanos = u64::try_from(duration.as_nanos())
            .expect("Cannot support more nanoseconds than u64 max");

        let current_value = TIME_PROVIDER.get().unwrap()();

        // Note this can overflow.
        let target_value = current_value + nanos;

        // Store this task in a btree.
        let mut waiting_tasks = WAITING_TASKS.lock();
        waiting_tasks.insert(target_value, shared_state.clone());

        // Schedule a wake up if needed
        TIMER_REGISTRAR
            .get()
            .expect("Time Provider should only be initialized once")(
            target_value,
            ALLOWED_TIMER_SKEW,
        );

        TimerFuture { shared_state }
    }
}

/// List of tasks that are waiting to be invoked.
/// This is a sorted list with key being the nano second timer.
static WAITING_TASKS: Mutex<BTreeMap<u64, Arc<Mutex<TimerSharedState>>>> =
    Mutex::new(BTreeMap::new());

static START_COUNTER: OnceCell<u64> = OnceCell::uninit();

/// Function to provide the current timer in ns.
static TIME_PROVIDER: OnceCell<fn() -> u64> = OnceCell::uninit();

/// Function to allow invoking of timer at a target counter value and skew.
/// Calling this function with larger values will have no effect.
static TIMER_REGISTRAR: OnceCell<fn(target_time: u64, timer_skew: u64) -> ()> = OnceCell::uninit();

// NOTE: ALL NUMBER ARE IN NANO SECONDS
const ALLOWED_TIMER_SKEW: u64 = 10_000; // 10 us

/// Function to be called by a TIMER Provider. (HPET, etc)
/// Function to provide the current timer in ns.
pub(crate) fn set_timer_provider(provider: fn() -> u64) {
    START_COUNTER.init_once(|| provider());
    TIME_PROVIDER
        .try_init_once(|| provider)
        .expect("Time Provider should only be initialized once");
}

/// Function to be called by a TIMER Registrar. (HPET, etc)
/// Function to allow invoking of timer at a target counter value and skew.
/// Calling this function with larger values will have no effect.
pub(crate) fn set_timer_registrar(provider: fn(target_time: u64, timer_skew: u64) -> ()) {
    TIMER_REGISTRAR
        .try_init_once(|| provider)
        .expect("Time Provider should only be initialized once");
}

/// Task that acts as the timer handler. This function handles invoking of tasks that use
/// timers. This function should be called before using any timers used in code.
pub async fn timer_task() {
    let mut timer_queue = TimerStream::new();

    while let Some(current_time) = timer_queue.next().await {
        let mut task_list = WAITING_TASKS.lock();
        loop {
            if let Some(first_task) = task_list.first_entry() {
                if *first_task.key() <= current_time + ALLOWED_TIMER_SKEW {
                    // WAKE THIS TASK
                    let task = first_task.remove();
                    let mut state = task.lock();
                    state.completed = true;
                    if let Some(waker) = state.waker.take() {
                        waker.wake()
                    }
                } else {
                    // All tasks are in the future.
                    // Set next wake up and
                    // Go back to sleep.
                    TIMER_REGISTRAR
                        .get()
                        .expect("Time Provider should only be initialized once")(
                        *first_task.key(),
                        ALLOWED_TIMER_SKEW,
                    );
                    break;
                }
            } else {
                break;
            }
        }
    }
}

/////////////////////////
// TIMER INTERRUPT QUEUE
/////////////////////////

/// Queue to be used to raise interrupts from timer interrupts.
static INTERRUPT_QUEUE: OnceCell<ArrayQueue<u64>> = OnceCell::uninit();
static TIMER_WAKER: AtomicWaker = AtomicWaker::new();

/// Called by the timer interrupt handler
///
/// Must not block or allocate.
pub(crate) fn add_notification() {
    let value = TIME_PROVIDER.get().expect("Cannot find time provider")();
    if let Ok(queue) = INTERRUPT_QUEUE.try_get() {
        if let Err(_) = queue.push(value) {
            kernel_info!("WARNING: timer interrupt queue full; Ignoring notification.");
        } else {
            TIMER_WAKER.wake();
        }
    } else {
        kernel_warn!("WARNING: timer interrupt queue uninitialized");
    }
}

struct TimerStream {
    _private: (),
}

impl TimerStream {
    pub fn new() -> Self {
        INTERRUPT_QUEUE
            .try_init_once(|| ArrayQueue::new(1000))
            .expect("TimerStream::new should only be called once");
        TimerStream { _private: () }
    }
}

impl Stream for TimerStream {
    type Item = u64;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u64>> {
        let queue = INTERRUPT_QUEUE
            .try_get()
            .expect("TimerStream queue not initialized");

        // fast path
        if let Ok(current_time) = queue.pop() {
            return Poll::Ready(Some(current_time));
        }

        TIMER_WAKER.register(&cx.waker());
        match queue.pop() {
            Ok(current_time) => {
                TIMER_WAKER.take();
                Poll::Ready(Some(current_time))
            }
            Err(crossbeam_queue::PopError) => Poll::Pending,
        }
    }
}

pub fn up_time() -> Duration {
    let current_value = TIME_PROVIDER.get().unwrap()();
    let diff = current_value - START_COUNTER.get().unwrap();
    let secs = diff / 1000_000_000;
    let nanos = (secs % 1000_000_000) as u32;
    Duration::new(secs, nanos)
}
