use crate::{server::{admin::AdminEventApi, enums::ResultStatus, structs::ApiResult}, utils::offset_to_naive};
use chrono::DateTime;
use leptos::{prelude::*, task::spawn_local, web_sys::Event};
use time::OffsetDateTime;

#[component]
pub fn Event(event: crate::server::db::structs::Event, refresh: RwSignal<i32>) -> impl IntoView {
    let id_signal = RwSignal::new(event.id.clone());
    let name_signal = RwSignal::new(event.name.clone());
    let description_signal = RwSignal::new(event.description.clone());
    let start_date_signal = RwSignal::new(event.start_date);
    let end_date_signal = RwSignal::new(event.end_date);

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
                <p class="text-lg/8"><b>"ID: "</b> {move || id_signal.get().clone()}</p>
                {move || {
                    if let Some(description) = description_signal.get() {
                        view! {
                            <p class="text-lg/8"><b>"Description: "</b>{description.clone()}</p>
                        }.into_any()
                    } else {
                        view! {}.into_any()
                    }
                }}
                //<time datetime=move || start_date_signal.get()></time>
                //<time datetime=move || end_date_signal.get()></time>
                <p class="text-lg/8"><b>"Start Date: "</b> {move || start_date_signal.get().to_string()}</p>
                <p class="text-lg/8"><b>"End Date: "</b> {move || end_date_signal.get().to_string()}</p>
            </Show>

            <Show when=move || editing.get()>
                <label class="block text-sm font-medium text-gray-700 mb-1">"Name"</label>
                <input class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" name="name" value=move || name_signal.get() bind:value=name_signal/>
                
                <label class="block text-sm font-medium text-gray-700 mb-1">"Description"</label>
                <input class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" name="description" value=move || description_signal.get() on:change=move |ev: Event| {
                    let value = event_target_value(&ev);
                    description_signal.set(Some(value));
                }/>
                
                <label class="block text-sm font-medium text-gray-700 mb-1">"Start Date"</label>
                <input class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" type="datetime-local" name="start_date" value=move || start_date_signal.get().to_string() on:change=move |ev: Event| {
                    let value = OffsetDateTime::from_event(&ev).unwrap();
                    start_date_signal.set(value);
                }/>
                
                <label class="block text-sm font-medium text-gray-700 mb-1">"End Date"</label>
                <input class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" type="datetime-local" name="end_date" value=move || end_date_signal.get().to_string() on:change=move |ev: Event| {
                    let value = OffsetDateTime::from_event(&ev).unwrap();
                    end_date_signal.set(value);
                }/>
            </Show>

            <div class="flex flex-row-reverse gap-3 mt-2">
                <Show when=move || editing.get() || deleting.get()>
                    <button 
                        class="px-4 py-2 rounded-md border border-gray-300 text-sm hover:bg-gray-50" 
                        on:click=move |_| {
                            spawn_local(async move {
                                editing.set(false);
                                deleting.set(false);
                            });
                        }
                    >"Cancel"</button>
                </Show>
                <button type="button" hidden=move || deleting.get() class="inline-flex items-center gap-2 rounded-lg text-white px-4 py-2 text-sm font-medium focus:outline-none focus:ring-2 active:scale-95 transition bg-indigo-600 hover:bg-indigo-700 focus:ring-indigo-400" on:click=move |_| {
                    if editing.get() {
                        spawn_local(async move {
                            tracing::debug!("editing event: {}", id_signal.get().clone());
                            let start_date_naive = offset_to_naive(start_date_signal.get());
                            let end_date_naive = offset_to_naive(end_date_signal.get());
                            if let Ok(ApiResult { result, .. }) = crate::server::admin::event(
                                crate::server::admin::EventAction::Edit { 
                                    id: id_signal.get().clone(), 
                                    name: name_signal.get().clone(), 
                                    description: description_signal.get().clone().unwrap_or_default(), 
                                    start_date: start_date_naive, 
                                    end_date: end_date_naive
                                }
                            ).await && result == ResultStatus::Success {
                                refresh.update(|n| *n += 1);
                            }
                        });
                        editing.set(false)
                    } else {
                        editing.set(true)
                    }
                }>{ move || edit_submit_btn_text.get() }</button>

                <button
                    hidden=move || editing.get()
                    class="ml-auto inline-flex items-center px-4 py-2 rounded-md bg-red-600 text-white text-sm font-semibold shadow-sm hover:bg-red-500 focus:outline-none focus:ring-2 focus:ring-indigo-500"
                    on:click=move |_| {
                        if deleting.get() {
                            let event_id = event.id.clone();
                            spawn_local(async move {
                                tracing::debug!("deleting event: {event_id}");
                                if let Ok(ApiResult { result, .. }) = crate::server::admin::event(
                                    crate::server::admin::EventAction::Delete { id: event_id.clone() } 
                                ).await && result == ResultStatus::Success {
                                    refresh.update(|n| *n += 1);
                                }
                            });
                            deleting.set(false);
                        } else {
                            deleting.set(true);
                        }
                    }
                >
                    { move || delete_submit_btn_text.get() }
                </button>
            </div>
        </div>
    }
}
