use chrono::{DateTime, Duration, Utc};
use conquer_once::spin::OnceCell;

pub(crate) static TIME_PROVIDER: OnceCell<fn() -> DateTime<Utc>> = OnceCell::uninit();
pub(crate) static UPTIME_PROVIDER: OnceCell<fn() -> Duration> = OnceCell::uninit();

pub fn get_current_time() -> DateTime<Utc> {
    TIME_PROVIDER.get().unwrap()()
}

pub fn get_uptime() -> Duration {
    UPTIME_PROVIDER.get().unwrap()()
}
