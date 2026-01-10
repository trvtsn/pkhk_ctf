use crate::server::db::structs::DbUser;
use leptos::prelude::*;

#[component]
pub fn User(
    user: DbUser
) -> impl IntoView {
    view! {
        <div class="bg-yale-blue-50 hover:bg-yale-blue-100 rounded-lg p-4 content-center">
            <p class="text-lg/8"><b>"ID: "</b> {user.id}</p>
            <p class="text-lg/8"><b>"Username: "</b> {user.username}</p>
            <p class="text-lg/8"><b>"E-mail: "</b> {user.email}</p>
            <p class="text-lg/8"><b>"Created: "</b> {user.created_at.to_string()}</p>
            <p class="text-lg/8"><b>"Last active: "</b> {user.last_active_at.to_string()}</p>
        </div>
    }
}
