use crate::{app::RefreshUser, server::{api::LogoutUser, db::{enums::UserRole, structs::DbUserWithoutPII}}};
use icondata as i;
use leptos::prelude::*;
use leptos_icons::Icon;

/// Top navigation bar. Shows links, user dropdown (profile/settings/logout), and points.
#[component]
pub fn NavBar() -> impl IntoView {
    let open = RwSignal::new(false);
    let user = expect_context::<RwSignal<Option<DbUserWithoutPII>>>();
    let refresh_user = expect_context::<RwSignal<RefreshUser>>();
    let logout = ServerAction::<LogoutUser>::new();

    let (role, username, points) = {
        let role = Memo::new(move |_| user.get().map(|u| u.role).unwrap_or(UserRole::Competitor));
        let username = Memo::new(move |_| user.get().map(|u| u.username).unwrap_or_default());
        let points = Memo::new(move |_| user.get().map(|u| u.points).unwrap_or(0));
        (role, username, points)
    };

    Effect::watch(
        move || logout.value().get(), 
        move |val, _, _| {
            if let Some(Ok(_)) = val {
                refresh_user.update(|r| r.iteration += 1);
                open.set(false);
            }
        }, 
        false
    );

    view! {
        <div class=r#"flex top-0 items-center p-4 w-full shadow-sm bg-background text-text z-15"#>
            <div class=r#"flex-1"#></div>

            <nav class=r#"flex justify-center items-center"#>
                <ul class=r#"flex gap-6 items-center p-0 m-0 list-none"#>
                    <li class=r#"flex gap-2 items-center"#>
                        <a href="/" class=r#"inline-flex gap-2 items-center m-1"#>
                            <Icon icon=i::LuHouse />
                            "Home"
                        </a>
                    </li>
                    <li class=r#"flex gap-2 items-center"#>
                        <a href="/challenges" class=r#"inline-flex gap-2 items-center m-1"#>
                            <Icon icon=i::MdiBullseyeArrow />
                            "Challenges"
                        </a>
                    </li>
                    <li class=r#"flex gap-2 items-center"#>
                        <a href="/leaderboard" class=r#"inline-flex gap-2 items-center m-1"#>
                            <Icon icon=i::LuChartLine />
                            "Leaderboard"
                        </a>
                    </li>
                    <Show when=move || user.get().is_some() && role.get() == UserRole::Admin>
                        <a href="/admin" class=r#"inline-flex gap-2 items-center m-1"#>
                            <Icon icon=i::LuSettings />
                            "Admin"
                        </a>
                    </Show>
                </ul>
            </nav>

            <nav class=r#"flex flex-1 gap-2 justify-end items-center p-2"#>
                <div class="flex justify-center w-1/3">
                    <ul class=r#"flex gap-4 items-center p-0 m-0 list-none"#>
                        <Show when=move || user.get().is_some()>
                            <li class=r#"relative flex gap-2 items-center"#>
                                <a
                                    class=r#"inline-flex gap-2 items-center m-1 cursor-pointer"#
                                    on:click=move |_| {
                                        open.set(!open.get_untracked());
                                    }
                                >
                                    <Icon icon=i::LuUser />
                                    {move || username.get()}
                                </a>
                                <Show when=move || open.get() fallback=|| ()>
                                    <nav
                                        class=r#"absolute z-10 top-full left-1/2 -translate-x-1/2 flex-col p-4 mt-2 rounded-md shadow-sm bg-background"#
                                        on:blur=move |_| { open.set(false) }
                                    >
                                        <ul class=r#"flex flex-col gap-4 items-center"#>
                                            <li class=r#"w-full"#>
                                                <a href=move || format!("/profile/{}", username.get()) class="flex gap-2 items-center">
                                                    <Icon icon=i::LuCircleUser />
                                                    "Profile"
                                                </a>
                                            </li>
                                            <li class=r#"w-full"#>
                                                <a href="/settings" class="flex gap-2 items-center">
                                                    <Icon icon=i::LuUserCog />
                                                    "Settings"
                                                </a>
                                            </li>
                                            <li class=r#"w-full"#>
                                                <ActionForm action=logout>
                                                    <button
                                                        class=r#"cursor-pointer flex gap-2 items-center"#
                                                        type="Submit"
                                                    >
                                                        <Icon icon=i::LuLogOut />
                                                        "Logout"
                                                    </button>
                                                </ActionForm>
                                            </li>
                                        </ul>
                                    </nav>
                                </Show>
                            </li>
                            <b>"Points: "{move || points.get()}</b>
                        </Show>

                        <Show when=move || user.get().is_none()>
                            <li class=r#"flex gap-2 items-center"#>
                                <a href="/login" class=r#"inline-flex gap-2 items-center m-1"#>
                                    <Icon icon=i::LuLogIn />
                                    "Login"
                                </a>
                            </li>
                            <li class=r#"flex gap-2 items-center"#>
                                <a href="/register" class=r#"inline-flex gap-2 items-center m-1"#>
                                    <Icon icon=i::LuUserPlus />
                                    "Register"
                                </a>
                            </li>
                        </Show>
                    </ul>
                </div>
            </nav>
        </div>
    }
}
