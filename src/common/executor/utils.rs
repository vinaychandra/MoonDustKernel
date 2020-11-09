//! Executors ported from https://github.com/smol-rs/async-executor/blob/master/src/lib.rs

use core::{
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    task::Poll,
    task::Waker,
};

use alloc::{sync::Arc, vec::Vec};
use async_task::Runnable;
use crossbeam_queue::SegQueue;
use futures_lite::future;
use spin::{Mutex, RwLock};

use crate::common::utils::arena::Arena;

/// The state of a executor.
#[derive(Debug)]
pub struct State {
    /// The global queue.
    pub queue: SegQueue<Runnable>,

    /// Local queues created by runners.
    local_queues: RwLock<Vec<Arc<SegQueue<Runnable>>>>,

    /// Set to `true` when a sleeping ticker is notified or no tickers are sleeping.
    notified: AtomicBool,

    /// A list of sleeping tickers.
    sleepers: Mutex<Sleepers>,

    /// Currently active tasks.
    pub active: Mutex<Arena<Waker>>,
}

impl State {
    /// Creates state for a new executor.
    pub fn new() -> State {
        State {
            queue: SegQueue::new(),
            local_queues: RwLock::new(Vec::new()),
            notified: AtomicBool::new(true),
            sleepers: Mutex::new(Sleepers {
                count: 0,
                wakers: Vec::new(),
                free_ids: Vec::new(),
            }),
            active: Mutex::new(Arena::new()),
        }
    }

    /// Notifies a sleeping ticker.
    #[inline]
    pub fn notify(&self) {
        if !self
            .notified
            .compare_and_swap(false, true, Ordering::SeqCst)
        {
            let waker = self.sleepers.lock().notify();
            if let Some(w) = waker {
                w.wake();
            }
        }
    }
}

/// A list of sleeping tickers.
#[derive(Debug)]
pub struct Sleepers {
    /// Number of sleeping tickers (both notified and unnotified).
    count: usize,

    /// IDs and wakers of sleeping unnotified tickers.
    ///
    /// A sleeping ticker is notified when its waker is missing from this list.
    wakers: Vec<(usize, Waker)>,

    /// Reclaimed IDs.
    free_ids: Vec<usize>,
}

impl Sleepers {
    /// Inserts a new sleeping ticker.
    fn insert(&mut self, waker: &Waker) -> usize {
        let id = match self.free_ids.pop() {
            Some(id) => id,
            None => self.count + 1,
        };
        self.count += 1;
        self.wakers.push((id, waker.clone()));
        id
    }

    /// Re-inserts a sleeping ticker's waker if it was notified.
    ///
    /// Returns `true` if the ticker was notified.
    fn update(&mut self, id: usize, waker: &Waker) -> bool {
        for item in &mut self.wakers {
            if item.0 == id {
                if !item.1.will_wake(waker) {
                    item.1 = waker.clone();
                }
                return false;
            }
        }

        self.wakers.push((id, waker.clone()));
        true
    }

    /// Removes a previously inserted sleeping ticker.
    ///
    /// Returns `true` if the ticker was notified.
    fn remove(&mut self, id: usize) -> bool {
        self.count -= 1;
        self.free_ids.push(id);

        for i in (0..self.wakers.len()).rev() {
            if self.wakers[i].0 == id {
                self.wakers.remove(i);
                return false;
            }
        }
        true
    }

    /// Returns `true` if a sleeping ticker is notified or no tickers are sleeping.
    fn is_notified(&self) -> bool {
        self.count == 0 || self.count > self.wakers.len()
    }

    /// Returns notification waker for a sleeping ticker.
    ///
    /// If a ticker was notified already or there are no tickers, `None` will be returned.
    fn notify(&mut self) -> Option<Waker> {
        if self.wakers.len() == self.count {
            self.wakers.pop().map(|item| item.1)
        } else {
            None
        }
    }
}

/// Runs task one by one.
#[derive(Debug)]
pub struct Ticker<'a> {
    /// The executor state.
    state: &'a State,

    /// Set to a non-zero sleeper ID when in sleeping state.
    ///
    /// States a ticker can be in:
    /// 1) Woken.
    /// 2a) Sleeping and unnotified.
    /// 2b) Sleeping and notified.
    sleeping: AtomicUsize,
}

impl Ticker<'_> {
    /// Creates a ticker.
    pub fn new(state: &State) -> Ticker<'_> {
        Ticker {
            state,
            sleeping: AtomicUsize::new(0),
        }
    }

    /// Moves the ticker into sleeping and unnotified state.
    ///
    /// Returns `false` if the ticker was already sleeping and unnotified.
    fn sleep(&self, waker: &Waker) -> bool {
        let mut sleepers = self.state.sleepers.lock();

        match self.sleeping.load(Ordering::SeqCst) {
            // Move to sleeping state.
            0 => self
                .sleeping
                .store(sleepers.insert(waker), Ordering::SeqCst),

            // Already sleeping, check if notified.
            id => {
                if !sleepers.update(id, waker) {
                    return false;
                }
            }
        }

        self.state
            .notified
            .swap(sleepers.is_notified(), Ordering::SeqCst);

        true
    }

    /// Moves the ticker into woken state.
    fn wake(&self) {
        let id = self.sleeping.swap(0, Ordering::SeqCst);
        if id != 0 {
            let mut sleepers = self.state.sleepers.lock();
            sleepers.remove(id);

            self.state
                .notified
                .swap(sleepers.is_notified(), Ordering::SeqCst);
        }
    }

    /// Waits for the next runnable task to run.
    pub async fn runnable(&self) -> Runnable {
        self.runnable_with(|| self.state.queue.pop()).await
    }

    /// Waits for the next runnable task to run, given a function that searches for a task.
    async fn runnable_with(&self, mut search: impl FnMut() -> Option<Runnable>) -> Runnable {
        future::poll_fn(|cx| {
            loop {
                match search() {
                    None => {
                        // Move to sleeping and unnotified state.
                        if !self.sleep(cx.waker()) {
                            // If already sleeping and unnotified, return.
                            return Poll::Pending;
                        }
                    }
                    Some(r) => {
                        // Wake up.
                        self.wake();

                        // Notify another ticker now to pick up where this ticker left off, just in
                        // case running the task takes a long time.
                        self.state.notify();

                        return Poll::Ready(r);
                    }
                }
            }
        })
        .await
    }
}

impl Drop for Ticker<'_> {
    fn drop(&mut self) {
        // If this ticker is in sleeping state, it must be removed from the sleepers list.
        let id = self.sleeping.swap(0, Ordering::SeqCst);
        if id != 0 {
            let mut sleepers = self.state.sleepers.lock();
            let notified = sleepers.remove(id);

            self.state
                .notified
                .swap(sleepers.is_notified(), Ordering::SeqCst);

            // If this ticker was notified, then notify another ticker.
            if notified {
                drop(sleepers);
                self.state.notify();
            }
        }
    }
}

/// A worker in a work-stealing executor.
///
/// This is just a ticker that also has an associated local queue for improved cache locality.
#[derive(Debug)]
pub struct Runner<'a> {
    /// The executor state.
    state: &'a State,

    /// Inner ticker.
    ticker: Ticker<'a>,

    /// The local queue.
    local: Arc<SegQueue<Runnable>>,

    /// Bumped every time a runnable task is found.
    ticks: AtomicUsize,
}

impl Runner<'_> {
    /// Creates a runner and registers it in the executor state.
    pub fn new(state: &State) -> Runner<'_> {
        let runner = Runner {
            state,
            ticker: Ticker::new(state),
            local: Arc::new(SegQueue::new()),
            ticks: AtomicUsize::new(0),
        };
        state.local_queues.write().push(runner.local.clone());
        runner
    }

    /// Waits for the next runnable task to run.
    pub async fn runnable(&self) -> Runnable {
        let runnable = self
            .ticker
            .runnable_with(|| {
                // Try the local queue.
                if let Some(r) = self.local.pop() {
                    return Some(r);
                }

                // Try stealing from the global queue.
                if let Some(r) = self.state.queue.pop() {
                    steal(&self.state.queue, &self.local);
                    return Some(r);
                }

                // Try stealing from other runners.
                let local_queues = self.state.local_queues.read();

                // Pick a random starting point in the iterator list and rotate the list.
                let n = local_queues.len();
                // TODO: Setup rand
                // let start = fastrand::usize(..n);
                let mut rng = <rand_chacha::ChaCha8Rng as rand::SeedableRng>::seed_from_u64(987);
                let start = rand::prelude::RngCore::next_u32(&mut rng) as usize % n;
                let iter = local_queues
                    .iter()
                    .chain(local_queues.iter())
                    .skip(start)
                    .take(n);

                // Remove this runner's local queue.
                let iter = iter.filter(|local| !Arc::ptr_eq(local, &self.local));

                // Try stealing from each local queue in the list.
                for local in iter {
                    steal(local, &self.local);
                    if let Some(r) = self.local.pop() {
                        return Some(r);
                    }
                }

                None
            })
            .await;

        // Bump the tick counter.
        let ticks = self.ticks.fetch_add(1, Ordering::SeqCst);

        if ticks % 64 == 0 {
            // Steal tasks from the global queue to ensure fair task scheduling.
            steal(&self.state.queue, &self.local);
        }

        runnable
    }
}

impl Drop for Runner<'_> {
    fn drop(&mut self) {
        // Remove the local queue.
        self.state
            .local_queues
            .write()
            .retain(|local| !Arc::ptr_eq(local, &self.local));

        // Re-schedule remaining tasks in the local queue.
        while let Some(r) = self.local.pop() {
            r.schedule();
        }
    }
}

/// Steals some items from one queue into another.
fn steal<T>(src: &SegQueue<T>, dest: &SegQueue<T>) {
    // Half of `src`'s length rounded up.
    let count = (src.len() + 1) / 2;

    if count > 0 {
        // Steal tasks.
        for _ in 0..count {
            if let Some(t) = src.pop() {
                dest.push(t);
            } else {
                break;
            }
        }
    }
}

/// Runs a closure when dropped.
pub struct CallOnDrop<F: Fn()>(pub F);

impl<F: Fn()> Drop for CallOnDrop<F> {
    fn drop(&mut self) {
        (self.0)();
    }
}
