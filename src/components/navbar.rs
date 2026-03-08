use crate::{app::RefreshUser, server::{LogoutUser, db::{enums::UserRole, structs::DbUserWithoutPII}}};
use icondata as i;
use leptos::prelude::*;
use leptos_icons::Icon;

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
                <ul class=r#"flex gap-4 items-center p-0 m-0 list-none"#>
                    <Show when=move || user.get().is_some()>
                        <li class=r#"flex gap-2 items-center"#>
                            <a
                                class=r#"inline-flex gap-2 items-center m-1 cursor-pointer"#
                                on:click=move |_| {
                                    open.set(!open.get_untracked());
                                }
                            >
                                <Icon icon=i::LuUser />
                                {move || username.get()}
                            </a>
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

                    <Show when=move || open.get() fallback=|| ()>
                        <nav
                            class=r#"absolute z-10 flex-col p-4 rounded-md shadow-sm bg-background mt-[15rem]"#
                            on:blur=move |_| { open.set(false) }
                        >
                            <ul class=r#"flex flex-col gap-4 items-center"#>
                                <li class=r#"w-full"#>
                                    <a href=move || format!("/profile/{}", username.get())>
                                        <Icon icon=i::LuCircleUser />
                                        "Profile"
                                    </a>
                                </li>
                                <li class=r#"w-full"#>
                                    <a href="/settings">
                                        <Icon icon=i::LuUserCog />
                                        "Settings"
                                    </a>
                                </li>
                                <li class=r#"w-full"#>
                                    <ActionForm action=logout>
                                        <button
                                            class=r#"cursor-pointer"#
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
                </ul>
            </nav>
        </div>
    }
}
