// use super::AdminNavBar;
// use crate::components::navbar::NavBar;
use chrono::NaiveTime;
use leptos::prelude::*;

use crate::{components::admin::event::Event, server::{admin::{AdminEventApi, get_all_events}, db}};

#[derive(Debug, Clone, PartialEq)]
pub enum Actions {
    Create,
    Delete,
    Edit
}

/// Default Home Page
#[component]
pub fn Events() -> impl IntoView {
    let section = RwSignal::new(Actions::Create);
    let event_action = ServerAction::<AdminEventApi>::new();

    let events = Resource::new(move || (), move |_| async move {
        match get_all_events().await {
            Ok(events) => Ok(events),
            Err(e) => Err(e)
        }
    });

    view! {
        <div class="flex gap-2 mb-4">
            <button class="border border-gray-300 px-3 py-1 rounded-md text-sm hover:bg-gray-50" on:click=move |_| section.set(Actions::Create)>"Create"</button>
            <button class="border border-gray-300 px-3 py-1 rounded-md text-sm hover:bg-gray-50" on:click=move |_| section.set(Actions::Delete)>"Delete"</button>
            <button class="border border-gray-300 px-3 py-1 rounded-md text-sm hover:bg-gray-50" on:click=move |_| section.set(Actions::Edit)>"Edit"</button>
        </div>

        <div class="flex flex-col gap-4">
            <Show when=move || section.get() == Actions::Create>
                <ActionForm action=event_action>
                    <label class="block text-sm font-medium text-gray-700 mb-1">"Name"</label>
                    <input class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" name="action[create][name]" />

                    <label class="block text-sm font-medium text-gray-700 mb-1">"Description"</label>
                    <input class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" name="action[create][description]" />

                    <label class="block text-sm font-medium text-gray-700 mb-1">"Start Date"</label>
                    <input class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" type="date" name="action[create][start_date]" />
                
                    <label class="block text-sm font-medium text-gray-700 mb-1">"End Date"</label>
                    <input class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" type="date" name="action[create][end_date]" />
                    
                    //<button loading=loading on_click=move |_| { loading.set(true) }>
                    <div class="flex gap-3 mt-2">
                        <button type="button" class="px-4 py-2 rounded-md border border-gray-300 text-sm hover:bg-gray-50">"Cancel"</button>
                        <input
                            type="submit"
                            class="ml-auto inline-flex items-center px-4 py-2 rounded-md bg-indigo-600 text-white text-sm font-semibold shadow-sm hover:bg-indigo-500 focus:outline-none focus:ring-2 focus:ring-indigo-500"
                            value="Create"
                        />
                    </div>
                </ActionForm>
            </Show>

            <Show when=move || section.get() == Actions::Delete>
                <ActionForm action=event_action>
                    <label class="block text-sm font-medium text-gray-700 mb-1">"Event ID"</label>
                    <input class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" type="number" name="action[delete][id]" />
                    
                    <div class="flex gap-3 mt-2">
                        <button type="button" class="px-4 py-2 rounded-md border border-gray-300 text-sm hover:bg-gray-50">"Cancel"</button>
                        <input
                            type="submit"
                            class="ml-auto inline-flex items-center px-4 py-2 rounded-md bg-indigo-600 text-white text-sm font-semibold shadow-sm hover:bg-indigo-500 focus:outline-none focus:ring-2 focus:ring-indigo-500"
                            value="Delete"
                        />
                    </div>
                </ActionForm>
            </Show>

            <Show when=move || section.get() == Actions::Edit>
                <ActionForm action=event_action>
                    <label class="block text-sm font-medium text-gray-700 mb-1">"ID"</label>
                    <input class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" type="number" name="action[edit][id]" />
                    
                    <label class="block text-sm font-medium text-gray-700 mb-1">"Name"</label>
                    <input class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" name="action[edit][name]" />
                    
                    <label class="block text-sm font-medium text-gray-700 mb-1">"Description"</label>
                    <input class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" name="action[edit][description]" />
                    
                    <label class="block text-sm font-medium text-gray-700 mb-1">"Start Date"</label>
                    <input class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" type="date" name="action[edit][start_date]" />
                    
                    <label class="block text-sm font-medium text-gray-700 mb-1">"End Date"</label>
                    <input class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" type="date" name="action[edit][end_date]" />
                    
                    <div class="flex gap-3 mt-2">
                        <button type="button" class="px-4 py-2 rounded-md border border-gray-300 text-sm hover:bg-gray-50">"Cancel"</button>
                        <input
                            type="submit"
                            class="ml-auto inline-flex items-center px-4 py-2 rounded-md bg-indigo-600 text-white text-sm font-semibold shadow-sm hover:bg-indigo-500 focus:outline-none focus:ring-2 focus:ring-indigo-500"
                            value="Edit"
                        />
                    </div>
                </ActionForm>
            </Show>
        </div>

        <div class="events">
            <Suspense fallback=move || view! { <div>"Loading..."</div> }>
                {move || {
                    let events = events.get().map(move |result| match result {
                        Ok(events) => {
                            view! {
                                <div class="m-4 grid grid-cols-4 content-stretch">
                                    <For
                                        each=move || events.clone()
                                        key=|event: &db::structs::Event| event.id
                                        let(event)
                                    >
                                        <div class="event p-2">
                                            <Event event/>
                                        </div>
                                    </For>
                                </div>
                            }.into_any()
                        }
                        Err(e) => {
                            view! {
                                <div class="challenge p-2">
                                    <p>"Bruh" {e.to_string()}</p>
                                </div>
                            }.into_any()
                        }
                    })
                    .collect_view()
                    .into_any();

                    view! {
                        {events}
                    }
                }}
            </Suspense>
        </div>
    }
}
