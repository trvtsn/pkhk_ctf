use crate::{components::{admin::event::Event, utils::{ComponentSize, Spinner}}, pages::admin::Actions, server::{admin::{get_all_events_with_attachments, get_all_user_groups, upload_files, upload_illustration}, db::{self, structs::AttachmentWithoutBlob}, enums::ResultStatus, structs::ApiResult}, utils::html_local_to_datetime};
use icondata as i;
use leptos::{prelude::*, task:: spawn_local};
use leptos::{web_sys::{FormData, HtmlInputElement, HtmlSelectElement, HtmlOptionElement, Event}, wasm_bindgen::JsCast};
use leptos_icons::Icon;

/// Default Home Page
#[component]
pub fn Events() -> impl IntoView {
    let creating = RwSignal::new(false);
    let section = RwSignal::new(Actions::None);
    let refresh = RwSignal::new(0);

    let name_signal = RwSignal::new("".to_string());
    let description_signal = RwSignal::new("".to_string());
    let start_at_signal = RwSignal::new("".to_string());
    let end_at_signal = RwSignal::new("".to_string());
    let visible_to_groups_signal = RwSignal::new(vec![]);

    let attachments = RwSignal::<Option<Vec<AttachmentWithoutBlob>>>::new(None);
    let illustration = RwSignal::<Option<AttachmentWithoutBlob>>::new(None);
    
    let ewa_resource = Resource::new(move || refresh.get(), move |_| async move {
        get_all_events_with_attachments().await.unwrap_or_default()
    });
    let groups_signal = RwSignal::new(vec![]);
    let groups_resource = Resource::new(move || refresh.get(), move |_| async move {
        get_all_user_groups().await.unwrap_or_default()
    });

    let file_upload_action = Action::new_local(move |data: &FormData| {
        let data = data.clone();
        async move {
            if let Ok(api_result) = upload_files(data.clone().into()).await {
                attachments.set(Some(api_result.details.clone()))
            }
        }
    });

    let illustration_upload_action = Action::new_local(move |data: &FormData| {
        let data = data.clone();
        async move {
            if let Ok(api_result) = upload_illustration(data.into()).await {
                illustration.set(Some(api_result.details.clone()));
            }
        }
    });

    let uploading_file_text = Memo::new(move |_| {
        if file_upload_action.pending().get() {
            "Uploading...".to_string()
        } else {
            "".to_string()
        }
    });

    let uploading_illustration_text = Memo::new(move |_| {
        if illustration_upload_action.pending().get() {
            "Uploading...".to_string()
        } else {
            "".to_string()
        }
    });

    view! {
        <Transition>
            {move || {
                let groups = groups_resource.get().unwrap_or_default();
                groups_signal.set(groups);

                view! {
                    <div class=r#"flex gap-2 mb-4"#>
                        <button
                            class=r#"py-1 px-3 text-sm rounded-md border border-input-border hover:bg-background-hover"#
                            on:click=move |_| {
                                if creating.get() {
                                    creating.set(false);
                                    section.set(Actions::None);
                                } else {
                                    creating.set(true);
                                    section.set(Actions::Create);
                                }
                            }
                        >
                            "Create"
                        </button>
                    </div>

                    <div class=r#"flex flex-col gap-4"#>
                        <Show when=move || section.get() == Actions::Create>
                            <div class="grid">
                                <label class=r#"block mb-1 text-sm font-medium text-text"#>"Name"</label>
                                <input
                                    class=r#"bg-background py-2 px-3 w-full text-sm rounded-md border border-input-border 
                                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                                    name="name"
                                    bind:value=name_signal
                                />
                            </div>

                            <div class="grid">
                                <label class=r#"block mb-1 text-sm font-medium text-text"#>"Description"</label>
                                <textarea
                                    class=r#"bg-background py-2 px-3 w-full text-sm rounded-md border border-input-border 
                                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                                    name="description"
                                    bind:value=description_signal
                                />
                            </div>

                            <div class="grid">
                                <label class=r#"block mb-1 text-sm font-medium text-text"#>"Start Date"</label>
                                <input
                                    class=r#"bg-background py-2 px-3 w-full text-sm rounded-md border border-input-border 
                                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                                    type="datetime-local"
                                    name="start_at"
                                    bind:value=start_at_signal
                                />
                            </div>

                            <div class="grid">
                                <label class=r#"block mb-1 text-sm font-medium text-text"#>"End Date"</label>
                                <input
                                    class=r#"bg-background py-2 px-3 w-full text-sm rounded-md border border-input-border 
                                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                                    type="datetime-local"
                                    name="end_at"
                                    bind:value=end_at_signal
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
                                        let sel = ev.target().unwrap().unchecked_into::<HtmlSelectElement>();
                                        let selected = sel.selected_options();
                                        let mut picked: Vec<String> = Vec::new();

                                        for i in 0..selected.length() {
                                            if let Some(item) = selected.item(i) {
                                                if let Ok(opt) = item.dyn_into::<HtmlOptionElement>() {
                                                    picked.push(opt.value());
                                                }
                                            }
                                        }

                                        visible_to_groups_signal.set(picked);
                                    }
                                >
                                    <option value="all">"All"</option>
                                    {move || {
                                        let groups = groups_signal.get();
                                        view! {
                                            <For
                                                each=move || groups.clone()
                                                key=|group: &String| group.clone()
                                                let(group)
                                            >
                                                <option value={group.clone()}>{group.clone()}</option>
                                            </For>
                                        }
                                    }}
                                </select>
                            </div>

                            <div class="grid">
                                <label class=r#"block mb-1 text-sm font-medium text-text"#>"Attachments"</label>
                                <div class="grid gap-2">
                                    <ForEnumerate
                                        each=move || attachments.get().unwrap_or_default()
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
                                                            let remove_at = index.get();

                                                            attachments.update(|a| {
                                                                a.get_or_insert_default().remove(remove_at);
                                                            });
                                                        } 
                                                    >
                                                        <Icon icon=i::LuX />
                                                    </button>
                                                </div>
                                            }
                                        }}
                                    />
                                </div>
                                <div class="flex gap-2">
                                    <input
                                        class=r#"bg-background w-full text-sm p-3 rounded-lg shadow-sm"#
                                        type="file"
                                        name="attachments"
                                        multiple
                                        on:change=move |ev: Event| {
                                            let input = ev.target().unwrap().unchecked_into::<HtmlInputElement>();
                                            if let Some(files) = input.files() && files.length() > 0 {
                                                let files_count = files.length();
                                                let fd = FormData::new().unwrap();
                                                for i in 0..files_count {
                                                    let file = files.get(i).unwrap();
                                                    fd.append_with_blob_and_filename("file", &file, &file.name()).unwrap();
                                                }
                                                file_upload_action.dispatch_local(fd);
                                            }
                                        }
                                    />
                                    <p>{uploading_file_text.get()}</p>
                                </div>
                            </div>

                            <div class="grid">
                                <label class=r#"block mb-1 text-sm font-medium text-text"#>"Illustration"</label>
                                <div class="grid gap-2">
                                    {move || {
                                        let illustration_signal_value = illustration.get();
                                        if let Some(illustr) = illustration_signal_value {
                                            let show_tooltip = RwSignal::new(false);
                                            let id = illustr.id.clone();
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
                                                        {move || illustr.file_name.clone()}
                                                        <Show when=move || show_tooltip.get()>
                                                            <div
                                                                role="tooltip"
                                                                class=r#"absolute left-1/2 bottom-full -translate-x-1/2 whitespace-nowrap 
                                                                    rounded p-1 text-xs bg-card-hover shadow-sm z-1"#
                                                            >
                                                                {format!("ID: {}", illustr.id)}
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
                                                            illustration.set(None);
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
                                </div>
                                <div class="flex gap-2">
                                    <input
                                        class=r#"bg-background w-full text-sm p-3 rounded-lg shadow-sm"#
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
                                    />
                                    <p>{uploading_illustration_text.get()}</p>
                                </div>
                            </div>

                            <div class=r#"flex gap-3 mt-2"#>
                                <button
                                    type="button"
                                    class=r#"py-2 px-4 text-sm rounded-md border border-input-border hover:bg-background-hover"#
                                    on:click=move |_| { section.set(Actions::None) }
                                >
                                    "Cancel"
                                </button>
                                <button
                                    type="button"
                                    class=r#"inline-flex items-center py-2 px-4 ml-auto text-sm font-semibold 
                                    text-white rounded-md shadow-sm focus:ring-2 focus:outline-none 
                                    bg-yale-blue-600 hover:bg-yale-blue-500 focus:ring-yale-blue-500"#
                                    on:click=move |_| {
                                        let name = name_signal.get();
                                        let description = description_signal.get();
                                        let start_at = html_local_to_datetime(start_at_signal.get());
                                        let end_at = html_local_to_datetime(end_at_signal.get());
                                        let visible_to_groups = visible_to_groups_signal.get().join(",");
                                        let attachments = attachments.get();
                                        let illustration = illustration.get();
                                        spawn_local(async move {
                                            tracing::debug!("creating event...");
                                            if let Ok(ApiResult { result, .. }) = crate::server::admin::event(crate::server::admin::EventAction::Create {
                                                    name,
                                                    description,
                                                    start_at,
                                                    end_at,
                                                    visible_to_groups,
                                                    attachments,
                                                    illustration
                                                })
                                                .await && result == ResultStatus::Success
                                            {
                                                refresh.update(|n| *n += 1);
                                            }
                                        });
                                    }
                                >
                                    "Create"
                                </button>
                            </div>
                        </Show>
                    </div>

                    <div class=r#"events pt-4"#>
                        <Transition fallback=move || {
                            view! { <Spinner component_size=ComponentSize::Big /> }
                        }>
                            {move || {
                                let events = ewa_resource.get().unwrap_or_default();
                                view! {
                                    <div class=r#"grid grid-cols-4 content-stretch"#>
                                        <For
                                            each=move || events.clone()
                                            key=|ewa: &db::structs::EventWithAttachments| ewa.event.id.clone()
                                            let(ewa)
                                        >
                                            <div class=r#"p-2 event"#>
                                                <Event 
                                                    ewa
                                                    user_groups=groups_signal
                                                    refresh 
                                                />
                                            </div>
                                        </For>
                                    </div>
                                }
                            }}
                        </Transition>
                    </div>
                }
            }}
        </Transition>
    }
}
