use crate::{components::navbar::NavBar, server::{Register, user_exists}};
use leptos::prelude::*;

/// Default Home Page
#[component]
pub fn Register() -> impl IntoView {
    let email = RwSignal::new("".to_string());
    let password = RwSignal::new("".to_string());
    let confirm_password = RwSignal::new("".to_string());
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
                        <p>"E-mail already in use. "</p>// <a href="">"Forgot Password?"</a>
                    }.into_any(),
                    _ => view! {}.into_any(),
                }
            })}
        </Transition>
    };

    let confirm_password_ui = move || view! {
        <Transition fallback=|| {
            view! { "..." }
        }>
            {move || 
                if password.get() != confirm_password.get() {
                    view! {
                        <p>"Must match with password"</p>
                    }.into_any()
                } else {
                    view! {}.into_any()
                }
            }
        </Transition>
    };

    view! {
        <NavBar />
        <div class="p-8 justify-center grid grid-col">
            <h3 class="text-4xl text-center">"Register"</h3>
            <br/>
            <ActionForm action=register>
                <label class="block text-sm font-medium text-gray-700 mb-1">"Email"</label>
                {available_ui}
                <input class="w-full rounded-md border border-gray-300 bg-white px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" type="email" name="email" on:blur=move |ev| {
                    let value = event_target_value(&ev);
                    email.set(value);
                }/>
                    
                <label class="block text-sm font-medium text-gray-700 mb-1">"Password"</label>
                <input class="w-full rounded-md border border-gray-300 bg-white px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" type="password" name="password" bind:value=password/>
                
                <label class="block text-sm font-medium text-gray-700 mb-1">"Confirm Password"</label>
                <input class="w-full rounded-md border border-gray-300 bg-white px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" type="password" name="confirm_password" bind:value=confirm_password/>
                {confirm_password_ui}

                <input
                    type="submit"
                    class="px-4 py-2 rounded-md border border-gray-300 text-sm hover:bg-gray-50"
                    value="Register"
                />
            </ActionForm>
        </div>
    }
}
