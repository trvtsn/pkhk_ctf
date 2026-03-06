use std::any::type_name;

use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc, offset::LocalResult};
use leptos::prelude::use_context;
use time::OffsetDateTime;

use crate::error_template::AppError;

pub fn offset_to_naive(offset_dt: OffsetDateTime) -> NaiveDateTime {
    let offset_dt_secs = offset_dt.unix_timestamp();
    let offset_dt_nsecs = offset_dt.nanosecond();

    DateTime::from_timestamp(offset_dt_secs, offset_dt_nsecs).unwrap_or_default().naive_local()
}

pub fn offset_to_datetime(offset_dt: OffsetDateTime) -> DateTime<Local> {
    let offset_dt_secs = offset_dt.unix_timestamp();
    let offset_dt_nsecs = offset_dt.nanosecond();

    match Local.timestamp_opt(offset_dt_secs, offset_dt_nsecs) {
        LocalResult::Single(dt) => dt,
        LocalResult::Ambiguous(dt_earlier, _dt_later) => dt_earlier,
        LocalResult::None => {
            let utc_dt = Utc.timestamp_opt(offset_dt_secs, offset_dt_nsecs)
                .single()
                .unwrap_or_default();
            utc_dt.with_timezone(&Local)
        }
    }
}

pub fn html_local_to_datetime(dt: String) -> DateTime<Local> {
    let formatted_dt = format!("{dt}:00Z");
    match dateparser::parse_with_timezone(&dt, &chrono::offset::Local) {
        Ok(dt) => dt.into(),
        Err(_) => dateparser::parse_with_timezone(&formatted_dt, &chrono::offset::Local).unwrap_or(Utc::now()).into()
    }
}

pub fn get_context<T: 'static + Clone>() -> Result<T, AppError> {
    let type_name_short = type_name::<T>().rsplit("::").next().unwrap_or(type_name::<T>());
    match use_context::<T>() {
        Some(val) => Ok(val),
        None => Err(AppError::InternalError(format!("Failed to extract {type_name_short} from context")))
    }
}
