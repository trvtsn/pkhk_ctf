use chrono::{DateTime, FixedOffset, NaiveDateTime};
use std::error::Error;
use time::{Duration, OffsetDateTime, UtcOffset};

pub fn offset_to_naive(offset_dt: OffsetDateTime) -> NaiveDateTime {
    let offset_dt_secs = offset_dt.unix_timestamp();
    let offset_dt_nsecs = offset_dt.nanosecond();

    DateTime::from_timestamp(offset_dt_secs, offset_dt_nsecs).unwrap_or_default().naive_local()
}
