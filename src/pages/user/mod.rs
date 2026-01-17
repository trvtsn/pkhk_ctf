pub mod settings;

use crate::{components::navbar::NavBar, pages::not_found::NotFound, server::{get_db_user, get_avatar}};
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
    
    let user_res = Resource::new(move || (), move |_| {
        let username = params.read().get("username").unwrap_or_default();
        async move { get_db_user(Some(username)).await.unwrap_or_default() } // leaks pw_hash
    });

    let user_avatar = Resource::new(move || (), move |_| {
        let username = params.read().get("username").unwrap_or_default();
        async move { get_avatar(username).await.unwrap_or_default() }
    });

    view! {
        <NavBar />
        <div class="container">
            <Suspense fallback=move || {
                view! { <div>"Loading..."</div> }
            }>
                {move || {
                    let user = user_res.get().unwrap_or_default();
                    let view = match user {
                        Some(user) => {
                            let avatar = user_avatar.get().unwrap_or_default();

                            view! {
                                <p>{avatar}</p>
                                <p>{user.username}</p>
                                <p>
                                    <b>"Points: "</b>
                                    {user.points}
                                </p>
                                <p>
                                    <b>"Date Joined: "</b>
                                    {user.created_at.to_string()}
                                </p>
                            }
                                .into_any()
                        }
                        None => view! { <NotFound /> }.into_any(),
                    };

                    view! { {view} }
                }}
            </Suspense>
        </div>
    }
}
