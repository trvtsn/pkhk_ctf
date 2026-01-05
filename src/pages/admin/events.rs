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
        <button class="border-2 border-black p-2 text-black rounded" on:click=move |_| section.set(Actions::Create)>"Create"</button>
        <button class="border-2 border-black p-2 text-black rounded" on:click=move |_| section.set(Actions::Delete)>"Delete"</button>
        <button class="border-2 border-black p-2 text-black rounded" on:click=move |_| section.set(Actions::Edit)>"Edit"</button>

        <Show when=move || section.get() == Actions::Create>
            <ActionForm action=event_action>
                <label>
                    <b>"Name"</b>
                    <input class="bg-white border" name="action[create][name]" />
                </label>
                <label>
                    <b>"Description"</b>
                    <input class="bg-white border" name="action[create][description]" />
                </label>
                <label>
                    <b>"Start Date"</b>
                    <input class="bg-white border" type="date" name="action[create][start_date]" />
                </label>
                <label>
                    <b>"End Date"</b>
                    <input class="bg-white border" type="date" name="action[create][end_date]" />
                </label>
                //<button loading=loading on_click=move |_| { loading.set(true) }>
                <input
                    type="submit"
                    class=r#"flex w-full justify-center rounded-md bg-indigo-600 px-3 py-1.5 text-sm font-semibold
                        leading-6 text-white shadow-sm hover:bg-indigo-500 focus-visible:outline 
                        focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600"#
                    value="Create"
                />
            </ActionForm>
        </Show>

        <Show when=move || section.get() == Actions::Delete>
            <ActionForm action=event_action>
                <label>
                    <b>"Event ID"</b>
                    <input class="bg-white border" type="number" name="action[delete][id]" />
                </label>
                //<button loading=loading on_click=move |_| { loading.set(true) }>
                <input
                    type="submit"
                    class=r#"flex w-full justify-center rounded-md bg-indigo-600 px-3 py-1.5 text-sm font-semibold
                        leading-6 text-white shadow-sm hover:bg-indigo-500 focus-visible:outline 
                        focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600"#
                    value="Delete"
                />
            </ActionForm>
        </Show>

        <Show when=move || section.get() == Actions::Edit>
            <ActionForm action=event_action>
                <label>
                    <b>"ID"</b>
                    <input class="bg-white border" type="number" name="action[edit][id]" />
                </label>
                <label>
                    <b>"Name"</b>
                    <input class="bg-white border" name="action[edit][name]" />
                </label>
                <label>
                    <b>"Description"</b>
                    <input class="bg-white border" name="action[edit][description]" />
                </label>
                <label>
                    <b>"Start Date"</b>
                    <input class="bg-white border" type="date" name="action[edit][start_date]" />
                </label>
                <label>
                    <b>"End Date"</b>
                    <input class="bg-white border" type="date" name="action[edit][end_date]" />
                </label>
                //<button loading=loading on_click=move |_| { loading.set(true) }>
                <input
                    type="submit"
                    class=r#"flex w-full justify-center rounded-md bg-indigo-600 px-3 py-1.5 text-sm font-semibold
                        leading-6 text-white shadow-sm hover:bg-indigo-500 focus-visible:outline 
                        focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600"#
                    value="Edit"
                />
            </ActionForm>
        </Show>

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
