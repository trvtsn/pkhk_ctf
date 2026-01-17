use leptos::prelude::*;

/// Default Home Page
#[component]
pub fn SiteSettings() -> impl IntoView {
    view! {
        <div class="container">
            <button class="inline-flex items-center py-2 px-4 ml-auto text-sm font-semibold text-white rounded-md shadow-sm focus:ring-2 focus:ring-yale-blue-500 focus:outline-none bg-yale-blue-600 hover:bg-yale-blue-500">
                "Change Favicon"
            </button>
        </div>
    }
}
