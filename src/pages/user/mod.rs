pub mod settings;

use crate::{components::{navbar::NavBar, utils::{ComponentSize, Spinner}}, pages::not_found::NotFound, server::{db::enums::UserIdentifier, get_avatar_id, get_db_user_without_pii}};
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

// pub fn router() -> Router<()> {
//     Router::new()
//         .route("/user/:username", get(self::get::protected))
// }

/// Default Home Page
#[component]
pub fn User() -> impl IntoView {
    let params = use_params_map();
    
    let user_resource = Resource::new(move || (), move |_| {
        let username = params.read().get("username").unwrap_or_default();
        async move { get_db_user_without_pii(Some(username)).await.unwrap_or_default() }
    });

    let user_avatar = Resource::new(move || (), move |_| {
        let username = params.read().get("username").unwrap_or_default();
        async move { get_avatar_id(UserIdentifier::Username(username)).await.unwrap_or_default() }
    });

    view! {
        <NavBar />
        <div class=r#"bg-background text-text h-full p-4 min-h-screen"#>
            <div class=r#"grid grid-cols-4"#>
                <div class="col-start-1 col-end-1 bg-background-secondary m-4 p-4 rounded-lg">
                    <Suspense fallback=move || {
                        view! { <Spinner component_size=ComponentSize::Big /> }
                    }>
                        {move || {
                            let user = user_resource.get().unwrap_or_default();

                            match user {
                                Some(user) => {
                                    view! {
                                        {if let Some(id) = user_avatar.get().unwrap_or_default() {
                                            view! {
                                                <div class="flex justify-center mb-4">
                                                    <img 
                                                        src=move || format!("/avatar/{}", id) 
                                                        class="rounded-[50%] object-cover shadow-sm"
                                                    />
                                                </div>
                                            }.into_any()
                                        } else {
                                            "".into_any()
                                        }}
                                        <p>{user.username}</p>
                                        <p>
                                            <b>"Points: "</b>
                                            {user.points}
                                        </p>
                                        <p>
                                            <b>"Group: "</b>
                                            {
                                                let user_group = user.group;
                                                if user_group.clone().is_empty() {
                                                    view! {
                                                        <i>"None"</i>
                                                    }.into_any()
                                                } else {
                                                    user_group.into_any()
                                                }
                                            }
                                        </p>
                                        <p>
                                            <b>"Date Joined: "</b>
                                            {user.created_at.format("%Y-%m-%d %H:%M:%S").to_string()}
                                        </p>
                                        <p>
                                            <b>"Last Active: "</b>
                                            {user.last_active_at.format("%Y-%m-%d %H:%M:%S").to_string()}
                                        </p>
                                    }.into_any()
                                }
                                None => view! { <NotFound /> }.into_any()
                            }
                        }}
                    </Suspense>
                </div>
            </div>
        </div>
    }
}
