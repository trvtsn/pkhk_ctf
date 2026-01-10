use leptos::prelude::*;

#[component]
pub fn Event(
    event: crate::server::db::structs::Event
) -> impl IntoView {
    view! {
        <div class="bg-yale-blue-50 hover:bg-yale-blue-100 rounded-lg p-4 content-center">
            <p class="text-lg/8"><b>"ID: "</b> {event.id}</p>
            <p class="text-lg/8"><b>"Name: "</b> {event.name}</p>
            <p class="text-lg/8"><b>"Description: "</b> {event.description.unwrap_or_default()}</p>
            <p class="text-lg/8"><b>"Start Date: "</b> {event.start_date.to_string()}</p>
            <p class="text-lg/8"><b>"End Date: "</b> {event.end_date.to_string()}</p>
        </div>
    }
}
