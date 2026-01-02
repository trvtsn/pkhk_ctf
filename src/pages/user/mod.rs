pub mod settings;

use crate::{components::navbar::NavBar, pages::not_found::NotFound, server::{self, db::{self, enums::UserRole, structs::DbUser}, get_db_user, structs::ApiResult}};
#[cfg(feature = "ssr")]
use axum::Router;
use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use time::OffsetDateTime;

// pub fn router() -> Router<()> {
//     Router::new()
//         .route("/user/:username", get(self::get::protected))
// }

/// Default Home Page
#[component]
pub fn User() -> impl IntoView {
    let params = use_params_map();
    let username = move || params.read().get("username").unwrap_or_default();
    
    let user_res = Resource::new(move || (), move |_| async move {
        get_db_user(username()).await.unwrap_or_default()
    });

    view! {
        <NavBar />
        <div class="container">
            <Suspense fallback=move || view! { <div>"Loading..."</div> }>
                <p>
                {move || {
                    let view_result = user_res.get().map(|j| match j {
                        Some(user) => { 
                            view! { {user.username} }.into_any()
                        },
                        None => view! { <NotFound /> }.into_any()
                    });

                    view! {
                        { view_result }
                    }.into_any()
                }}
                </p>
                //<p>"Date Joined: " {move || user.get().created_at.to_string()}</p>
            </Suspense>
        </div>
    }
}
