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
        <div class="bg-yale-blue-50 hover:bg-yale-blue-100 rounded-lg p-4 content-center">
            <Show when=move || !editing.get()>
                <h3 class="text-3xl/8 font-bold">{move || name_signal.get().clone()}</h3>
                <p class="text-lg/8"><b>"ID: "</b>{move || id_signal.get().clone()}</p>
                <p class="text-lg/8"><b>"Description: "</b>{move || {
                    if let Some(description) = description_signal.get() {
                        description.clone().into_any()
                    } else {
                        "".into_any()
                    }
                }}</p>
                //<time datetime=move || start_at_signal.get()></time>
                //<time datetime=move || end_at_signal.get()></time>
                <p class="text-lg/8"><b>"Start Date: "</b>{move || start_at_signal.get().to_string()}</p>
                <p class="text-lg/8"><b>"End Date: "</b>{move || end_at_signal.get().to_string()}</p>
            </Show>

            <Show when=move || editing.get()>
                <label class="block text-sm font-medium text-gray-700 mb-1">"Name"</label>
                <input 
                    class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" 
                    name="name" 
                    value=move || name_signal.get() 
                    bind:value=name_edit
                />
                
                <label class="block text-sm font-medium text-gray-700 mb-1">"Description"</label>
                <input 
                    class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500"
                    name="description" 
                    value=move || description_signal.get() 
                    on:change=move |ev: Event| {
                        let value = event_target_value(&ev);
                        description_edit.set(Some(value));
                    }
                />
                
                <label class="block text-sm font-medium text-gray-700 mb-1">"Start Date"</label>
                <input 
                    class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" 
                    type="datetime-local" 
                    name="start_at" 
                    value=move || start_at_signal.get().to_string() 
                    on:change=move |ev: Event| {
                        let value_string = event_target_value(&ev);
                        let value = DateTime::from_event(&ev).unwrap_or(html_local_to_datetime(value_string));
                        start_at_edit.set(value);
                    }
                />
                
                <label class="block text-sm font-medium text-gray-700 mb-1">"End Date"</label>
                <input 
                    class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" 
                    type="datetime-local" 
                    name="end_at" 
                    value=move || end_at_signal.get().to_string() 
                    on:change=move |ev: Event| {
                        let value_string = event_target_value(&ev);
                        let value = DateTime::from_event(&ev).unwrap_or(html_local_to_datetime(value_string));
                        end_at_edit.set(value);
                    }
                />
            </Show>

            <div class="flex flex-row-reverse gap-3 mt-2">
                <Show when=move || editing.get() || deleting.get()>
                    <button 
                        class="px-4 py-2 rounded-md border border-gray-300 text-sm hover:bg-gray-50" 
                        on:click=move |_| {editing.set(false); deleting.set(false);}
                    >"Cancel"</button>
                </Show>
                <button 
                    type="button" 
                    hidden=move || deleting.get() 
                    class=r#"inline-flex items-center gap-2 rounded-lg text-white px-4 py-2 text-sm font-medium focus:outline-none 
                    focus:ring-2 active:scale-95 transition bg-indigo-600 hover:bg-indigo-700 focus:ring-indigo-400"# 
                    on:click=move |_| {
                        let event_id = id_signal.get();
                        let name = name_edit.get();
                        let description = description_edit.get();
                        let start_at = start_at_edit.get();
                        let end_at = end_at_edit.get();
                        if editing.get() {
                            spawn_local(async move {
                                tracing::debug!("editing event: {}", id_signal.get().clone());
                                if let Ok(ApiResult { result, .. }) = crate::server::admin::event(
                                    crate::server::admin::EventAction::Edit { 
                                        id: event_id, 
                                        name: name.clone(),
                                        description: description.clone().unwrap_or_default(),
                                        start_at,
                                        end_at
                                    }
                                ).await && result == ResultStatus::Success {
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
                >{move || edit_submit_btn_text.get()}</button>

                <button
                    hidden=move || editing.get()
                    class=r#"ml-auto inline-flex items-center px-4 py-2 rounded-md bg-red-600 text-white text-sm 
                    font-semibold shadow-sm hover:bg-red-500 focus:outline-none focus:ring-2 focus:ring-indigo-500"# 
                    on:click=move |_| {
                        if deleting.get() {
                            let event_id = event.id.clone();
                            spawn_local(async move {
                                tracing::debug!("deleting event: {event_id}");
                                if let Ok(ApiResult { result, .. }) = crate::server::admin::event(
                                    crate::server::admin::EventAction::Delete { id: event_id.clone() } 
                                ).await && result == ResultStatus::Success {
                                    refresh.update(|n| *n += 1);
                                    deleting.set(false);
                                }
                            });
                        } else {
                            deleting.set(true);
                        }
                    }
                >{move || delete_submit_btn_text.get()}</button>
            </div>
        </div>
    }
}
