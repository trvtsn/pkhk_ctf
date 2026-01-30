pub mod ldap;

use crate::{components::{navbar::NavBar, utils::HidePasswordButton}, server::{LoginUser, get_user}};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

/// Default Home Page
#[component]
pub fn Login() -> impl IntoView {
    let email = RwSignal::new("".to_string());
    let password = RwSignal::new("".to_string());
    let password_hidden = RwSignal::new(true);

    let login: ServerAction<LoginUser> = ServerAction::new();

    let logged_in_user = Resource::new(
        move || login.version().get(),
        move |_user| async move {
            if let Ok(Some(user)) = get_user().await {
                Some(user)
            } else {
                None
            }
        }
    );

    Effect::new(move || {
        if let Some(Some(_)) = logged_in_user.get() {
            let nav = use_navigate();
            nav("/", Default::default());
        }
    });

    view! {
        <NavBar />
        <div class=r#"grid justify-center p-8 grid-col bg-background text-text h-full"#>
            <h3 class=r#"text-4xl text-center"#>"Login"</h3>
            <br />
            <ActionForm action=login>
                <label class=r#"block mb-1 text-sm font-medium text-gray-700"#>"Email"</label>
                <input
                    class=r#"py-2 px-3 w-full text-sm bg-white rounded-md border border-gray-300 
                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                    type="email"
                    name="email"
                    bind:value=email
                />

                <label class=r#"block mb-1 text-sm font-medium text-gray-700"#>"Password"</label>
                <input
                    class=r#"py-2 px-3 w-full text-sm bg-white rounded-md border border-gray-300 
                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                    type=move || if password_hidden.get() { "password" } else { "text" }
                    name="password"
                    bind:value=password
                />
                <HidePasswordButton hidden=password_hidden />

                <input
                    hidden=true
                    name="auth_type"
                    value="normal"
                />

                <input
                    type="submit"
                    class=r#"py-2 px-4 text-sm rounded-md border border-gray-300 hover:bg-gray-50"#
                    value="Login"
                />
            </ActionForm>
            <a href="/login/ldap">"Login with LDAP"</a>
        </div>
    }
}
