use crate::server::{db::enums::UserRole, get_user, get_user_points, logout_user};
use icondata as i;
use leptos::{prelude::*, task::spawn_local};
use leptos_icons::Icon;
use leptos_router::hooks::use_navigate;

#[component]
pub fn NavBar() -> impl IntoView {
    let open = RwSignal::new(false);
    let user = Resource::new(move || (), async move |_| {
        get_user().await.unwrap_or(None)
    });
    let user_points = Resource::new(move || (), async move |_| {
        get_user_points().await
    });
    let username = RwSignal::new("".to_string());
    let user_profile_path = RwSignal::new("".to_string());

    view! {
        <div class="flex top-0 w-full items-center bg-lavender-blush-50/25 border-b p-4">
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
                    <Suspense fallback=move || view! { <p>"Loading..."</p> }>
                        {move || user.get().map(|user| match user {
                            Some(user) => {
                                view! {
                                    <Show when=move || user.role == UserRole::Admin >
                                        <a href="/admin" class="inline-flex items-center gap-2 m-1">
                                            <Icon icon=i::LuSettings />
                                            "Admin"
                                        </a>
                                    </Show>
                                }.into_any()
                            }
                            None => {
                                view! {}.into_any()
                            }
                        })}
                    </Suspense>
                </ul>
            </nav>

            <nav class="flex-1 flex justify-end items-center p-2 gap-2">
                <ul class="flex items-center gap-4 list-none p-0 m-0">
                    <Suspense fallback=move || view! { <p>"Loading..."</p> }>
                        {move || user.get().map(|j| match j {
                            Some(user) => {
                                username.set(user.username);
                                user_profile_path.set(format!("/profile/{}", username.get()));
                                view! {
                                    <li class="flex items-center gap-2">
                                        <a
                                            class="inline-flex items-center gap-2 m-1"
                                            on:click=move |_| {
                                                open.set(!open.get());
                                            }
                                        >
                                            <Icon icon=i::LuUser />
                                            {username.get()}
                                        </a>
                                    </li>
                                    <b>"Points: "{move || user_points.get().map(|user_points| match user_points {
                                        Ok(user_points) => user_points,
                                        Err(_e) => 0_u32
                                    })}</b>
                                }.into_any()
                            }
                            None => {
                                view! {
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
                                }.into_any()
                            }
                        })}
                    </Suspense>
                    <Show when=move || open.get() fallback=|| ()>
                        <nav class="flex-col w-inherit">
                            <ul>
                                <li>
                                    <a href=user_profile_path.get()>
                                        <Icon icon=i::LuCircleUser />
                                        "Profile"
                                    </a>
                                </li>
                                <li>
                                    <a href="/settings">
                                        <Icon icon=i::LuUserCog />
                                        "Settings"
                                    </a>
                                </li>
                                <li>
                                    <button on:click=move |_| {
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
