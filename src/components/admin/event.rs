use crate::{server::{admin::{upload_files, upload_illustration}, db::{self, enums::AttachmentIdentifier}, enums::ResultStatus, get_illustration_id, structs::ApiResult}, utils::html_local_to_datetime};
use chrono::DateTime;
use leptos::{prelude::*, task::spawn_local, wasm_bindgen::JsCast, web_sys::{Event, FormData, HtmlInputElement}};

#[component]
pub fn Event(event: db::structs::Event, refresh: RwSignal<i32>) -> impl IntoView {
    let id_signal = RwSignal::new(event.id.clone());
    let name_signal = RwSignal::new(event.name.clone());
    let description_signal = RwSignal::new(event.description.clone());
    let start_at_signal = RwSignal::new(event.start_at);
    let end_at_signal = RwSignal::new(event.end_at);

    let name_edit = RwSignal::new(event.name.clone());
    let description_edit = RwSignal::new(event.description.clone());
    let start_at_edit = RwSignal::new(event.start_at);
    let end_at_edit = RwSignal::new(event.end_at);
    let attachments_edit = RwSignal::new(None);
    let illustration_edit = RwSignal::new(None);

    let illustration = Resource::new(move || refresh.get(), move |_| {
        let event_id = id_signal.get();
        async move { get_illustration_id(AttachmentIdentifier::EventId(event_id)).await.unwrap_or_default() }
    });

    let file_upload_action = Action::new_local(|data: &FormData| {
        upload_files(data.clone().into())
    });

    let illustration_upload_action = Action::new_local(|data: &FormData| {
        upload_illustration(data.clone().into())
    });

    Effect::new(move |_| {
        if let Some(Ok(api_result)) = file_upload_action.value().get() {
            attachments_edit.set(Some(api_result.details.clone()));
        } else if let Some(Ok(api_result)) = illustration_upload_action.value().get() {
            illustration_edit.set(Some(api_result.details.clone()));
        }
    });

    let editing = RwSignal::new(false);
    let deleting = RwSignal::new(false);

    let delete_submit_btn_text = Memo::new(move |_| {
        if deleting.get() { "Confirm Delete".to_string() } else { "Delete".to_string() }
    });
    let edit_submit_btn_text = Memo::new(move |_| {
        if editing.get() { "Confirm Edit".to_string() } else { "Edit".to_string() }
    });

    let uploading_file_text = Memo::new(move |_| {
        if file_upload_action.pending().get() {
            "Uploading...".to_string()
        // } else if let Some(Ok(val)) = upload_action.value().get() {
        //     format!("Uploaded: {}", val.details.file_name)
        // } else {
        } else {
            "".to_string()
        }
    });

    let uploading_illustration_text = Memo::new(move |_| {
        if illustration_upload_action.pending().get() {
            "Uploading...".to_string()
        // } else if let Some(Ok(val)) = upload_action.value().get() {
        //     format!("Uploaded: {}", val.details.file_name)
        // } else {
        } else {
            "".to_string()
        }
    });

    view! {
        <div class=r#"content-center p-4 rounded-lg bg-card hover:bg-card-hover"#>
            <Show when=move || !editing.get()>
                <Transition fallback=move || {
                    view! { <div>"Loading..."</div> }
                }>
                    {move || {
                        if let Some(id) = illustration.get().unwrap_or_default() { 
                            view! {
                                <div class="h-48 w-48 flex justify-center m-auto">
                                    <img 
                                        src=move || format!("/image/{}", id) 
                                        class=r#"text-blue-600 underline object-cover shadow-sm"#
                                    />
                                </div>
                            }.into_any()
                        } else {
                            "".into_any()
                        }
                    }}
                </Transition>
                <h3 class=r#"font-bold text-3xl/8"#>{move || name_signal.get().clone()}</h3>
                <p class=r#"text-lg/8"#>
                    <b>"ID: "</b>
                    {move || id_signal.get().clone()}
                </p>
                <p class=r#"text-lg/8"#>
                    <b>"Description: "</b>
                    {move || {
                        if let Some(description) = description_signal.get() {
                            description.clone().into_any()
                        } else {
                            "".into_any()
                        }
                    }}
                </p>
                // <time datetime=move || start_at_signal.get()></time>
                // <time datetime=move || end_at_signal.get()></time>
                <p class=r#"text-lg/8"#>
                    <b>"Start Date: "</b>
                    {move || start_at_signal.get().to_string()}
                </p>
                <p class=r#"text-lg/8"#>
                    <b>"End Date: "</b>
                    {move || end_at_signal.get().to_string()}
                </p>
            </Show>

            <Show when=move || editing.get()>
                <label class=r#"block mb-1 text-sm font-medium text-gray-700"#>"Name"</label>
                <input
                    class=r#"py-2 px-3 w-full text-sm rounded-md border border-gray-300 
                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                    name="name"
                    value=move || name_signal.get()
                    bind:value=name_edit
                />

                <label class=r#"block mb-1 text-sm font-medium text-gray-700"#>"Description"</label>
                <input
                    class=r#"py-2 px-3 w-full text-sm rounded-md border border-gray-300 
                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                    name="description"
                    value=move || description_signal.get()
                    on:change=move |ev: Event| {
                        let value = event_target_value(&ev);
                        description_edit.set(Some(value));
                    }
                />

                <label class=r#"block mb-1 text-sm font-medium text-gray-700"#>"Start Date"</label>
                <input
                    class=r#"py-2 px-3 w-full text-sm rounded-md border border-gray-300 
                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                    type="datetime-local"
                    name="start_at"
                    value=move || start_at_signal.get().to_string()
                    on:change=move |ev: Event| {
                        let value_string = event_target_value(&ev);
                        let value = DateTime::from_event(&ev)
                            .unwrap_or(html_local_to_datetime(value_string));
                        start_at_edit.set(value);
                    }
                />

                <label class=r#"block mb-1 text-sm font-medium text-gray-700"#>"End Date"</label>
                <input
                    class=r#"py-2 px-3 w-full text-sm rounded-md border border-gray-300 
                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                    type="datetime-local"
                    name="end_at"
                    value=move || end_at_signal.get().to_string()
                    on:change=move |ev: Event| {
                        let value_string = event_target_value(&ev);
                        let value = DateTime::from_event(&ev)
                            .unwrap_or(html_local_to_datetime(value_string));
                        end_at_edit.set(value);
                    }
                />

                <label class=r#"block mb-1 text-sm font-medium text-gray-700"#>"Attachment"</label>
                <input
                    class=r#"w-full text-sm"#
                    type="file"
                    name="attachment"
                    multiple
                    on:change=move |ev: Event| {
                        let input = ev.target().unwrap().unchecked_into::<HtmlInputElement>();
                        if let Some(files) = input.files() && files.length() > 0 {
                            let file = files.get(0).unwrap();
                            let fd = FormData::new().unwrap();
                            fd.append_with_blob_and_filename("file", &file, &file.name()).unwrap();
                            file_upload_action.dispatch_local(fd);
                        }
                    }
                /><p>{move || uploading_file_text.get()}</p>

                <label class=r#"block mb-1 text-sm font-medium text-gray-700"#>"Illustration"</label>
                <input
                    class=r#"w-full text-sm"#
                    type="file"
                    name="illustration"
                    on:change=move |ev: Event| {
                        let input = ev.target().unwrap().unchecked_into::<HtmlInputElement>();
                        if let Some(files) = input.files() && files.length() > 0 {
                            let file = files.get(0).unwrap();
                            let fd = FormData::new().unwrap();
                            fd.append_with_blob_and_filename("file", &file, &file.name()).unwrap();
                            illustration_upload_action.dispatch_local(fd);
                        }
                    }
                /><p>{move || uploading_illustration_text.get()}</p>
            </Show>

            <div class=r#"flex flex-row-reverse gap-3 mt-2"#>
                <Show when=move || editing.get() || deleting.get()>
                    <button
                        class=r#"py-2 px-4 text-sm rounded-md border border-gray-300 hover:bg-gray-50"#
                        on:click=move |_| {
                            editing.set(false);
                            deleting.set(false);
                        }
                    >
                        "Cancel"
                    </button>
                </Show>
                <button
                    type="button"
                    hidden=move || deleting.get()
                    class=r#"inline-flex gap-2 items-center py-2 px-4 text-sm font-medium text-white 
                    rounded-lg transition focus:ring-2 focus:outline-none active:scale-95 
                    bg-yale-blue-600 hover:bg-yale-blue-700 focus:ring-yale-blue-400"#
                    on:click=move |_| {
                        let event_id = id_signal.get();
                        let name = name_edit.get();
                        let description = description_edit.get();
                        let start_at = start_at_edit.get();
                        let end_at = end_at_edit.get();
                        let attachments = attachments_edit.get();
                        let illustration = illustration_edit.get();
                        if editing.get() {
                            spawn_local(async move {
                                tracing::debug!("editing event: {}", id_signal.get().clone());
                                if let Ok(ApiResult { result, .. }) = crate::server::admin::event(crate::server::admin::EventAction::Edit {
                                        id: event_id,
                                        name: name.clone(),
                                        description: description.clone().unwrap_or_default(),
                                        start_at,
                                        end_at,
                                        attachments,
                                        illustration
                                    })
                                    .await && result == ResultStatus::Success
                                {
                                    refresh.update(|n| *n += 1);
                                    name_signal.set(name);
                                    description_signal.set(description);
                                    start_at_signal.set(start_at);
                                    end_at_signal.set(end_at);
                                }
                            });
                            editing.set(false)
                        } else {
                            editing.set(true)
                        }
                    }
                >
                    {move || edit_submit_btn_text.get()}
                </button>

                <button
                    hidden=move || editing.get()
                    class=r#"inline-flex items-center py-2 px-4 ml-auto text-sm font-semibold text-white 
                    bg-red-600 rounded-md shadow-sm hover:bg-red-500 focus:ring-2 focus:outline-none 
                    focus:ring-yale-blue-500"#
                    on:click=move |_| {
                        if deleting.get() {
                            let event_id = event.id.clone();
                            spawn_local(async move {
                                tracing::debug!("deleting event: {event_id}");
                                if let Ok(ApiResult { result, .. }) = crate::server::admin::event(crate::server::admin::EventAction::Delete {
                                        id: event_id.clone(),
                                    })
                                    .await && result == ResultStatus::Success
                                {
                                    refresh.update(|n| *n += 1);
                                    deleting.set(false);
                                }
                            });
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
