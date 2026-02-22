use crate::server::{db::structs::AttachmentWithoutBlob, enums::ResultStatus, structs::ApiResult};
use icondata as i;
use leptos::{prelude::*, task::spawn_local};
use leptos_icons::Icon;

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

    let file_name_signal = RwSignal::new(file_name.clone());
    let new_file_name = RwSignal::new(file_name);
    let file_url_path = Memo::new(
        { let id = id.clone(); move |_| { format!("/file/{}", id) } }
    );
    let deleting = RwSignal::new(false);
    let renaming = RwSignal::new(false);
    let rename_submit_btn_text = Memo::new(move |_| {
        if deleting.get() { "Confirm Rename".to_string() } else { "Rename".to_string() }
    });
    let delete_submit_btn_text = Memo::new(move |_| {
        if deleting.get() { "Confirm Delete".to_string() } else { "Delete".to_string() }
    });

    view! {
        <div class=r#"grid content-center p-4 m-4 rounded-lg bg-card hover:bg-card-hover break-all"#>
            <h3 class=r#"font-bold text-3xl/8"#>{move || file_name_signal.get()}</h3>
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
            <div class="flex gap-2 mt-2">
                <a download href=move || file_url_path.get() class=r#""#>
                    <Icon icon=i::LuDownload />
                </a>
            </div>

            <Show when=move || renaming.get()>
                <div class="mt-2">
                    <label class=r#"block mb-1 text-sm font-medium"#>"New File Name"</label>
                    <input
                        class=r#"bg-background py-2 px-3 w-full text-sm rounded-md border border-input-border 
                        focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                        name="name"
                        value=move || new_file_name.get()
                        bind:value=new_file_name
                    />
                </div>
            </Show>
            
            <div class=r#"flex flex-row-reverse gap-3 mt-2"#>
                <Show when=move || renaming.get() || deleting.get()>
                    <button
                        class=r#"py-2 px-4 text-sm rounded-md border border-input-border hover:bg-background-hover"#
                        on:click=move |_| {
                            renaming.set(false);
                            deleting.set(false);
                        }
                    >
                        "Cancel"
                    </button>
                </Show>
                <button
                    hidden=move || deleting.get()
                    class=r#"inline-flex gap-2 items-center py-2 px-4 text-sm font-medium text-white 
                    rounded-lg transition focus:ring-2 focus:outline-none active:scale-95 
                    bg-yale-blue-600 hover:bg-yale-blue-700 focus:ring-yale-blue-400"#
                    on:click={
                        let value = id.clone();
                        move |_| {
                            let id = value.clone();
                            let new_file_name = new_file_name.get();
                            if renaming.get() {
                                spawn_local(async move {
                                    tracing::debug!("renaming file: {id}");
                                    if let Ok(ApiResult { result, .. }) = crate::server::admin::rename_file(
                                            id,
                                            new_file_name.clone()
                                        )
                                        .await && result == ResultStatus::Success
                                    {
                                        file_name_signal.set(new_file_name);
                                        refresh.update(|n| *n += 1);
                                    }
                                });
                                renaming.set(false);
                            } else {
                                renaming.set(true);
                            }
                        }
                    }
                >
                    {move || rename_submit_btn_text.get()}
                </button>
                
                <button
                    hidden=move || renaming.get()
                    class=r#"inline-flex items-center py-2 px-4 ml-auto text-sm font-semibold text-white 
                    bg-red-600 rounded-md shadow-sm hover:bg-red-500 focus:ring-2 focus:outline-none 
                    focus:ring-yale-blue-500"#
                    on:click=move |_| {
                        let id = id.clone();
                        if deleting.get() {
                            spawn_local(async move {
                                tracing::debug!("deleting file: {id}");
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
        </div>
    }
}
