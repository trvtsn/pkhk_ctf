use leptos::prelude::*;

/// Admin site settings (placeholder - currently only a static "Change Favicon" button).
#[component]
pub fn SiteSettings() -> impl IntoView {
    view! {
        <div class=r#"container"#>
            <button class=r#"inline-flex items-center py-2 px-4 ml-auto text-sm font-semibold 
            text-white rounded-md shadow-sm focus:ring-2 focus:outline-none bg-yale-blue-600 
            hover:bg-yale-blue-500 focus:ring-yale-blue-500"#>
                "Change Favicon"
            </button>
        </div>
    }
}
