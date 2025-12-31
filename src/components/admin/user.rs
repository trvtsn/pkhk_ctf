use leptos::prelude::*;

use crate::server::db::structs::DbUser;
// use thaw::*;

#[component]
pub fn User(
    user: DbUser
) -> impl IntoView {
    view! {
        <div class="rounded-2xl border-2 border-black p-2">
            <p>"ID:" {user.id}</p>
            <p>"Username:" {user.username}</p>
            <p>"E-mail:" {user.email}</p>
            <p>"Created:" {user.created_at.to_string()}</p>
            <p>"Last active:" {user.last_active_at.to_string()}</p>
        </div>
    }
}
