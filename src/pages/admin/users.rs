// use super::AdminNavBar;
// use crate::components::navbar::NavBar;
use leptos::prelude::*;

use crate::{components::admin::user::User, server::{structs::ApiResult, admin::{get_all_users}, db::structs::DbUser}};

/// Default Home Page
#[component]
pub fn Users() -> impl IntoView {
    let users = Resource::new(move || (), move |_| async move {
        match get_all_users().await {
            Ok(users) => Ok(users),
            Err(e) => Err(e)
        }
    });

    view! {
        <div class="grid-cols-4 p-4 m-4 flex">
            <Suspense fallback=move || view! { <div>"Loading..."</div> }>
                {move || {
                    let users = users.get().map(move |result| match result {
                        Ok(users) => {
                            view! {
                                <For
                                    each=move || users.clone()
                                    key=|user: &DbUser| user.id
                                    let(user)
                                >
                                    <User user/>
                                </For>
                            }.into_any()
                        }
                        Err(e) => {
                            view! {
                                <div class="challenge p-2">
                                    <p>"Bruh" {e.to_string()}</p>
                                </div>
                            }.into_any()
                        }
                    })
                    .collect_view()
                    .into_any();
            
                    view! {
                        {users}
                    }
                }}
            </Suspense>
        </div>
    }
}
