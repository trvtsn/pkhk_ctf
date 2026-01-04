use leptos::prelude::*;

use crate::server::db::structs::DbUser;
// use thaw::*;

#[component]
pub fn User(
    user: DbUser
) -> impl IntoView {
    view! {
        <div class="bg-yale-blue-50 hover:bg-yale-blue-100 rounded-2xl p-4 content-center">
            <p>"ID: " {user.id}</p>
            <p>"Username: " {user.username}</p>
            <p>"E-mail: " {user.email}</p>
            <p>"Created: " {user.created_at.to_string()}</p>
            <p>"Last active: " {user.last_active_at.to_string()}</p>
        </div>
    }
}
