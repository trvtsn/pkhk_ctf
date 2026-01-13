use crate::server::{db::structs::AttachmentWithoutBlob, enums::ResultStatus, structs::ApiResult};
use leptos::{prelude::*, task::spawn_local};

#[component]
pub fn File(
    file: crate::server::db::structs::AttachmentWithoutBlob,
    refresh: RwSignal<i32>
) -> impl IntoView {
    let AttachmentWithoutBlob { 
        id, 
        challenge_id: _, 
        event_id: _, 
        user_id: _, 
        file_name, 
        file_type, 
        mime_type, 
        file_size 
    } = file;
    
    let file_url_path = RwSignal::<String>::new("".to_string());
    let deleting = RwSignal::new(false);
    let delete_submit_btn_text = Memo::new(move |_| {
        if deleting.get() { "Confirm Delete".to_string() } else { "Delete".to_string() }
    });

    view! {
        <div class="bg-yale-blue-50 hover:bg-yale-blue-100 rounded-lg p-4 m-4 content-center">
            <h3 class="text-3xl/8 font-bold">{file_name.clone()}</h3>
            <p class="text-lg/8"><b>"ID: "</b> {id.clone()}</p>
            <p class="text-lg/8"><b>"MIME Type: "</b> {mime_type.unwrap_or_default()}</p>
            <p class="text-lg/8"><b>"File Type: "</b> {file_type.to_string()}</p>
            <p class="text-lg/8"><b>"File Size: "</b> {(file_size.unwrap_or_default() as f64) / 1000000_f64} " MB"</p>
            {file_url_path.set(format!("/file/{}", id))}
            <a download href=move || file_url_path.get() class="underline text-blue-600">{file_name}</a>

            <button
                class="ml-auto inline-flex items-center px-4 py-2 rounded-md bg-red-600 text-white text-sm font-semibold shadow-sm hover:bg-red-500 focus:outline-none focus:ring-2 focus:ring-indigo-500"
                on:click=move |_| {
                    let id = id.clone();
                    if deleting.get() {
                        spawn_local(async move {
                            tracing::debug!("deleting user: {id}");
                            if let Ok(ApiResult { result, .. }) = crate::server::admin::delete_file(id).await && result == ResultStatus::Success {
                                refresh.update(|n| *n += 1);
                            }
                        });
                        deleting.set(false);
                    } else {
                        deleting.set(true);
                    }
                }
            >{move || delete_submit_btn_text.get()}</button>
        </div>
    }
}
