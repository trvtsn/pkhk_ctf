// use super::AdminNavBar;
// use crate::components::navbar::NavBar;
use leptos::{prelude::*, web_sys::{FormData, HtmlFormElement, SubmitEvent}, wasm_bindgen::JsCast};
use leptos::server::codee::string::FromToStringCodec;
use leptos_use::{use_event_source_with_options, UseEventSourceOptions, UseEventSourceReturn};

use crate::{components::admin::file::File, server::{admin::{AdminUploadFile, get_all_files, upload_file}, db, structs::ApiResult}};

/// Default Home Page
#[component]
pub fn Files() -> impl IntoView {
    let all_files = Resource::new(move || (), move |_| async move {
        get_all_files().await.unwrap_or_default()
    });

    let upload_action = Action::new_local(|data: &FormData| {
        // `MultipartData` implements `From<FormData>`
        upload_file(data.clone().into())
    });

    let attachment_filename = RwSignal::<String>::new("".to_string());

    view! {
        <form on:submit=move |ev: SubmitEvent| {
            ev.prevent_default();
            let target = ev.target().unwrap().unchecked_into::<HtmlFormElement>();
            let form_data = FormData::new_with_form(&target).unwrap();
            upload_action.dispatch_local(form_data);
        }>
            <input class="bg-white border" type="file" name="file" />
            <input class="bg-white border" type="submit" value="Upload" />
        </form>
        <p>
            {move || {
                if upload_action.pending().get() {
                    "Uploading...".to_string()
                } else if let Some(Ok(value)) = upload_action.value().get() {
                    let ApiResult { result, details } = value;
                    details
                } else {
                    "".to_string()
                }
            }}

        </p>
        <div class="files m-2">
            <For
                each=move || all_files.get().clone().unwrap_or_default()
                key=|file: &db::structs::AttachmentWithoutBlob| file.id
                let(file)
            >
                <File file />
            </For>
        </div>
    }
}
