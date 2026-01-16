use crate::{components::admin::event::Event, pages::admin::Actions, server::{admin::get_all_events, db, enums::ResultStatus, structs::ApiResult}, utils::html_local_to_datetime};
use leptos::{prelude::*, task:: spawn_local};

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
    
    let events_resource = Resource::new(move || refresh.get(), move |_| async move {
        get_all_events().await.unwrap_or_default()
    });

    view! {
        <div class="flex gap-2 mb-4">
            <button class="border border-gray-300 px-3 py-1 rounded-md text-sm hover:bg-gray-50" on:click=move |_| {
                if creating.get() {
                    creating.set(false);
                    section.set(Actions::None);
                } else {
                    creating.set(true);
                    section.set(Actions::Create);
                }
            }>"Create"</button>
        </div>

        <div class="flex flex-col gap-4">
            <Show when=move || section.get() == Actions::Create>
                <label class="block text-sm font-medium text-gray-700 mb-1">"Name"</label>
                <input 
                    class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" 
                    name="name" 
                    bind:value=name_signal
                />

                <label class="block text-sm font-medium text-gray-700 mb-1">"Description"</label>
                <input 
                    class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" 
                    name="description" bind:value=description_signal
                />

                <label class="block text-sm font-medium text-gray-700 mb-1">"Start Date"</label>
                <input 
                    class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" 
                    type="datetime-local" 
                    name="start_at" 
                    bind:value=start_at_signal
                />
            
                <label class="block text-sm font-medium text-gray-700 mb-1">"End Date"</label>
                <input 
                    class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" 
                    type="datetime-local" 
                    name="end_at" 
                    bind:value=end_at_signal
                />
                
                <div class="flex gap-3 mt-2">
                    <button 
                        type="button" 
                        class="px-4 py-2 rounded-md border border-gray-300 text-sm hover:bg-gray-50" 
                        on:click=move |_| {section.set(Actions::None)}
                    >"Cancel"</button>
                    <button
                        type="button"
                        class=r#"ml-auto inline-flex items-center px-4 py-2 rounded-md bg-indigo-600 text-white text-sm font-semibold 
                        shadow-sm hover:bg-indigo-500 focus:outline-none focus:ring-2 focus:ring-indigo-500"#
                        on:click=move |_| {
                            let name = name_signal.get();
                            let description = description_signal.get();
                            let start_at = html_local_to_datetime(start_at_signal.get());
                            let end_at = html_local_to_datetime(end_at_signal.get());
                            spawn_local(async move {
                                tracing::debug!("creating event...");
                                if let Ok(ApiResult { result, .. }) = crate::server::admin::event(
                                    crate::server::admin::EventAction::Create { name, description, start_at, end_at }
                                ).await && result == ResultStatus::Success {
                                    refresh.update(|n| *n += 1);
                                }
                            });
                        }
                    >"Create"</button>
                </div>
            </Show>
        </div>

        <div class="events">
            <div class="m-4 grid grid-cols-4 content-stretch">
                <Transition fallback=move || view! { <div>"Loading..."</div> }>
                    <For
                        each=move || events_resource.get().unwrap_or_default()
                        key=|event: &db::structs::Event| event.id.clone()
                        let(event)
                    >
                        <div class="event p-2">
                            <Event event refresh/>
                        </div>
                    </For>
                </Transition>
            </div>
        </div>
    }
}
