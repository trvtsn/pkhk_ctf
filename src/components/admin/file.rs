use leptos::prelude::*;

use crate::server::db::structs::DbUser;
// use thaw::*;

#[component]
pub fn File(
    file: crate::server::db::structs::AttachmentWithoutBlob
) -> impl IntoView {
    let attachment_filename = RwSignal::<String>::new("".to_string());
    view! {
        <div class="bg-yale-blue-50 hover:bg-yale-blue-100 rounded-2xl p-4 m-4 content-center">
            <p>"ID: " {file.id}</p>
            <p>"File Name: " {file.file_name.clone()}</p>
            <p>"MIME Type: " {file.mime_type.unwrap_or_default()}</p>
            <p>"File Type: " {file.file_type.to_string()}</p>
            <p>"File Size: " {(file.file_size.unwrap_or_default() as f64) / 1000000_f64} " MB"</p>
            {attachment_filename.set(format!("/file/{}/{}", file.id, file.file_name))}
            <a download href=move || attachment_filename.get() class="underline text-blue-600">{file.file_name}</a>
        </div>
    }
}
