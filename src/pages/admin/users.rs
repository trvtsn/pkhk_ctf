// use super::AdminNavBar;
// use crate::components::navbar::NavBar;
use leptos::prelude::*;

use crate::server::{db::structs::DbUser, get_all_users};

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
                                <div class="challenge p-2">
                                    <p>{user.username}</p>
                                    <p>{user.email}</p>
                                    <p>{user.created_at.to_string()}</p>
                                </div>
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
    }
}
