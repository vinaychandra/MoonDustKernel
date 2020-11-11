use chrono::{DateTime, Duration, Utc};
use conquer_once::spin::OnceCell;

crate static TIME_PROVIDER: OnceCell<fn() -> DateTime<Utc>> = OnceCell::uninit();
crate static UPTIME_PROVIDER: OnceCell<fn() -> Duration> = OnceCell::uninit();

static SYNC_POINT: OnceCell<(DateTime<Utc>, Duration)> = OnceCell::uninit();

pub fn get_current_time() -> DateTime<Utc> {
    TIME_PROVIDER.get().unwrap()()
}

pub fn get_uptime() -> Duration {
    UPTIME_PROVIDER.get().unwrap()()
}

// Generally, uptime is a better clock. We can use that to retrieve precise moments.
pub fn get_current_time_precise() -> DateTime<Utc> {
    let a = SYNC_POINT.get_or_init(|| (get_current_time(), get_uptime()));
    let current_instant = get_uptime();

    a.0 - a.1 + current_instant
}
