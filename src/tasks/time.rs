use core::sync::atomic::AtomicU64;

use alloc::{collections::BTreeSet, sync::Arc};
use chrono::{DateTime, Duration, Utc};

use crate::{
    common::time::get_current_time_precise,
    sync::{mutex::Mutex, signal::Signal},
};

struct TimerInfo(DateTime<Utc>, Arc<Signal>, u64);
impl Ord for TimerInfo {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        let v = self.0.cmp(&other.0);
        if v == core::cmp::Ordering::Equal {
            return self.2.cmp(&other.2);
        }

        return v;
    }
}

impl PartialOrd for TimerInfo {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for TimerInfo {}

impl PartialEq for TimerInfo {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0) && self.2.eq(&other.2)
    }
}

static TIMERS_SIGNAL: Signal = Signal::new();
static TIMERS: Mutex<BTreeSet<TimerInfo>> = Mutex::new(BTreeSet::new());
static INSERTED_COUNT: AtomicU64 = AtomicU64::new(0);

crate async fn process_timer_tasks() -> u8 {
    info!(target:"time", "Timer tasks process started");
    let skew_allowed = Duration::microseconds(50);
    loop {
        TIMERS_SIGNAL.wait_async().await;
        let cur_time = get_current_time_precise();
        {
            let timers = &mut TIMERS.lock().await;
            loop {
                if let Some(first) = timers.first() {
                    if cur_time + skew_allowed > first.0 {
                        let first = timers.pop_first().unwrap();
                        first.1.signal();
                    } else {
                        // Timers are in the future
                        crate::arch::send_interrupt_in(first.0 - cur_time);
                        break;
                    }
                } else {
                    // No more timers. Stop iterating
                    break;
                }
            }
        }
    }
}

crate fn signal_timer() {
    TIMERS_SIGNAL.signal();
}

pub async fn delay_async(duration: Duration) {
    if duration.num_microseconds().unwrap() < 100 {
        return;
    }

    let signal = Arc::new(Signal::new());
    {
        let timers = &mut TIMERS.lock().await;
        let insert_val = INSERTED_COUNT.fetch_add(1, core::sync::atomic::Ordering::Relaxed);
        let cur_time = get_current_time_precise();
        timers.insert(TimerInfo(cur_time + duration, signal.clone(), insert_val));

        if timers.len() == 1 {
            crate::arch::send_interrupt_in(duration);
        }
    }
    signal.wait_async().await
}
