use crate::{components::toast::{ToastMessageType, push_new_toast}, server::{admin::{upload_files, upload_illustration}, db::{self, structs::AttachmentWithoutBlob}, enums::ResultStatus, structs::ApiResult}, utils::html_local_to_datetime};
use chrono::DateTime;
use icondata as i;
use leptos::{prelude::*, task::spawn_local, wasm_bindgen::JsCast, web_sys::{Event, FormData, HtmlOptionElement, HtmlSelectElement}};
use leptos_icons::Icon;

#[component]
pub fn Event(
    ewa: db::structs::EventWithAttachments,
    user_groups: RwSignal<Vec<String>>, 
    refresh: RwSignal<i32>
) -> impl IntoView {
    let attachments_ref = NodeRef::new();
    let illustration_ref = NodeRef::new();

    let id_signal = RwSignal::new(ewa.event.id.clone());
    let name_signal = RwSignal::new(ewa.event.name.clone());
    let description_signal = RwSignal::new(ewa.event.description.clone());
    let start_at_signal = RwSignal::new(ewa.event.start_at);
    let end_at_signal = RwSignal::new(ewa.event.end_at);
    let visible_to_groups_signal = RwSignal::new(ewa.event.visible_to_groups.clone());
    let attachments_signal = RwSignal::new(ewa.attachments.clone());
    let illustration_signal = RwSignal::new(ewa.illustration.clone());

    let name_edit = RwSignal::new(ewa.event.name.clone());
    let description_edit = RwSignal::new(ewa.event.description.clone());
    let start_at_edit = RwSignal::new(ewa.event.start_at);
    let end_at_edit = RwSignal::new(ewa.event.end_at);
    let visible_to_groups_edit = RwSignal::new(ewa.event.visible_to_groups);
    let attachments_edit = RwSignal::new(ewa.attachments);
    let illustration_edit = RwSignal::new(ewa.illustration);

    let editing = RwSignal::new(false);
    let deleting = RwSignal::new(false);

    let delete_submit_btn_text = Memo::new(move |_| {
        if deleting.get() { "Confirm Delete".to_string() } else { "Delete".to_string() }
    });
    let edit_submit_btn_text = Memo::new(move |_| {
        if editing.get() { "Confirm Edit".to_string() } else { "Edit".to_string() }
    });

    let any_changes_made = Memo::new(move |_| {
        if name_signal.get() == name_edit.get() &&
            description_signal.get() == description_edit.get() &&
            start_at_signal.get() == start_at_edit.get() &&
            end_at_signal.get() == end_at_edit.get() &&
            visible_to_groups_signal.get() == visible_to_groups_edit.get() &&
            attachments_signal.get() == attachments_edit.get() &&
            illustration_signal.get() == illustration_edit.get() 
        { false } else { true }
    });

    view! {
        <div class=r#"content-center p-4 rounded-lg bg-card hover:bg-card-hover text-text break-all"#>
            <Show when=move || !editing.get()>
                <h3 class=r#"font-bold text-3xl/8 mb-4"#>{move || name_signal.get()}</h3>
                {move || {
                    if let Some(illustration) = illustration_signal.get() {
                        view! {
                            <div class="flex justify-center m-auto mb-4">
                                <img 
                                    src=move || format!("/image/{}", illustration.id) 
                                    class=r#"shadow-sm"#
                                />
                            </div>
                        }.into_any()
                    } else {
                        "".into_any()
                    }
                }}
                <p class=r#"text-lg/8"#>
                    <b>"ID: "</b>
                    {move || id_signal.get()}
                </p>
                <p class=r#"text-lg/8 whitespace-pre-wrap"#>
                    <b>"Description: "</b>
                    {move || {
                        if let Some(description) = description_signal.get() {
                            description.into_any()
                        } else {
                            "".into_any()
                        }
                    }}
                </p>
                // <time datetime=move || start_at_signal.get()></time>
                // <time datetime=move || end_at_signal.get()></time>
                <p class=r#"text-lg/8"#>
                    <b>"Visible To Groups: "</b>
                    {move || visible_to_groups_signal.get().replace(",", ", ")}
                </p>
                <p class=r#"text-lg/8"#>
                    <b>"Start Date: "</b>
                    {move || start_at_signal.get().format("%Y-%m-%d %H:%M:%S").to_string()}
                </p>
                <p class=r#"text-lg/8"#>
                    <b>"End Date: "</b>
                    {move || end_at_signal.get().format("%Y-%m-%d %H:%M:%S").to_string()}
                </p>
            </Show>

            <Show when=move || editing.get()>
                <div class="grid gap-3">
                    <div class="grid">
                        <label class=r#"block mb-1 text-sm font-medium"#>"Name"</label>
                        <input
                            class=r#"bg-background py-2 px-3 w-full text-sm rounded-md border border-input-border 
                            focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                            name="name"
                            value=move || name_signal.get()
                            bind:value=name_edit
                        />
                    </div>

                    <div class="grid">
                        <label class=r#"block mb-1 text-sm font-medium"#>"Description"</label>
                        <textarea
                            class=r#"bg-background py-2 px-3 w-full text-sm rounded-md border border-input-border 
                            focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                            name="description"
                            prop:value=move || description_signal.get()
                            on:change=move |ev: Event| {
                                let value = event_target_value(&ev);
                                description_edit.set(Some(value));
                            }
                        />
                    </div>

                    <div class="grid">
                        <label class=r#"block mb-1 text-sm font-medium"#>"Start Date"</label>
                        <input
                            class=r#"bg-background py-2 px-3 w-full text-sm rounded-md border border-input-border 
                            focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                            type="datetime-local"
                            name="start_at"
                            value=move || start_at_signal.get().format("%Y-%m-%d %H:%M:%S").to_string()
                            on:change=move |ev: Event| {
                                let value_string = event_target_value(&ev);
                                let value = DateTime::from_event(&ev)
                                    .unwrap_or(html_local_to_datetime(value_string));
                                start_at_edit.set(value);
                            }
                        />
                    </div>

                    <div class="grid">
                        <label class=r#"block mb-1 text-sm font-medium"#>"End Date"</label>
                        <input
                            class=r#"bg-background py-2 px-3 w-full text-sm rounded-md border border-input-border 
                            focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                            type="datetime-local"
                            name="end_at"
                            value=move || end_at_signal.get().format("%Y-%m-%d %H:%M:%S").to_string()
                            on:change=move |ev: Event| {
                                let value_string = event_target_value(&ev);
                                let value = DateTime::from_event(&ev)
                                    .unwrap_or(html_local_to_datetime(value_string));
                                end_at_edit.set(value);
                            }
                        />
                    </div>

                    <div class="grid">
                        <label class=r#"block mb-1 text-sm font-medium text-text"#>"Visible To Groups"</label>
                        <select
                            class=r#"bg-background py-2 px-3 w-full text-sm rounded-md border border-input-border 
                            focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                            name="visible_to_groups"
                            multiple=true
                            on:change=move |ev: Event| {
                                let sel = match ev.target() {
                                    Some(target) => target.unchecked_into::<HtmlSelectElement>(),
                                    None => { push_new_toast(ToastMessageType::ErrorOccurred); return }
                                };
                                let selected = sel.selected_options();
                                let mut picked: Vec<String> = Vec::new();

                                for i in 0..selected.length() {
                                    if let Some(item) = selected.item(i) {
                                        if let Ok(opt) = item.dyn_into::<HtmlOptionElement>() {
                                            picked.push(opt.value());
                                        }
                                    }
                                }

                                visible_to_groups_edit.set(picked.join(","));
                            }
                        >
                            <option 
                                value="all"
                                selected=move || {
                                    visible_to_groups_edit
                                        .get().split(",")
                                        .map(String::from)
                                        .collect::<Vec<String>>()
                                        .contains(&"all".to_string())
                                }
                            >
                                "All"
                            </option>
                            {move || {
                                view! {
                                    <For
                                        each=move || user_groups.get()
                                        key=|group: &String| group.clone()
                                        children=move |group| {
                                            let selected = visible_to_groups_edit
                                                .get().split(",")
                                                .map(String::from)
                                                .collect::<Vec<String>>()
                                                .contains(&group);

                                            view! {
                                                <option 
                                                    value=group
                                                    selected=selected
                                                >
                                                    {group.clone()}
                                                </option>
                                            }
                                        }
                                    />
                                }
                            }}
                        </select>
                    </div>

                    <div class="grid">
                        <label class=r#"block mb-1 text-sm font-medium"#>"Attachments"</label>
                        <div class="grid gap-2">
                            <ForEnumerate
                                each=move || attachments_edit.get()
                                key=|a: &AttachmentWithoutBlob| a.id.clone()
                                children={move |index, a| {
                                    let show_tooltip = RwSignal::new(false);
                                    let id = a.id.clone();
                                    let file_name = a.file_name.clone();
                                    view! {  
                                        <div class="flex gap-2 items-center">
                                            <span
                                                class="relative inline-block"
                                                on:mouseenter=move |_| show_tooltip.set(true)
                                                on:mouseleave=move |_| show_tooltip.set(false)
                                                // keyboard focus
                                                on:focus=move |_| show_tooltip.set(true)
                                                on:blur=move |_| show_tooltip.set(false)
                                                tabindex="0"
                                            >
                                                {file_name}
                                                <Show when=move || show_tooltip.get()>
                                                    <div
                                                        role="tooltip"
                                                        class=r#"absolute left-1/2 bottom-full -translate-x-1/2 whitespace-nowrap 
                                                            rounded p-1 text-xs bg-card shadow-sm z-1"#
                                                    >
                                                        {format!("ID: {}", a.id)}
                                                    </div>
                                                </Show>
                                            </span>
                                            
                                            <a
                                                download
                                                href=move || format!("/file/{}", id)
                                            >
                                                <Icon icon=i::LuDownload />
                                            </a>
                                            <button 
                                                class="cursor-pointer"
                                                on:click=move |_| {
                                                    let remove_at = index.get_untracked();

                                                    attachments_edit.update(|a| {
                                                        a.remove(remove_at);
                                                    });
                                                } 
                                            >
                                                <Icon icon=i::LuX />
                                            </button>
                                        </div>
                                    }
                                }}
                            />
                            <div class="flex gap-2">
                                <input
                                    class=r#"bg-background w-full text-sm p-3 rounded-lg shadow-sm"#
                                    type="file"
                                    name="attachments"
                                    multiple
                                    node_ref=attachments_ref
                                />
                            </div>
                        </div>
                    </div>

                    <div class="grid">
                        <label class=r#"block mb-1 text-sm font-medium"#>"Illustration"</label>
                        <div class="grid gap-2">
                            {move || {
                                if let Some(illustration) = illustration_edit.get() {
                                    let show_tooltip = RwSignal::new(false);
                                    let id = illustration.id.clone();
                                    view! {
                                        <div class="flex gap-2 items-center">
                                            <span
                                                class="relative inline-block"
                                                on:mouseenter=move |_| show_tooltip.set(true)
                                                on:mouseleave=move |_| show_tooltip.set(false)
                                                on:focus=move |_| show_tooltip.set(true)
                                                on:blur=move |_| show_tooltip.set(false)
                                                tabindex="0"
                                            >
                                                {move || illustration.file_name.clone()}
                                                <Show when=move || show_tooltip.get()>
                                                    <div
                                                        role="tooltip"
                                                        class=r#"absolute left-1/2 bottom-full -translate-x-1/2 whitespace-nowrap 
                                                            rounded p-1 text-xs bg-card shadow-sm z-1"#
                                                    >
                                                        {format!("ID: {}", illustration.id)}
                                                    </div>
                                                </Show>
                                            </span>
                                            
                                            <a
                                                download
                                                href=move || format!("/file/{}", id)
                                            >
                                                <Icon icon=i::LuDownload />
                                            </a>
                                            <button 
                                                class="cursor-pointer"
                                                on:click=move |_| {
                                                    illustration_edit.set(None);
                                                } 
                                            >
                                                <Icon icon=i::LuX />
                                            </button>
                                        </div>
                                    }.into_any()
                                } else {
                                    "".into_any()
                                }
                            }}
                            <input
                                class=r#"bg-background w-full text-sm p-3 rounded-lg shadow-sm"#
                                type="file"
                                name="illustration"
                                node_ref=illustration_ref
                            />
                        </div>
                    </div>
                </div>
            </Show>

            <div class=r#"flex flex-row-reverse gap-3 mt-2"#>
                <Show when=move || editing.get() || deleting.get()>
                    <button
                        class=r#"py-2 px-4 text-sm rounded-md border border-input-border hover:bg-background-hover"#
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
                        let event_id = id_signal.get_untracked();
                        let name = name_edit.get_untracked();
                        let description = description_edit.get_untracked();
                        let start_at = start_at_edit.get_untracked();
                        let end_at = end_at_edit.get_untracked();
                        let visible_to_groups = visible_to_groups_edit.get_untracked();

                        let attachments_ref = attachments_ref.get_untracked();
                        let illustration_ref = illustration_ref.get_untracked();

                        if editing.get_untracked() {
                            spawn_local(async move {
                                tracing::debug!("editing event: {}", id_signal.get_untracked());

                                if let Some(att_el) = attachments_ref {
                                    if let Some(files) = att_el.files() {
                                        if files.length() > 0 {
                                            let fd = match FormData::new() {
                                                Ok(fd) => fd,
                                                Err(_) => { push_new_toast(ToastMessageType::ErrorOccurred); return }
                                            };
                                            for i in 0..files.length() {
                                                let file = match files.get(i) {
                                                    Some(file) => file,
                                                    None => { push_new_toast(ToastMessageType::ErrorOccurred); return }
                                                };
                                                match fd.append_with_blob_and_filename("file", &file, &file.name()) {
                                                    Ok(_) => {},
                                                    Err(_) => { push_new_toast(ToastMessageType::ErrorOccurred); return }
                                                }
                                            }

                                            if let Ok(api_result) = upload_files(fd.into()).await {
                                                attachments_edit.set(api_result.details);
                                            }
                                        }
                                    }
                                }

                                if let Some(illustr_el) = illustration_ref {
                                    if let Some(files) = illustr_el.files() {
                                        if files.length() > 0 {
                                            let file = match files.get(0) {
                                                Some(file) => file,
                                                None => { push_new_toast(ToastMessageType::ErrorOccurred); return }
                                            };
                                            let fd = match FormData::new() {
                                                Ok(fd) => fd,
                                                Err(_) => { push_new_toast(ToastMessageType::ErrorOccurred); return }
                                            };
                                            match fd.append_with_blob_and_filename("file", &file, &file.name()) {
                                                Ok(_) => {},
                                                Err(_) => { push_new_toast(ToastMessageType::ErrorOccurred); return }
                                            }

                                            if let Ok(api_result) = upload_illustration(fd.into()).await {
                                                illustration_edit.set(Some(api_result.details))
                                            }
                                        }
                                    }
                                }

                                // using .get_untracked as we're inside an on:click event handler and don't require an active subscription to these values
                                // consider changing the .get()'s of the initial values above
                                let attachments = if attachments_edit.get_untracked().is_empty() { None } else { Some(attachments_edit.get_untracked()) };
                                let illustration = illustration_edit.get_untracked();

                                if !any_changes_made.get_untracked() {
                                    editing.set(false);
                                    push_new_toast(ToastMessageType::NoChangesMade);
                                } else {
                                    if let Ok(ApiResult { result, .. }) = crate::server::admin::event(crate::server::admin::EventAction::Edit {
                                            id: event_id,
                                            name: name.clone(),
                                            description: description.clone().unwrap_or_default(),
                                            start_at,
                                            end_at,
                                            visible_to_groups,
                                            attachments,
                                            illustration
                                        })
                                        .await && result == ResultStatus::Success
                                    {
                                        push_new_toast(ToastMessageType::EventEdited);
                                        refresh.update(|n| *n += 1);
                                        name_signal.set(name);
                                        description_signal.set(description);
                                        start_at_signal.set(start_at);
                                        end_at_signal.set(end_at);
                                    } else {
                                        push_new_toast(ToastMessageType::EventEditFail);
                                    }
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
                        if deleting.get_untracked() {
                            let event_id = ewa.event.id.clone();
                            spawn_local(async move {
                                tracing::debug!("deleting event: {event_id}");
                                if let Ok(ApiResult { result, .. }) = crate::server::admin::event(crate::server::admin::EventAction::Delete {
                                        id: event_id.clone(),
                                    })
                                    .await && result == ResultStatus::Success
                                {
                                    push_new_toast(ToastMessageType::EventDeleted);
                                    refresh.update(|n| *n += 1);
                                    deleting.set(false);
                                } else {
                                    push_new_toast(ToastMessageType::EventDeleteFail);
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
