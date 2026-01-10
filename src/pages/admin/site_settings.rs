// use super::AdminNavBar;
// use crate::components::navbar::NavBar;
use leptos::prelude::*;

/// Default Home Page
#[component]
pub fn SiteSettings() -> impl IntoView {
    view! {
        <div class="container">
            <button class="ml-auto inline-flex items-center px-4 py-2 rounded-md bg-indigo-600 text-white text-sm font-semibold shadow-sm hover:bg-indigo-500 focus:outline-none focus:ring-2 focus:ring-indigo-500">"Change Favicon"</button>
        </div>
    }
}
