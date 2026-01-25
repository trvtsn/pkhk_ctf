use crate::{components::{navbar::NavBar, utils::HidePasswordButton}, server::{Register, user_exists}};
use leptos::prelude::*;

/// Default Home Page
#[component]
pub fn Register() -> impl IntoView {
    let email = RwSignal::new("".to_string());
    let password = RwSignal::new("".to_string());
    let confirm_password = RwSignal::new("".to_string());
    let password_hidden = RwSignal::new(true);
    let confirm_password_hidden = RwSignal::new(true);

    let register: ServerAction<Register> = ServerAction::new();

    let name_taken = Resource::new(move || email.get(), move |email| async move {user_exists(email).await});
    let available_ui = Suspend::new(async move {
        match name_taken.await {
            Ok(true) => "E-mail already in use.",
            Ok(false) => "",
            Err(_) => ""
        }
    });

    view! {
        <NavBar />
        <div class=r#"grid justify-center p-8 grid-col bg-background text-text h-full"#>
            <h3 class=r#"text-4xl text-center"#>"Register"</h3>
            <br />
            <ActionForm action=register>
                <label class=r#"block mb-1 text-sm font-medium text-gray-700"#>"Email"</label>
                <Transition fallback=|| view! { "..." }>{available_ui}</Transition>

                <input
                    class=r#"py-2 px-3 w-full text-sm bg-white rounded-md border border-gray-300 
                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                    type="email"
                    name="email"
                    on:blur=move |ev| {
                        let value = event_target_value(&ev);
                        email.set(value);
                    }
                />

                <label class=r#"block mb-1 text-sm font-medium text-gray-700"#>"Password"</label>
                <input
                    class=r#"py-2 px-3 w-full text-sm bg-white rounded-md border border-gray-300 
                    ocus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                    type=move || if password_hidden.get() { "password" } else { "text" }
                    name="password"
                    bind:value=password
                />
                <HidePasswordButton hidden=password_hidden />

                <label class=r#"block mb-1 text-sm font-medium text-gray-700"#>
                    "Confirm Password"
                </label>
                <input
                    class=r#"py-2 px-3 w-full text-sm bg-white rounded-md border border-gray-300 
                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                    type=move || if confirm_password_hidden.get() { "password" } else { "text" }
                    name="confirm_password"
                    bind:value=confirm_password
                />
                <HidePasswordButton hidden=confirm_password_hidden />
                <Transition fallback=|| {
                    view! { "..." }
                }>
                    {if password.get() != confirm_password.get() {
                        "Confirmation must match"
                    } else {
                        ""
                    }}
                </Transition>

                <input
                    type="submit"
                    class=r#"py-2 px-4 text-sm rounded-md border border-gray-300 hover:bg-gray-50"#
                    value="Register"
                />
            </ActionForm>
        </div>
    }
}
