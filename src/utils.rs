/// src/utils.rs
///
/// This file contains code for reusable abstractions which don't really belong anywhere else.
/// They can be used in both server and client code.

use crate::{components::toast::{ToastMessageType, push_new_toast}, error_template::AppError};
use std::any::type_name;
use chrono::{DateTime, Local, NaiveDateTime, ParseError, TimeZone, Utc, offset::LocalResult};
use leptos::{prelude::*, web_sys::FormData, wasm_bindgen::JsCast, web_sys::{HtmlInputElement, HtmlOptionElement, HtmlSelectElement}};
use time::OffsetDateTime;
use tracing::instrument;

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

#[instrument]
pub fn local_string_to_datetime(dt: String) -> Result<DateTime<Local>, ParseError> {
    let dt = DateTime::parse_from_rfc3339(&dt)?;
    let dt_local = dt.with_timezone(&Local);
    Ok(dt_local)
}

pub fn get_context<T: 'static + Clone>() -> Result<T, AppError> {
    let type_name_short = type_name::<T>().rsplit("::").next().unwrap_or(type_name::<T>());
    match use_context::<T>() {
        Some(val) => Ok(val),
        None => Err(AppError::InternalError(format!("Failed to extract {type_name_short} from context")))
    }
}

pub fn csv_contains(csv: &str, value: &str) -> bool {
    csv.split(',').any(|s| s == value)
}

pub fn collect_selected_options(select: &HtmlSelectElement) -> Vec<String> {
    let selected = select.selected_options();
    let mut picked = Vec::new();
    for i in 0..selected.length() {
        if let Some(item) = selected.item(i) {
            if let Ok(opt) = item.dyn_into::<HtmlOptionElement>() {
                picked.push(opt.value());
            }
        }
    }
    picked
}

pub fn build_single_file_form_data(node_ref: Option<HtmlInputElement>) -> Option<FormData> {
    let el = node_ref?;
    let files = el.files()?;
    if files.length() == 0 { return None; }
    let file = match files.get(0) {
        Some(f) => f,
        None => { push_new_toast(ToastMessageType::ErrorOccurred); return None; }
    };
    let fd = match FormData::new() {
        Ok(fd) => fd,
        Err(_) => { push_new_toast(ToastMessageType::ErrorOccurred); return None; }
    };
    match fd.append_with_blob_and_filename("file", &file, &file.name()) {
        Ok(_) => {},
        Err(_) => { push_new_toast(ToastMessageType::ErrorOccurred); return None; }
    }
    Some(fd)
}

pub fn action_btn_text(
    signal: impl Fn() -> bool + Send + Sync + 'static,
    active_text: &'static str,
    inactive_text: &'static str,
) -> Memo<String> {
    Memo::new(move |_| {
        if signal() { active_text.to_string() } else { inactive_text.to_string() }
    })
}

pub fn build_multi_file_form_data(node_ref: Option<HtmlInputElement>) -> Option<FormData> {
    let el = node_ref?;
    let files = el.files()?;
    if files.length() == 0 { return None; }
    let fd = match FormData::new() {
        Ok(fd) => fd,
        Err(_) => { push_new_toast(ToastMessageType::ErrorOccurred); return None; }
    };
    for i in 0..files.length() {
        let file = match files.get(i) {
            Some(f) => f,
            None => { push_new_toast(ToastMessageType::ErrorOccurred); return None; }
        };
        match fd.append_with_blob_and_filename("file", &file, &file.name()) {
            Ok(_) => {},
            Err(_) => { push_new_toast(ToastMessageType::ErrorOccurred); return None; }
        }
    }
    Some(fd)
}

pub fn format_duration(seconds: u64) -> String {
    let d = seconds / 86400;
    let h = (seconds % 86400) / 3600;
    let m = (seconds % 3600) / 60;
    let s = seconds % 60;
    if d > 0 {
        format!("{d}d {h}h {m}m {s}s")
    } else if h > 0 {
        format!("{h}h {m}m {s}s")
    } else if m > 0 {
        format!("{m}m {s}s")
    } else {
        format!("{s}s")
    }
}

pub fn format_traffic(bytes: u64) -> String {
    const KIB: f64 = 1024.0;
    const MIB: f64 = KIB * 1024.0;
    const GIB: f64 = MIB * 1024.0;
    let b = bytes as f64;
    if b >= GIB {
        format!("{:.2} GiB", b / GIB)
    } else if b >= MIB {
        format!("{:.2} MiB", b / MIB)
    } else if b >= KIB {
        format!("{:.2} KiB", b / KIB)
    } else {
        format!("{} B", bytes)
    }
}

pub fn format_file_size(bytes: u64) -> String {
    const KB: f64 = 1000.0;
    const MB: f64 = KB * 1000.0;
    const GB: f64 = MB * 1000.0;
    let b = bytes as f64;
    if b >= GB {
        format!("{:.2} GB", b / GB)
    } else if b >= MB {
        format!("{:.2} MB", b / MB)
    } else if b >= KB {
        format!("{:.2} KB", b / KB)
    } else {
        format!("{} B", bytes)
    }
}
