pub mod challenges;
pub mod events;
pub mod log;
pub mod site_settings;
pub mod users;
pub mod files;

use super::admin::{challenges::Challenges, events::Events, log::Log, site_settings::SiteSettings, users::Users, files::Files};
use crate::{components::navbar::NavBar};
// use axum::{response::IntoResponse, Router, routing::get};
// use axum_login::login_required;
use leptos::prelude::*;
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
                    <p class="flex items-center gap-3 px-3 py-2 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-indigo-500" on:click=move |_| selected.set(AdminSections::SiteSettings)>"Site Settings"</p>
                </li>
                <li>
                    <p class="flex items-center gap-3 px-3 py-2 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-indigo-500" on:click=move |_| selected.set(AdminSections::Events)>"Events"</p>
                </li>
                <li>
                    <p class="flex items-center gap-3 px-3 py-2 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-indigo-500" on:click=move |_| selected.set(AdminSections::Challenges)>"Challenges"</p>
                </li>
                <li>
                    <p class="flex items-center gap-3 px-3 py-2 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-indigo-500" on:click=move |_| selected.set(AdminSections::Files)>"Files"</p>
                </li>
                <li>
                    <p class="flex items-center gap-3 px-3 py-2 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-indigo-500" on:click=move |_| selected.set(AdminSections::Users)>"Users"</p>
                </li>
                <li>
                    <p class="flex items-center gap-3 px-3 py-2 rounded-md text-sm font-medium text-gray-700 hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-indigo-500" on:click=move |_| selected.set(AdminSections::Log)>"Log"</p>
                </li>
            </ul>
        </nav>
        // <NavDrawer>
        //     <NavItem
        //         value="Site Settings"
        //         on:click=move |_| selected.set("Site Settings".to_string())
        //     >
        //         <p>"Site Settings"</p>
        //     </NavItem>
        //     <NavItem value="Events" on:click=move |_| selected.set("Events".to_string())>
        //         <p>"Events"</p>
        //     </NavItem>
        //     <NavItem
        //         value="Challenges"
        //         on:click=move |_| selected.set("Challenges".to_string())
        //     >
        //         <p>"Challenges"</p>
        //     </NavItem>
        //     <NavItem value="Log" on:click=move |_| selected.set("Log".to_string())>
        //         <p>"Log"</p>
        //     </NavItem>
        // </NavDrawer>
    }
}
