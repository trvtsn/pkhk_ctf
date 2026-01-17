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
        upload_files(data.clone().into())
    });

    let upload_action_text = Memo::new(move |_| {
        if upload_action.pending().get() { "Uploading..." } else { "" }
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
            <input class="p-2 bg-white rounded-lg shadow-sm" type="file" name="files" multiple />
            <input
                class="inline-flex items-center py-2 px-4 ml-auto text-sm font-semibold text-white rounded-md shadow-sm focus:ring-2 focus:ring-yale-blue-500 focus:outline-none bg-yale-blue-600 hover:bg-yale-blue-500"
                type="submit"
                value="Upload"
            />
        </form>
        <p>{move || upload_action_text.get()}</p>
        <div class="grid grid-cols-4 m-2 files">
            <For
                each=move || all_files.get().clone().unwrap_or_default()
                key=|file: &db::structs::AttachmentWithoutBlob| file.id.clone()
                let(file)
            >
                <File file refresh />
            </For>
        </div>
    }
}
