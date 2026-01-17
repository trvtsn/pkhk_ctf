use crate::{server::{db, enums::ResultStatus, structs::ApiResult}, utils::{html_local_to_datetime}};
use chrono::DateTime;
use leptos::{prelude::*, task::spawn_local, web_sys::Event};

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

    let editing = RwSignal::new(false);
    let deleting = RwSignal::new(false);

    let delete_submit_btn_text = Memo::new(move |_| {
        if deleting.get() { "Confirm Delete".to_string() } else { "Delete".to_string() }
    });
    let edit_submit_btn_text = Memo::new(move |_| {
        if editing.get() { "Confirm Edit".to_string() } else { "Edit".to_string() }
    });

    view! {
        <div class="content-center p-4 rounded-lg bg-yale-blue-50 hover:bg-yale-blue-100">
            <Show when=move || !editing.get()>
                <h3 class="font-bold text-3xl/8">{move || name_signal.get().clone()}</h3>
                <p class="text-lg/8">
                    <b>"ID: "</b>
                    {move || id_signal.get().clone()}
                </p>
                <p class="text-lg/8">
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
                <p class="text-lg/8">
                    <b>"Start Date: "</b>
                    {move || start_at_signal.get().to_string()}
                </p>
                <p class="text-lg/8">
                    <b>"End Date: "</b>
                    {move || end_at_signal.get().to_string()}
                </p>
            </Show>

            <Show when=move || editing.get()>
                <label class="block mb-1 text-sm font-medium text-gray-700">"Name"</label>
                <input
                    class="py-2 px-3 w-full text-sm rounded-md border border-gray-300 focus:ring-2 focus:ring-yale-blue-500 focus:outline-none"
                    name="name"
                    value=move || name_signal.get()
                    bind:value=name_edit
                />

                <label class="block mb-1 text-sm font-medium text-gray-700">"Description"</label>
                <input
                    class="py-2 px-3 w-full text-sm rounded-md border border-gray-300 focus:ring-2 focus:ring-yale-blue-500 focus:outline-none"
                    name="description"
                    value=move || description_signal.get()
                    on:change=move |ev: Event| {
                        let value = event_target_value(&ev);
                        description_edit.set(Some(value));
                    }
                />

                <label class="block mb-1 text-sm font-medium text-gray-700">"Start Date"</label>
                <input
                    class="py-2 px-3 w-full text-sm rounded-md border border-gray-300 focus:ring-2 focus:ring-yale-blue-500 focus:outline-none"
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

                <label class="block mb-1 text-sm font-medium text-gray-700">"End Date"</label>
                <input
                    class="py-2 px-3 w-full text-sm rounded-md border border-gray-300 focus:ring-2 focus:ring-yale-blue-500 focus:outline-none"
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
            </Show>

            <div class="flex flex-row-reverse gap-3 mt-2">
                <Show when=move || editing.get() || deleting.get()>
                    <button
                        class="py-2 px-4 text-sm rounded-md border border-gray-300 hover:bg-gray-50"
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
                    class="inline-flex gap-2 items-center py-2 px-4 text-sm font-medium text-white rounded-lg transition focus:ring-2 focus:ring-yale-blue-400 focus:outline-none active:scale-95 bg-yale-blue-600 hover:bg-yale-blue-700"
                    on:click=move |_| {
                        let event_id = id_signal.get();
                        let name = name_edit.get();
                        let description = description_edit.get();
                        let start_at = start_at_edit.get();
                        let end_at = end_at_edit.get();
                        if editing.get() {
                            spawn_local(async move {
                                tracing::debug!("editing event: {}", id_signal.get().clone());
                                if let Ok(ApiResult { result, .. }) = crate::server::admin::event(crate::server::admin::EventAction::Edit {
                                        id: event_id,
                                        name: name.clone(),
                                        description: description.clone().unwrap_or_default(),
                                        start_at,
                                        end_at,
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
                    class="inline-flex items-center py-2 px-4 ml-auto text-sm font-semibold text-white bg-red-600 rounded-md shadow-sm hover:bg-red-500 focus:ring-2 focus:ring-yale-blue-500 focus:outline-none"
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
