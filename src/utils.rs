use chrono::{DateTime, FixedOffset, Local, NaiveDateTime, TimeZone, Utc};
use std::error::Error;
use time::{Duration, OffsetDateTime, UtcOffset};

pub fn offset_to_naive(offset_dt: OffsetDateTime) -> NaiveDateTime {
    let offset_dt_secs = offset_dt.unix_timestamp();
    let offset_dt_nsecs = offset_dt.nanosecond();

    DateTime::from_timestamp(offset_dt_secs, offset_dt_nsecs).unwrap_or_default().naive_local()
}

pub fn offset_to_datetime(offset_dt: OffsetDateTime) -> DateTime<Local> {
    let offset_dt_secs = offset_dt.unix_timestamp();
    let offset_dt_nsecs = offset_dt.nanosecond();

    DateTime::from_timestamp(offset_dt_secs, offset_dt_nsecs).unwrap_or_default().naive_local().and_local_timezone(Local).unwrap()
}

pub fn html_local_to_datetime(dt: String) -> DateTime<Local> {
    let formatted_dt = format!("{dt}:00Z");
    match dateparser::parse_with_timezone(&dt, &chrono::offset::Local) {
        Ok(dt) => dt.into(),
        Err(_) => dateparser::parse_with_timezone(&formatted_dt, &chrono::offset::Local).unwrap_or(Utc::now()).into()
    }
}
