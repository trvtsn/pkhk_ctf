use crate::{components::navbar::NavBar, server::{Register, get_user, user_exists}};
use leptos::{prelude::*, web_sys::{HtmlInputElement, Event}, wasm_bindgen::JsCast};
use leptos_router::hooks::use_navigate;

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

    let name_taken = Resource::new(move || email.get(), move |email| async move {user_exists(email).await});
 
    let available_ui = move || view! {
        <Transition fallback=|| {
            view! { "..." }
        }>
            {move || Suspend::new(async move {
                match name_taken.await {
                    Ok(true) => view! { 
                        <p>" E-mail already in use. "</p>// <a href="">"Forgot Password?"</a>
                    }.into_any(),
                    _ => view! {}.into_any(),
                }
            })}
        </Transition>
    };

    view! {
        <NavBar />
        <div class="container p-8 inline justify-center">
            <h3 class="text-4xl text-center">"Register"</h3>
            <ActionForm action=register>
                <label>"Email"</label>
                <input class="w-full rounded-md border border-gray-300 bg-white px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" type="email" name="email" on:blur=move |ev| {
                    let value = event_target_value(&ev);
                    email.set(value);
                }/>
                    {available_ui}
                <label>"Password"</label>
                <input class="w-full rounded-md border border-gray-300 bg-white px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" type="password" name="password" bind:value=password />
                
                <input
                    type="submit"
                    class="px-4 py-2 rounded-md border border-gray-300 text-sm hover:bg-gray-50"
                    value="Register"
                />
            </ActionForm>
        </div>
    }
}
