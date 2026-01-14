use crate::server::{db::enums::UserRole, get_user, logout_user};
use icondata as i;
use leptos::{prelude::*, task::spawn_local};
use leptos_icons::Icon;
use leptos_router::hooks::use_navigate;

#[component]
pub fn NavBar() -> impl IntoView {
    let open = RwSignal::new(false);
    let user = RwSignal::new(None);

    let user_resource = Resource::new(move || (), move |_| async move {
        get_user().await.unwrap_or(None)
    });

    // effects arent actually intended to synchronize with the reactive system,
    // find another solution
    Effect::new(move |_| {
        let user_result = user_resource.get().unwrap_or_default();
        user.set(user_result);
    });

    let role = Memo::new(move |_| {
        if let Some(user) = user.get() { user.role } else { UserRole::Competitor }
    });

    let username = Memo::new(move |_| {
        if let Some(user) = user.get() { user.username } else { "".to_string() }
    });

    let points = Memo::new(move |_| {
        if let Some(user) = user.get() { user.points } else { 0_u32 }
    });

    view! {
        <div class="flex top-0 w-full items-center bg-white/25 shadow-sm p-4">
            <div class="flex-1"></div>

            <nav class="flex items-center justify-center">
                <ul class="flex items-center gap-6 list-none p-0 m-0">
                    <li class="flex items-center gap-2">
                        <a href="/" class="inline-flex items-center gap-2 m-1">
                            <Icon icon=i::LuHouse />
                            "Home"
                        </a>
                    </li>
                    <li class="flex items-center gap-2">
                        <a href="/challenges" class="inline-flex items-center gap-2 m-1">
                            <Icon icon=i::MdiBullseyeArrow />
                            "Challenges"
                        </a>
                    </li>
                    <li class="flex items-center gap-2">
                        <a href="/leaderboard" class="inline-flex items-center gap-2 m-1">
                            <Icon icon=i::LuChartLine />
                            "Leaderboard"
                        </a>
                    </li>
                    <Transition fallback=move || view! { <p>"Loading..."</p> }>
                        <Show when=move || user.get().is_some() && role.get() == UserRole::Admin>
                            <a href="/admin" class="inline-flex items-center gap-2 m-1">
                                <Icon icon=i::LuSettings />
                                "Admin"
                            </a>
                        </Show>
                    </Transition>
                </ul>
            </nav>

            <nav class="flex-1 flex justify-end items-center p-2 gap-2">
                <ul class="flex items-center gap-4 list-none p-0 m-0">
                    <Transition fallback=move || view! { <p>"Loading..."</p> }>
                        <Show when=move || user.get().is_some()>
                            <li class="flex items-center gap-2">
                                <a class="inline-flex items-center gap-2 m-1 cursor-pointer" on:click=move |_| {
                                    open.set(!open.get());
                                }>
                                    <Icon icon=i::LuUser />
                                    {move || username.get()}
                                </a>
                            </li>
                            <b>"Points: "{move || points.get()}</b>
                        </Show>

                        <Show when=move || user.get().is_none()>
                            <li class="flex items-center gap-2">
                                <a href="/login" class="inline-flex items-center gap-2 m-1">
                                    <Icon icon=i::LuLogIn />
                                    "Login"
                                </a>
                            </li>
                            <li class="flex items-center gap-2">
                                <a href="/register" class="inline-flex items-center gap-2 m-1">
                                    <Icon icon=i::LuUserPlus />
                                    "Register"
                                </a>
                            </li>
                        </Show>
                    </Transition>

                    <Show when=move || open.get() fallback=|| ()>
                        <nav class="flex-col fixed bg-white/25 rounded-md p-4 z-50 mt-[15rem] shadow-sm" on:blur=move |_| {
                            open.set(false)
                        }>
                            <ul class="flex flex-col items-center gap-4">
                                <li class="w-full">
                                    <a href=move || format!("/profile/{}", username.get())>
                                        <Icon icon=i::LuCircleUser />
                                        "Profile"
                                    </a>
                                </li>
                                <li class="w-full">
                                    <a href="/settings">
                                        <Icon icon=i::LuUserCog />
                                        "Settings"
                                    </a>
                                </li>
                                <li class="w-full">
                                    <button
                                        class="cursor-pointer" 
                                        on:click=move |_| {
                                        spawn_local(async move {
                                            if let Ok(()) = logout_user().await {
                                                let nav = use_navigate();
                                                nav("/", Default::default());
                                            }
                                        });
                                    }>
                                        <Icon icon=i::LuLogOut />
                                        "Logout"
                                    </button>
                                </li>
                            </ul>
                        </nav>
                    </Show>
                </ul>
            </nav>
        </div>
    }
}
