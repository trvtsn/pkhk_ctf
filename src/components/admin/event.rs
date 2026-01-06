use leptos::prelude::*;

#[component]
pub fn Event(
    event: crate::server::db::structs::Event
) -> impl IntoView {
    view! {
        <div class="bg-yale-blue-50 hover:bg-yale-blue-100 rounded-2xl p-4 content-center">
            <p>"ID: " {event.id}</p>
            <p>"Name: " {event.name}</p>
            <p>"Description: " {event.description.unwrap_or_default()}</p>
            <p>"Start Date: " {event.start_date.to_string()}</p>
            <p>"End Date: " {event.end_date.to_string()}</p>
        </div>
    }
}
