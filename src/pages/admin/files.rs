// use super::AdminNavBar;
// use crate::components::navbar::NavBar;
use leptos::{prelude::*, web_sys::{FormData, HtmlFormElement, SubmitEvent}, wasm_bindgen::JsCast};
use leptos::server::codee::string::FromToStringCodec;
use leptos_use::{use_event_source_with_options, UseEventSourceOptions, UseEventSourceReturn};

use crate::server::{admin::{AdminUploadFile, upload_file}, structs::ApiResult};

/// Default Home Page
#[component]
pub fn Files() -> impl IntoView {
    let upload_action = Action::new_local(|data: &FormData| {
        // `MultipartData` implements `From<FormData>`
        upload_file(data.clone().into())
    });

    view! {
        <form on:submit=move |ev: SubmitEvent| {
            ev.prevent_default();
            let target = ev.target().unwrap().unchecked_into::<HtmlFormElement>();
            let form_data = FormData::new_with_form(&target).unwrap();
            upload_action.dispatch_local(form_data);
        }>
            <input type="file" name="file" />
            <input type="submit" />
        </form>
        <p>
            {move || {
                if upload_action.input().read().is_none() && upload_action.value().read().is_none()
                {
                    "Upload a file.".to_string()
                } else if upload_action.pending().get() {
                    "Uploading...".to_string()
                } else if let Some(Ok(value)) = upload_action.value().get() {
                    let ApiResult { result, details } = value;
                    details
                } else {
                    format!("{:?}", upload_action.value().get())
                }
            }}

        </p>
    }
}
