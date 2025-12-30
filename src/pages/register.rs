use crate::{components::navbar::NavBar, server::Register};
use leptos::prelude::*;

/// Default Home Page
#[component]
pub fn Register() -> impl IntoView {
    let email = RwSignal::new("".to_string());
    let password = RwSignal::new("".to_string());
    let loading = RwSignal::new(false);

    let register: ServerAction<Register> = ServerAction::new();

    let button_classes = Memo::new(move |_| {
        let base = "border-2 border-black p-2 text-black rounded";
        if loading.get() {
            format!("{base} {}", "animate-spin")
        } else {
            format!(
                "{base} {}",
                "bg-lavender-blush-100 hover:bg-lavender-blush-200"
            )
        }
    });

    view! {
        <NavBar />
        <div class="container p-8 inline justify-center">
            <h3 class="text-4xl text-center">"Register"</h3>
            <ActionForm action=register>
                <label>
                    <b>"Email"</b>
                    <input class="bg-white border" type="email" name="email" bind:value=email />
                </label>
                <label>
                    <b>"Password"</b>
                    <input class="bg-white border" type="password" name="password" bind:value=password />
                </label>
                //<button loading=loading on_click=move |_| { loading.set(true) }>
                <button class=move || button_classes on:click=move |_| { loading.set(true) } >
                    "Register"
                </button>
            </ActionForm>
        </div>
    }
}
