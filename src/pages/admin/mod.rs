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

#[derive(Clone, PartialEq)]
pub enum AdminSections {
    SiteSettings,
    Events,
    Challenges,
    Files,
    Users,
    Log
}

#[derive(Debug, Clone, PartialEq)]
pub enum Actions {
    Create,
    Delete,
    Edit,
    None
}

/// Default Home Page
#[component]
pub fn Admin() -> impl IntoView {
    let selected = RwSignal::new(AdminSections::SiteSettings);
    provide_context(selected);

    view! {
        <NavBar />
        <div class=r#"justify-center p-8 align-center bg-background text-text"#>
            <h3 class=r#"text-4xl text-center"#>"Admin"</h3>
            <div class=r#"grid grid-cols-5 gap-4 m-4"#>
                <AdminNavBar />
                <section class=r#"col-start-2 col-end-6 p-6 bg-background-secondary text-text rounded-lg shadow-sm main-panel"#>
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
        <nav class=r#"flex flex-col col-start-1 col-end-1 gap-2 p-4 bg-background-secondary text-text rounded-lg shadow-sm"#>
            <ul class=r#"flex flex-col gap-1"# role="menu" aria-label="Admin navigation">
                <li class="bg-background">
                    <p
                        class=r#"flex gap-3 items-center py-2 px-3 text-sm font-medium  
                        rounded-md hover:bg-gray-50 focus:ring-2 focus:outline-none 
                        focus:ring-yale-blue-500"#
                        on:click=move |_| selected.set(AdminSections::SiteSettings)
                    >
                        <Icon icon=i::LuSettings />
                        "Site Settings"
                    </p>
                </li>
                <li class="bg-background hover:bg-background-secondary">
                    <p
                        class=r#"flex gap-3 items-center py-2 px-3 text-sm font-medium text-gray-700 
                        rounded-md hover:bg-gray-50 focus:ring-2 focus:outline-none 
                        focus:ring-yale-blue-500"#
                        on:click=move |_| selected.set(AdminSections::Events)
                    >
                        <Icon icon=i::LuCalendarRange />
                        "Events"
                    </p>
                </li>
                <li class="bg-background hover:bg-background-secondary">
                    <p
                        class=r#"flex gap-3 items-center py-2 px-3 text-sm font-medium text-gray-700 
                        rounded-md hover:bg-gray-50 focus:ring-2 focus:outline-none 
                        focus:ring-yale-blue-500"#
                        on:click=move |_| selected.set(AdminSections::Challenges)
                    >
                        <Icon icon=i::MdiBullseyeArrow />
                        "Challenges"
                    </p>
                </li>
                <li class="bg-background">
                    <p
                        class=r#"flex gap-3 items-center py-2 px-3 text-sm font-medium text-gray-700 
                        rounded-md hover:bg-gray-50 focus:ring-2 focus:outline-none 
                        focus:ring-yale-blue-500"#
                        on:click=move |_| selected.set(AdminSections::Files)
                    >
                        <Icon icon=i::LuFiles />
                        "Files"
                    </p>
                </li>
                <li class="bg-background">
                    <p
                        class=r#"flex gap-3 items-center py-2 px-3 text-sm font-medium text-gray-700 
                        rounded-md hover:bg-gray-50 focus:ring-2 focus:outline-none 
                        focus:ring-yale-blue-500"#
                        on:click=move |_| selected.set(AdminSections::Users)
                    >
                        <Icon icon=i::LuUsers />
                        "Users"
                    </p>
                </li>
                <li class="bg-background">
                    <p
                        class=r#"flex gap-3 items-center py-2 px-3 text-sm font-medium text-gray-700 
                        rounded-md hover:bg-gray-50 focus:ring-2 focus:outline-none 
                        focus:ring-yale-blue-500"#
                        on:click=move |_| selected.set(AdminSections::Log)
                    >
                        <Icon icon=i::LuLogs />
                        "Log"
                    </p>
                </li>
            </ul>
        </nav>
    }
}
