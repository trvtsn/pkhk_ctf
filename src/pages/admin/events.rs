use crate::{components::{admin::event::Event, toast::{ToastMessageType, push_new_toast}, utils::{ComponentSize, FileTooltip, Spinner}}, pages::admin::Actions, server::{admin::api::{get_all_events_with_attachments, get_all_user_groups, upload_files, upload_illustration}, db::{self, structs::{AttachmentWithoutBlob, EventWithAttachments}}, enums::{ServerEventPayload, ResultStatus}, structs::ApiResult}, utils::html_local_to_datetime};
use crate::utils::{build_multi_file_form_data, build_single_file_form_data, collect_selected_options};
use leptos::{prelude::*, task:: spawn_local};
use leptos::{web_sys::{HtmlSelectElement, Event}, wasm_bindgen::JsCast};
use leptos_use::{UseEventSourceOptions, UseEventSourceReturn, use_event_source_with_options};
use leptos::server::codee::string::FromToStringCodec;

/// Admin event management.
/// Create, edit, and delete CTF events.
#[component]
pub fn Events() -> impl IntoView {
    let attachments_ref = NodeRef::new();
    let illustration_ref = NodeRef::new();

    let editing_ids = RwSignal::<Vec<String>>::new(vec![]);
    let pending_ewa_updates = RwSignal::<Vec<EventWithAttachments>>::new(vec![]);
    let creating = RwSignal::new(false);
    let section = RwSignal::new(Actions::None);

    let name_signal = RwSignal::new("".to_string());
    let description_signal = RwSignal::new("".to_string());
    let start_at_signal = RwSignal::new("".to_string());
    let end_at_signal = RwSignal::new("".to_string());
    let visible_to_groups_signal = RwSignal::new(vec![]);

    let ewa_signal = RwSignal::<Vec<EventWithAttachments>>::new(vec![]);
    let attachments = RwSignal::<Option<Vec<AttachmentWithoutBlob>>>::new(None);
    let illustration = RwSignal::<Option<AttachmentWithoutBlob>>::new(None);
    
    let ewa_resource = Resource::new(move || (), move |_| async move {
        get_all_events_with_attachments().await.unwrap_or_default()
    });
    
    let groups_signal = RwSignal::new(vec![]);
    let groups_resource = Resource::new(move || (), move |_| async move {
        get_all_user_groups().await.unwrap_or_default()
    });

    let UseEventSourceReturn { message, .. } =
        use_event_source_with_options::<String, FromToStringCodec>(
            "/admin/events".to_string(),
            UseEventSourceOptions::default().immediate(true)
        );

    Effect::new(move |_| {
        if let Some(msg) = message.get() {
            match serde_json::from_str::<ServerEventPayload>(&msg.data) {
                Ok(ServerEventPayload::EventEdited(new_ewa)) => {
                    if editing_ids.get_untracked().contains(&new_ewa.event.id) {
                        pending_ewa_updates.update(|pending| {
                            pending.retain(|p| p.event.id != new_ewa.event.id);
                            pending.push(new_ewa);
                        });
                    } else {
                        ewa_signal.update(|events| {
                            if let Some(existing) = events.iter_mut().find(|e| e.event.id == new_ewa.event.id) {
                                *existing = new_ewa;
                            }
                        });
                    }
                },
                Ok(ServerEventPayload::EventDeleted(id)) => {
                    ewa_signal.update(|ewa| ewa.retain(|ewa| ewa.event.id != id));
                },
                Ok(ServerEventPayload::NewEventCreated(new_ewa)) => {
                    ewa_signal.update(|ewa| ewa.push(new_ewa));
                }, 
                Ok(_) => {},
                Err(e) => tracing::warn!("failed to parse ServerEventPayload: {}", e)
            }
        }
    });

    // Flush pending SSE updates for events no longer being edited
    Effect::new(move |_| {
        let current_editing = editing_ids.get();
        let pending = pending_ewa_updates.get_untracked();
        let to_flush: Vec<_> = pending.iter()
            .filter(|p| !current_editing.contains(&p.event.id))
            .cloned()
            .collect();
        if !to_flush.is_empty() {
            pending_ewa_updates.update(|p| p.retain(|u| current_editing.contains(&u.event.id)));
            ewa_signal.update(|events| {
                for updated_ewa in to_flush {
                    if let Some(existing) = events.iter_mut().find(|c| c.event.id == updated_ewa.event.id) {
                        *existing = updated_ewa;
                    }
                }
            });
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
                            class=r#"py-1 px-3 text-sm rounded-md border border-input-border bg-background hover:bg-background-hover"#
                            on:click=move |_| {
                                if creating.get_untracked() {
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
                                        let sel = match ev.target() {
                                            Some(target) => target.unchecked_into::<HtmlSelectElement>(),
                                            None => { push_new_toast(ToastMessageType::ErrorOccurred); return }
                                        };
                                        visible_to_groups_signal.set(collect_selected_options(&sel));
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
                                                <option value=group>
                                                    {group.clone()}
                                                </option>
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
                                            let id = a.id.clone();
                                            view! {
                                                <FileTooltip
                                                    file_name=a.file_name.clone()
                                                    id=a.id.clone()
                                                    on_download=format!("/file/{}", id)
                                                    on_remove=Callback::new(move |_| {
                                                        let remove_at = index.get_untracked();
                                                        attachments.update(|a| { a.get_or_insert_default().remove(remove_at); });
                                                    })
                                                />
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
                                        node_ref=attachments_ref
                                    />
                                </div>
                            </div>

                            <div class="grid">
                                <label class=r#"block mb-1 text-sm font-medium text-text"#>"Illustration"</label>
                                <div class="grid gap-2">
                                    {move || {
                                        let illustration_signal_value = illustration.get();
                                        if let Some(illustr) = illustration_signal_value {
                                            view! {
                                                <FileTooltip
                                                    file_name=illustr.file_name.clone()
                                                    id=illustr.id.clone()
                                                    on_download=format!("/file/{}", illustr.id)
                                                    on_remove=Callback::new(move |_| illustration.set(None))
                                                />
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
                                        node_ref=illustration_ref
                                    />
                                </div>
                            </div>

                            <div class=r#"flex gap-3 mt-2"#>
                                <button
                                    type="button"
                                    class=r#"py-2 px-4 text-sm rounded-md border border-input-border hover:bg-background-hover"#
                                    on:click=move |_| section.set(Actions::None)
                                >
                                    "Cancel"
                                </button>
                                <button
                                    type="button"
                                    class=r#"inline-flex items-center py-2 px-4 ml-auto text-sm font-semibold 
                                    text-white rounded-md shadow-sm focus:ring-2 focus:outline-none 
                                    bg-yale-blue-600 hover:bg-yale-blue-500 focus:ring-yale-blue-500"#
                                    on:click=move |_| {
                                        let name = name_signal.get_untracked();
                                        let description = description_signal.get_untracked();
                                        let start_at = html_local_to_datetime(start_at_signal.get_untracked());
                                        let end_at = html_local_to_datetime(end_at_signal.get_untracked());
                                        let visible_to_groups = visible_to_groups_signal.get_untracked().join(",");

                                        let attachments_ref = attachments_ref.get_untracked();
                                        let illustration_ref = illustration_ref.get_untracked();

                                        spawn_local(async move {
                                            tracing::debug!("creating event...");

                                            if let Some(fd) = build_multi_file_form_data(attachments_ref) {
                                                if let Ok(api_result) = upload_files(fd.into()).await {
                                                    attachments.set(Some(api_result.details));
                                                }
                                            }

                                            if let Some(fd) = build_single_file_form_data(illustration_ref) {
                                                if let Ok(api_result) = upload_illustration(fd.into()).await {
                                                    illustration.set(Some(api_result.details));
                                                }
                                            }

                                            let attachments = attachments.get_untracked();
                                            let illustration = illustration.get_untracked();

                                            if let Ok(ApiResult { result, .. }) = crate::server::admin::api::event(crate::server::admin::EventAction::Create {
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
                                                push_new_toast(ToastMessageType::EventCreated);
                                            } else {
                                                push_new_toast(ToastMessageType::EventCreateFail);
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
                                ewa_signal.set(events);

                                view! {
                                    <div class=r#"grid grid-cols-4 content-stretch"#>
                                        <For
                                            each=move || ewa_signal.get()
                                            key=|ewa: &db::structs::EventWithAttachments| format!("{ewa:?}")
                                            let(ewa)
                                        >
                                            <div class=r#"p-2 event"#>
                                                <Event 
                                                    ewa
                                                    editing_ids
                                                    user_groups=groups_signal
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
