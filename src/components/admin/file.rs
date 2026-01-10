use leptos::prelude::*;

#[component]
pub fn File(
    file: crate::server::db::structs::AttachmentWithoutBlob
) -> impl IntoView {
    let attachment_filename = RwSignal::<String>::new("".to_string());
    view! {
        <div class="bg-yale-blue-50 hover:bg-yale-blue-100 rounded-lg p-4 m-4 content-center">
            <p class="text-lg/8"><b>"ID: "</b> {file.id.clone()}</p>
            <p class="text-lg/8"><b>"File Name: "</b> {file.file_name.clone()}</p>
            <p class="text-lg/8"><b>"MIME Type: "</b> {file.mime_type.unwrap_or_default()}</p>
            <p class="text-lg/8"><b>"File Type: "</b> {file.file_type.to_string()}</p>
            <p class="text-lg/8"><b>"File Size: "</b> {(file.file_size.unwrap_or_default() as f64) / 1000000_f64} " MB"</p>
            {attachment_filename.set(format!("/file/{}", file.id))}
            <a download href=move || attachment_filename.get() class="underline text-blue-600">{file.file_name}</a>
        </div>
    }
}
