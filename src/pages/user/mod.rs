pub mod settings;

use crate::{components::navbar::NavBar, pages::not_found::NotFound, server::{db::enums::UserIdentifier, get_avatar_id, get_db_user_without_pii}};
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
        async move { get_db_user_without_pii(Some(username)).await.unwrap_or_default() }
    });

    let user_avatar = Resource::new(move || (), move |_| {
        let username = params.read().get("username").unwrap_or_default();
        async move { get_avatar_id(UserIdentifier::Username(username)).await.unwrap_or_default() }
    });

    view! {
        <NavBar />
        <div class=r#"container"#>
            <Suspense fallback=move || {
                view! { <div>"Loading..."</div> }
            }>
                {move || {
                    let user = user_res.get().unwrap_or_default();
                    let avatar_view = {
                        if let Some(id) = user_avatar.get().unwrap_or_default() {
                            view! {
                                <div class="h-64 w-64">
                                    <img 
                                        src=move || format!("/avatar/{}", id) 
                                        class=r#"text-blue-600 underline rounded-[50%] 
                                        object-cover shadow-sm"#
                                    />
                                </div>
                            }.into_any()
                        } else {
                            "".into_any()
                        }
                    };

                    let view = match user {
                        Some(user) => {
                            view! {
                                {avatar_view}
                                <p>{user.username}</p>
                                <p>
                                    <b>"Points: "</b>
                                    {user.points}
                                </p>
                                <p>
                                    <b>"Date Joined: "</b>
                                    {user.created_at.to_string()}
                                </p>
                            }.into_any()
                        }
                        None => view! { <NotFound /> }.into_any(),
                    };

                    view! { {view} }
                }}
            </Suspense>
        </div>
    }
}
