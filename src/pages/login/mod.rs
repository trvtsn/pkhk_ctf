pub mod ldap;

use crate::{app::RefreshUser, components::{navbar::NavBar, toast::{ToastAppear, ToastMessageType}, utils::HidePasswordButton}, error_template::AppError, server::{get_user, is_ldap_enabled, login_user}};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

/// Default Home Page
#[component]
pub fn Login() -> impl IntoView {
    let toast_message_type = expect_context::<RwSignal<ToastMessageType>>();
    let toast_appear = expect_context::<RwSignal<ToastAppear>>();

    let email = RwSignal::new("".to_string());
    let password = RwSignal::new("".to_string());
    let password_hidden = RwSignal::new(true);
    let refresh_user = expect_context::<RwSignal<RefreshUser>>();

    let login = Action::new_local(move |(email, password): &(String, String)| {
        let email = email.clone();
        let password = password.clone();
        async move {
            if let Err(e) = login_user(email, password, crate::server::backend::enums::AuthType::Normal).await 
            && e == AppError::BadRequest("invalid credentials".to_string()) {
                toast_appear.set(true);
                toast_message_type.set(ToastMessageType::InvalidCredentials);
            }
        }
    });

    let logged_in_user = Resource::new(
        move || login.version().get(),
        move |_user| async move {
            if let Ok(Some(user)) = get_user().await {
                refresh_user.update(|r| r.iteration += 1);
                Some(user)
            } else {
                None
            }
        }
    );

    let ldap_enabled_resource = Resource::new(move || (), move |_| async move {
        is_ldap_enabled().await.unwrap_or_default()
    });

    let ldap_login_view = Suspend::new(async move {
        let is_ldap_enabled = ldap_enabled_resource.await;
        if is_ldap_enabled {
            view! {
                <a 
                    class="text-blue-600"
                    href="/login/ldap"
                >
                    "Login with LDAP"
                </a>
            }.into_any()
        } else {
            "".into_any()
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
            <h3 class=r#"text-4xl text-center"#>"Login"</h3>
            <form 
                class="flex flex-col gap-4"
                on:submit=move |ev| {
                    ev.prevent_default();
                    let email = email.get();
                    let password = password.get();
                    login.dispatch((email, password));
                }
            >
                <label class=r#"block mb-1 text-sm font-medium"#>"Email"</label>
                <input
                    class=r#"py-2 px-3 w-full text-sm rounded-md border border-input-border 
                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                    type="email"
                    name="email"
                    required
                    bind:value=email
                />

                <label class=r#"block mb-1 text-sm font-medium"#>"Password"</label>
                <div class="flex gap-2">
                    <input
                        class=r#"py-2 px-3 w-full text-sm rounded-md border border-input-border 
                        focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                        type=move || if password_hidden.get() { "password" } else { "text" }
                        name="password"
                        required
                        bind:value=password
                    />
                    <HidePasswordButton hidden=password_hidden />
                </div>

                <input
                    hidden=true
                    name="auth_type"
                    value="normal"
                />

                <input
                    type="submit"
                    class=r#"py-2 px-4 text-sm rounded-md border border-input-border hover:bg-background-hover"#
                    value="Login"
                />
            </form>
            {ldap_login_view}
        </div>
    }
}
