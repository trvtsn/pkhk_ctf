use crate::{components::admin::file::File, server::{admin::{get_all_files, upload_files}, db}};
use leptos::{prelude::*, web_sys::{FormData, HtmlFormElement, SubmitEvent}, wasm_bindgen::JsCast};

/// Default Home Page
#[component]
pub fn Files() -> impl IntoView {
    let refresh = RwSignal::new(0);
    let all_files = Resource::new(move || refresh.get(), move |_| async move {
        get_all_files().await.unwrap_or_default()
    });

    let upload_action = Action::new_local(|data: &FormData| {
        // `MultipartData` implements `From<FormData>`
        upload_files(data.clone().into())
    });

    Effect::new(move |_| {
        if let Some(Ok(_)) = upload_action.value().get() {
            all_files.refetch();
        }
    });

    view! {
        <form on:submit=move |ev: SubmitEvent| {
            ev.prevent_default();
            let target = ev.target().unwrap().unchecked_into::<HtmlFormElement>();
            let form_data = FormData::new_with_form(&target).unwrap();
            upload_action.dispatch_local(form_data);
        }>
            <label for="files">"Upload files..."</label>
            <input class="bg-white shadow-sm rounded-lg p-2" type="file" name="files" multiple />
            <input class="ml-auto inline-flex items-center px-4 py-2 rounded-md bg-indigo-600 text-white text-sm font-semibold shadow-sm hover:bg-indigo-500 focus:outline-none focus:ring-2 focus:ring-indigo-500" type="submit" value="Upload" />
        </form>
        <p>
            {move || {
                if upload_action.pending().get() {
                    "Uploading...".to_string()
                } else {
                    "".to_string()
                }
            }}

        </p>
        <div class="files m-2 grid grid-cols-4">
            <For
                each=move || all_files.get().clone().unwrap_or_default()
                key=|file: &db::structs::AttachmentWithoutBlob| file.id.clone()
                let(file)
            >
                <File file refresh/>
            </For>
        </div>
    }
}
