use leptos::prelude::*;

use crate::server::db::structs::DbUser;
// use thaw::*;

#[component]
pub fn Event(
    event: crate::server::db::structs::Event
) -> impl IntoView {
    view! {
        <div class="rounded-1xl border-2 border-black p-2">
            <p>"ID: " {event.id}</p>
            <p>"Name: " {event.name}</p>
            <p>"Description: " {event.description.unwrap_or_default()}</p>
            <p>"Start Date: " {event.start_date.to_string()}</p>
            <p>"End Date: " {event.end_date.to_string()}</p>
        </div>
    }
}
