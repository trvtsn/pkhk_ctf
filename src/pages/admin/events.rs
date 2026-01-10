use crate::{components::admin::event::Event, server::{admin::{AdminEventApi, get_all_events}, db, enums::ResultStatus, structs::ApiResult}, utils::offset_to_naive};
use leptos::{prelude::*, task::{spawn, spawn_local}, web_sys::Event};
use time::OffsetDateTime;

#[derive(Debug, Clone, PartialEq)]
pub enum Actions {
    Create,
    Delete,
    Edit,
    None
}

/// Default Home Page
#[component]
pub fn Events() -> impl IntoView {
    let section = RwSignal::new(Actions::None);
    let event_action = ServerAction::<AdminEventApi>::new();
    let refresh = RwSignal::new(0);
    
    let events = Resource::new(move || refresh.get(), move |_| async move {
        get_all_events().await.unwrap_or_default()
    });

    let creating = RwSignal::new(false);

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
                <ActionForm action=event_action>
                    <label class="block text-sm font-medium text-gray-700 mb-1">"Name"</label>
                    <input class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" name="action[create][name]"/>

                    <label class="block text-sm font-medium text-gray-700 mb-1">"Description"</label>
                    <input class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" name="action[create][description]"/>

                    <label class="block text-sm font-medium text-gray-700 mb-1">"Start Date"</label>
                    <input class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" type="datetime-local" name="action[create][start_date]"/>
                
                    <label class="block text-sm font-medium text-gray-700 mb-1">"End Date"</label>
                    <input class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" type="datetime-local" name="action[create][end_date]"/>
                    
                    <div class="flex gap-3 mt-2">
                        <button type="button" class="px-4 py-2 rounded-md border border-gray-300 text-sm hover:bg-gray-50" on:click=move |_| {section.set(Actions::None)}>"Cancel"</button>
                        <input
                            type="submit"
                            class="ml-auto inline-flex items-center px-4 py-2 rounded-md bg-indigo-600 text-white text-sm font-semibold shadow-sm hover:bg-indigo-500 focus:outline-none focus:ring-2 focus:ring-indigo-500"
                            value="Create"
                        />
                    </div>
                </ActionForm>
            </Show>
        </div>

        <div class="events">
            <Suspense fallback=move || view! { <div>"Loading..."</div> }>
                {move || {
                    events.get().map(move |events| {
                        view! {
                            <div class="m-4 grid grid-cols-4 content-stretch">
                                <For
                                    each=move || events.clone()
                                    key=|event: &db::structs::Event| event.id.clone()
                                    let(event)
                                >
                                    <div class="event p-2">
                                        <Event event refresh/>
                                    </div>
                                </For>
                            </div>
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}
