use crate::components::navbar::NavBar;
use leptos::prelude::*;

/// Default Home Page
#[component]
pub fn Settings() -> impl IntoView {
    let dark_mode = RwSignal::new(false);
    view! {
        <NavBar />
        <div class="container">
            <label>"Dark Mode" <input type="checkbox" bind:checked=dark_mode /></label>
        </div>
    }
}
