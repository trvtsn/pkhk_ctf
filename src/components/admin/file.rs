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
        <div class=r#"grid content-center p-4 m-4 rounded-lg bg-card hover:bg-card-hover"#>
            <h3 class=r#"font-bold text-3xl/8"#>{file_name.clone()}</h3>
            <p class=r#"text-lg/8"#>
                <b>"ID: "</b>
                {id.clone()}
            </p>
            <p class=r#"text-lg/8"#>
                <b>"MIME Type: "</b>
                {mime_type.unwrap_or_default()}
            </p>
            <p class=r#"text-lg/8"#>
                <b>"File Type: "</b>
                {file_type.to_string()}
            </p>
            <p class=r#"text-lg/8"#>
                <b>"File Size: "</b>
                {(file_size.unwrap_or_default() as f64) / 1000000_f64}
                " MB"
            </p>
            {file_url_path.set(format!("/file/{}", id))}
            <a download href=move || file_url_path.get() class=r#"text-blue-600 underline"#>
                {file_name}
            </a>

            <button
                class=r#"inline-flex items-center py-2 px-4 ml-auto text-sm font-semibold text-white 
                bg-red-600 rounded-md shadow-sm hover:bg-red-500 focus:ring-2 focus:outline-none 
                focus:ring-yale-blue-500"#
                on:click=move |_| {
                    let id = id.clone();
                    if deleting.get() {
                        spawn_local(async move {
                            tracing::debug!("deleting user: {id}");
                            if let Ok(ApiResult { result, .. }) = crate::server::admin::delete_file(
                                    id,
                                )
                                .await && result == ResultStatus::Success
                            {
                                refresh.update(|n| *n += 1);
                            }
                        });
                        deleting.set(false);
                    } else {
                        deleting.set(true);
                    }
                }
            >
                {move || delete_submit_btn_text.get()}
            </button>
        </div>
    }
}
