pub mod challenges;
pub mod events;
pub mod files;
pub mod ldap;
pub mod log;
pub mod proxmox;
pub mod site_settings;
pub mod users;

use super::admin::{challenges::Challenges, events::Events, log::Log, site_settings::SiteSettings, users::Users, files::Files};
use crate::{components::navbar::NavBar, pages::admin::{ldap::Ldap, proxmox::Proxmox}};
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
    Log,
    Ldap,
    Proxmox
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
        <div class=r#"justify-center p-8 align-center bg-background text-text min-h-screen"#>
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

                            <Show when=move || selected.get() == AdminSections::Ldap>
                                <Ldap />
                            </Show>

                            <Show when=move || selected.get() == AdminSections::Proxmox>
                                <Proxmox />
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
    let selected = expect_context::<RwSignal<AdminSections>>();

    view! {
        <nav class=r#"flex flex-col col-start-1 col-end-1 gap-2 p-4 bg-background-secondary text-text rounded-lg shadow-sm"#>
            <ul class=r#"flex flex-col gap-1"# role="menu" aria-label="Admin navigation">
                <li 
                    class=move || {
                        if selected.get() == AdminSections::SiteSettings { 
                            "bg-background-hover hover:bg-background-hover" 
                        } else { 
                            "bg-background hover:bg-background-hover" 
                        }
                    }
                >
                    <p
                        class=r#"flex gap-3 items-center py-2 px-3 text-sm font-medium  
                        rounded-md focus:ring-2 focus:outline-none 
                        focus:ring-yale-blue-500 hover:text-hover"#
                        on:click=move |_| selected.set(AdminSections::SiteSettings)
                    >
                        <Icon icon=i::LuSettings />
                        "Site Settings"
                    </p>
                </li>
                <li 
                    class=move || {
                        if selected.get() == AdminSections::Events { 
                            "bg-background-hover hover:bg-background-hover" 
                        } else { 
                            "bg-background hover:bg-background-hover" 
                        }
                    }
                >
                    <p
                        class=r#"flex gap-3 items-center py-2 px-3 text-sm font-medium text-text 
                        rounded-md focus:ring-2 focus:outline-none 
                        focus:ring-yale-blue-500 hover:text-hover"#
                        on:click=move |_| selected.set(AdminSections::Events)
                    >
                        <Icon icon=i::LuCalendarRange />
                        "Events"
                    </p>
                </li>
                <li 
                    class=move || {
                        if selected.get() == AdminSections::Challenges { 
                            "bg-background-hover hover:bg-background-hover" 
                        } else { 
                            "bg-background hover:bg-background-hover" 
                        }
                    }
                >
                    <p
                        class=r#"flex gap-3 items-center py-2 px-3 text-sm font-medium text-text 
                        rounded-md focus:ring-2 focus:outline-none 
                        focus:ring-yale-blue-500 hover:text-hover"#
                        on:click=move |_| selected.set(AdminSections::Challenges)
                    >
                        <Icon icon=i::MdiBullseyeArrow />
                        "Challenges"
                    </p>
                </li>
                <li 
                    class=move || {
                        if selected.get() == AdminSections::Files { 
                            "bg-background-hover hover:bg-background-hover" 
                        } else { 
                            "bg-background hover:bg-background-hover" 
                        }
                    }
                >
                    <p
                        class=r#"flex gap-3 items-center py-2 px-3 text-sm font-medium text-text 
                        rounded-md focus:ring-2 focus:outline-none 
                        focus:ring-yale-blue-500 hover:text-hover"#
                        on:click=move |_| selected.set(AdminSections::Files)
                    >
                        <Icon icon=i::LuFiles />
                        "Files"
                    </p>
                </li>
                <li 
                    class=move || {
                        if selected.get() == AdminSections::Users { 
                            "bg-background-hover hover:bg-background-hover" 
                        } else { 
                            "bg-background hover:bg-background-hover" 
                        }
                    }
                >
                    <p
                        class=r#"flex gap-3 items-center py-2 px-3 text-sm font-medium text-text 
                        rounded-md focus:ring-2 focus:outline-none 
                        focus:ring-yale-blue-500 hover:text-hover"#
                        on:click=move |_| selected.set(AdminSections::Users)
                    >
                        <Icon icon=i::LuUsers />
                        "Users"
                    </p>
                </li>
                <li 
                    class=move || {
                        if selected.get() == AdminSections::Ldap { 
                            "bg-background-hover hover:bg-background-hover" 
                        } else { 
                            "bg-background hover:bg-background-hover" 
                        }
                    }
                >
                    <p
                        class=r#"flex gap-3 items-center py-2 px-3 text-sm font-medium text-text 
                        rounded-md focus:ring-2 focus:outline-none 
                        focus:ring-yale-blue-500 hover:text-hover"#
                        on:click=move |_| selected.set(AdminSections::Ldap)
                    >
                        <Icon icon=i::LuUserLock />
                        "LDAP"
                    </p>
                </li>
                <li 
                    class=move || {
                        if selected.get() == AdminSections::Proxmox { 
                            "bg-background-hover hover:bg-background-hover" 
                        } else { 
                            "bg-background hover:bg-background-hover" 
                        }
                    }
                >
                    <p
                        class=r#"flex gap-3 items-center py-2 px-3 text-sm font-medium text-text 
                        rounded-md focus:ring-2 focus:outline-none 
                        focus:ring-yale-blue-500 hover:text-hover"#
                        on:click=move |_| selected.set(AdminSections::Proxmox)
                    >
                        <Icon icon=i::LuServer />
                        "Proxmox"
                    </p>
                </li>
                <li 
                    class=move || {
                        if selected.get() == AdminSections::Log { 
                            "bg-background-hover hover:bg-background-hover" 
                        } else { 
                            "bg-background hover:bg-background-hover" 
                        }
                    }
                >
                    <p
                        class=r#"flex gap-3 items-center py-2 px-3 text-sm font-medium text-text 
                        rounded-md focus:ring-2 focus:outline-none 
                        focus:ring-yale-blue-500 hover:text-hover"#
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
