pub mod challenges;
pub mod events;
pub mod log;
pub mod site_settings;
pub mod users;

use super::admin::{challenges::Challenges, events::Events, log::Log, site_settings::SiteSettings, users::Users};
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
        <div class="container p-8 inline justify-center">
            <h3 class="text-4xl text-center">"Admin"</h3>
            <AdminNavBar />
            <section class="main-panel">
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
    }
}

#[component]
pub fn AdminNavBar() -> impl IntoView {
    let selected = use_context::<RwSignal<AdminSections>>().expect("to have found the setter provided");

    view! {
        <div class="container">
            <nav class="flex-col p-2">
                <p class="p-2 border-2 border-black" on:click=move |_| selected.set(AdminSections::SiteSettings)>"Site Settings"</p>
                <p class="p-2 border-2 border-black" on:click=move |_| selected.set(AdminSections::Events)>"Events"</p>
                <p class="p-2 border-2 border-black" on:click=move |_| selected.set(AdminSections::Challenges)>"Challenges"</p>
                <p class="p-2 border-2 border-black" on:click=move |_| selected.set(AdminSections::Users)>"Users"</p>
                <p class="p-2 border-2 border-black" on:click=move |_| selected.set(AdminSections::Log)>"Log"</p>
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
        </div>
    }
}
