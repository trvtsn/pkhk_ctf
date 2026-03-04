// Modules
pub mod app;
pub mod components;
pub mod constants;
pub mod error_template;
pub mod logging;
pub mod pages;
pub mod server;
pub mod utils;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
