pub mod challenges;
pub mod events;
pub mod log;
pub mod site_settings;
pub mod users;
pub mod files;

use super::admin::{challenges::Challenges, events::Events, log::Log, site_settings::SiteSettings, users::Users, files::Files};
use crate::{components::navbar::NavBar};
use icondata as i;
use leptos::prelude::*;
use leptos_icons::Icon;
// use std::fmt::Display;
// use thaw::*;

#[derive(Clone, PartialEq)]
pub enum AdminSections {
    SiteSettings,
    Events,
    Challenges,
    Files,
    Users,
    Log
}

/// Default Home Page
#[component]
pub fn Admin() -> impl IntoView {
    let selected = RwSignal::new(AdminSections::SiteSettings);
    provide_context(selected);

    view! {
        <NavBar />
        <div class="p-8 justify-center align-center">
            <h3 class="text-4xl text-center">"Admin"</h3>
            <div class="grid grid-cols-5 gap-4 m-4">
                <AdminNavBar />
                <section class="main-panel col-start-2 col-end-6 p-6 bg-white rounded-lg shadow-sm">
                    {move || {
                        view! {
                            <Show when=move || selected.get() == AdminSections::SiteSettings>
                                <SiteSettings />
                            </Show>

                            <Show when=move || selected.get() == AdminSections::Events>
                                <Events />
                            </Show>

                            <Show when=move || selected.get() == AdminSections::Challenges>
                                <Challenges />
                            </Show>

                            <Show when=move || selected.get() == AdminSections::Files>
                                <Files />
                            </Show>

                            <Show when=move || selected.get() == AdminSections::Users>
                                <Users />
                            </Show>

                            <Show when=move || selected.get() == AdminSections::Log>
                                <Log />
                            </Show>
                        }
                    }}
                </section>
            </div>
        </div>
    }
}

#[component]
pub fn AdminNavBar() -> impl IntoView {
    let selected = use_context::<RwSignal<AdminSections>>().expect("to have found the setter provided");

    view! {
        <nav class="col-start-1 col-end-1 p-4 bg-white rounded-lg shadow-sm flex flex-col gap-2">
            <ul class="flex flex-col gap-1" role="menu" aria-label="Admin navigation">
                <li>
                    <p 
                        class="flex items-center gap-3 px-3 py-2 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-indigo-500"
                        on:click=move |_| selected.set(AdminSections::SiteSettings)>
                        <Icon icon=i::LuSettings/>
                        "Site Settings"
                    </p>
                </li>
                <li>
                    <p 
                        class="flex items-center gap-3 px-3 py-2 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-indigo-500" 
                        on:click=move |_| selected.set(AdminSections::Events)>
                        <Icon icon=i::LuCalendarRange/>
                        "Events"
                    </p>
                </li>
                <li>
                    <p 
                        class="flex items-center gap-3 px-3 py-2 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-indigo-500" 
                        on:click=move |_| selected.set(AdminSections::Challenges)>
                        <Icon icon=i::MdiBullseyeArrow/>
                        "Challenges"
                    </p>
                </li>
                <li>
                    <p 
                        class="flex items-center gap-3 px-3 py-2 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-indigo-500" 
                        on:click=move |_| selected.set(AdminSections::Files)>
                        <Icon icon=i::LuFiles/>
                        "Files"
                    </p>
                </li>
                <li>
                    <p 
                        class="flex items-center gap-3 px-3 py-2 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-indigo-500" 
                        on:click=move |_| selected.set(AdminSections::Users)>
                        <Icon icon=i::LuUsers/>
                        "Users"
                    </p>
                </li>
                <li>
                    <p 
                        class="flex items-center gap-3 px-3 py-2 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-indigo-500" 
                        on:click=move |_| selected.set(AdminSections::Log)>
                        <Icon icon=i::LuLogs/>
                        "Log"
                    </p>
                </li>
            </ul>
        </nav>
    }
}
