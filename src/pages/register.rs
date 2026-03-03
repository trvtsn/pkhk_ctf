use crate::{app::RefreshUser, components::{navbar::NavBar, utils::{ComponentSize, HidePasswordButton, Spinner}}, server::{get_user, register_user, user_exists}};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

/// Default Home Page
#[component]
pub fn Register() -> impl IntoView {
    let email = RwSignal::new("".to_string());
    let password = RwSignal::new("".to_string());
    let confirm_password = RwSignal::new("".to_string());
    let password_confirm_matches = RwSignal::new(false);

    let password_hidden = RwSignal::new(true);
    let confirm_password_hidden = RwSignal::new(true);
    let refresh_user = expect_context::<RwSignal<RefreshUser>>();

    let register = Action::new_local(move |(email, password, confirm_password): &(String, String, String)| {
        let email = email.clone();
        let password = password.clone();
        let confirm_password = confirm_password.clone();
        async move {
            register_user(email, password, confirm_password).await
        }
    });

    let logged_in_user = Resource::new(
        move || register.version().get(),
        move |_user| async move {
            if let Ok(Some(user)) = get_user().await {
                refresh_user.update(|r| r.iteration += 1);
                Some(user)
            } else {
                None
            }
        }
    );

    let name_taken = Resource::new(move || email.get(), move |email| async move {user_exists(email).await});
    let available_ui = Suspend::new(async move {
        match name_taken.await {
            Ok(true) => "E-mail already in use.",
            Ok(false) => "",
            Err(_) => ""            
        }
    });

    Effect::new(move || {
        if let Some(Some(_)) = logged_in_user.get() {
            let nav = use_navigate();
            nav("/", Default::default());
        }
    });

    view! {
        <NavBar />
        <div class=r#"grid justify-center p-8 grid-col bg-background text-text min-h-screen"#>
            <h3 class=r#"text-4xl text-center"#>"Register"</h3>
            <form 
                class="flex flex-col gap-4"
                on:submit=move |ev| {
                    ev.prevent_default();
                    let email = email.get();
                    let password = password.get();
                    let confirm_password = confirm_password.get();
                    register.dispatch((email, password, confirm_password));
                }
            >
                <label class=r#"block mb-1 text-sm font-medium text-text"#>"Email"</label>
                <Transition fallback=|| view! { <Spinner component_size=ComponentSize::Medium /> }>
                    {available_ui}
                </Transition>

                <input
                    class=r#"py-2 px-3 w-full text-sm rounded-md border border-input-border 
                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                    type="email"
                    name="email"
                    required
                    on:blur=move |ev| {
                        let value = event_target_value(&ev);
                        email.set(value);
                    }
                />

                <label class=r#"block mb-1 text-sm font-medium text-text"#>"Password"</label>
                <div class="flex gap-2">
                    <input
                        class=r#"py-2 px-3 w-full text-sm rounded-md border border-input-border 
                        ocus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                        type=move || if password_hidden.get() { "password" } else { "text" }
                        name="password"
                        required
                        bind:value=password
                    />
                    <HidePasswordButton hidden=password_hidden />
                </div>

                <label class=r#"block mb-1 text-sm font-medium text-text"#>
                    "Confirm Password"
                </label>
                <div class="flex gap-2">
                    <input
                        class=r#"py-2 px-3 w-full text-sm rounded-md border border-input-border 
                        focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                        type=move || if confirm_password_hidden.get() { "password" } else { "text" }
                        name="confirm_password"
                        required
                        bind:value=confirm_password
                    />
                    <HidePasswordButton hidden=confirm_password_hidden />
                </div>

                {move || if password.get() != confirm_password.get() {
                    "Confirmation must match"
                } else {
                    password_confirm_matches.set(true);
                    ""
                }}

                <input
                    type="submit"
                    class=r#"py-2 px-4 text-sm rounded-md border border-input-border hover:bg-background-hover"#
                    disabled=move || !password_confirm_matches.get()
                    value="Register"
                />
            </form>
        </div>
    }
}
