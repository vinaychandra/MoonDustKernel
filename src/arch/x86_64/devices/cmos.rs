use chrono::{DateTime, NaiveDate, Utc};
use cmos::{CMOSCenturyHandler, CMOS};

fn get_current_time() -> DateTime<Utc> {
    let mut cmos = unsafe { CMOS::new() };
    let rtc = cmos.read_rtc(CMOSCenturyHandler::CenturyRegister(0xA5));
    let ndt = NaiveDate::from_ymd(rtc.year as i32 + 2000, rtc.month as u32, rtc.day as u32)
        .and_hms(rtc.hour as u32, rtc.minute as u32, rtc.second as u32);
    DateTime::from_utc(ndt, Utc)
}

pub fn init_cmos() {
    crate::common::time::TIME_PROVIDER.init_once(|| get_current_time);
}
