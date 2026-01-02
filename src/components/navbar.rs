use crate::server::{db::enums::UserRole, get_user, get_user_points, structs::{ApiResult, User}};
//use icondata as i;
use leptos::{prelude::*, task::spawn_local};
//use leptos_icons::Icon;
// use thaw::*;

#[component]
pub fn NavBar() -> impl IntoView {
    let open = RwSignal::new(false);
    let user = Resource::new(move || (), async move |_| {
        get_user().await
    });
    let user_points = Resource::new(move || (), async move |_| {
        get_user_points().await
    });
    let username = RwSignal::new("".to_string());
    let user_profile_path = RwSignal::new("".to_string());

    view! {
        <div class="sticky flex-row gap-2 top-0 w-full text-center items-center justify-between bg-lavender-blush-50/25 border-b p-4">
            <nav class="flex-row p-2">
                <a href="/" class="m-1">
                    // <Icon icon=i::IoHome />
                    "Home"
                </a>
                <a href="/challenges" class="m-1">
                    //<Icon icon=i::MdiBullseyeArrow />
                    "Challenges"
                </a>
                <a href="/leaderboard" class="m-1">
                    // <Icon icon=i::IoPodium />
                    "Leaderboard"
                </a>
                <Suspense fallback=move || view! { <p>"Loading..."</p> }>
                    {move || user.get().map(|j| match j {
                        Ok(user) => {
                            match user {
                                Some(user) => {
                                    username.set(user.username);
                                    user_profile_path.set(format!("/user/{}", username.get()));
                                    view! {
                                        <Show when=move || user.role == UserRole::Admin >
                                            <a href="/admin" class="m-1">
                                                //<Icon icon=i::IoSettings />
                                                "Admin"
                                            </a>
                                        </Show>
                                        <a
                                            class="m-1"
                                            on:click=move |_| {
                                                open.set(!open.get());
                                            }
                                        >
                                        {username.get()}
                                        </a>
                                        <b>"Points: "{move || user_points.get().map(|user_points| match user_points {
                                            Ok(user_points) => user_points,
                                            Err(e) => 0 as u32
                                        })}</b>
                                    }.into_any()
                                },
                                None => {
                                    view! {
                                        <a href="/login" class="m-1">
                                            //<Icon icon=i::IoSettings />
                                            "Login"
                                        </a>
                                        <a href="/register" class="m-1">
                                            //<Icon icon=i::IoSettings />
                                            "Register"
                                        </a>
                                    }.into_any()
                                }
                            }
                        },
                        Err(e) => {
                            view! {
                                <a href="/login" class="m-1">
                                    //<Icon icon=i::IoSettings />
                                    "Login"
                                </a>
                                <a href="/register" class="m-1">
                                    //<Icon icon=i::IoSettings />
                                    "Register"
                                </a>
                            }.into_any()
                        }
                    })}
                </Suspense>
                <Show when=move || open.get() fallback=|| ()>
                    <nav class="flex-col p-2 w-inherit">
                        <a href=user_profile_path.get()>"Profile"</a>
                        <a href="/user/settings">"Settings"</a>
                        <a href="/logout">"Logout"</a>
                    </nav>
                    // <NavDrawer class="flex-col">
                    //     <NavItem class="m-1" value="Profile" href="/user/profile"><p>Profile</p></NavItem>
                    //     <NavItem class="m-1" value="Settings" href="/user/settings"><p>Settings</p></NavItem>
                    //     <NavItem class="m-1" value="Logout" href="/user/logout" on:click=move |_| {
                    //         spawn_local(async {_ = logout_user().await;});
                    //     }><p>Logout</p></NavItem>
                    // </NavDrawer>
                </Show>
            </nav>
        </div>
    }
}
