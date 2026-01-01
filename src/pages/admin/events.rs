// use super::AdminNavBar;
// use crate::components::navbar::NavBar;
use chrono::NaiveTime;
use leptos::prelude::*;

use crate::server::admin::AdminEventApi;

pub struct Event {
    pub title: String,
    pub description: String,
    pub start_date: NaiveTime,
    pub end_date: NaiveTime,
}

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

    view! {
        <div class="container p-8 inline justify-center">
            <button class="border-2 border-black p-2 text-black rounded" on:click=move |_| section.set(Actions::Create)>"Create"</button>
            <button class="border-2 border-black p-2 text-black rounded" on:click=move |_| section.set(Actions::Delete)>"Delete"</button>
            <button class="border-2 border-black p-2 text-black rounded" on:click=move |_| section.set(Actions::Edit)>"Edit"</button>

            <section class="main-panel">
                <Show when=move || section.get() == Actions::Create>
                    <ActionForm action=event_action>
                        <label>
                            <b>"Name"</b>
                            <input class="bg-white border" name="action[Create][name]" />
                        </label>
                        <label>
                            <b>"Description"</b>
                            <input class="bg-white border" name="action[Create][description]" />
                        </label>
                        <label>
                            <b>"Start Date"</b>
                            <input class="bg-white border" type="date" name="action[Create][start_date]" />
                        </label>
                        <label>
                            <b>"End Date"</b>
                            <input class="bg-white border" type="date" name="action[Create][end_date]" />
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
                    "Delete"
                    <ActionForm action=event_action>
                        <label>
                            <b>"Event ID"</b>
                            <input class="bg-white border" type="number" name="action[Delete][id]" />
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
                    "Edit"
                    <ActionForm action=event_action>
                        <label>
                            <b>"ID"</b>
                            <input class="bg-white border" type="number" name="action[Edit][id]" />
                        </label>
                        <label>
                            <b>"Name"</b>
                            <input class="bg-white border" name="action[Edit][name]" />
                        </label>
                        <label>
                            <b>"Description"</b>
                            <input class="bg-white border" name="action[Edit][description]" />
                        </label>
                        <label>
                            <b>"Start Date"</b>
                            <input class="bg-white border" type="date" name="action[Edit][start_date]" />
                        </label>
                        <label>
                            <b>"End Date"</b>
                            <input class="bg-white border" type="date" name="action[Edit][end_date]" />
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
            </section>
        </div>
    }
}
