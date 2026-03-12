use crate::{app::RefreshUser, components::{navbar::NavBar, toast::{ToastMessageType, push_new_toast}, utils::{ComponentSize, HidePasswordButton, Spinner}}, error_template::AppError, server::{get_user, is_ldap_enabled, login_user, structs::Credentials}};
use leptos::{prelude::*, task::spawn_local};
use leptos_router::hooks::use_navigate;

/// Default Home Page
#[component]
pub fn Login() -> impl IntoView {
    let username = RwSignal::new("".to_string());
    let password = RwSignal::new("".to_string());
    let password_hidden = RwSignal::new(true);
    let refresh_user = expect_context::<RwSignal<RefreshUser>>();

    let login = Action::new_local(move |(username, password): &(String, String)| {
        let username = username.clone();
        let password = password.clone();
        let creds = Credentials { 
            user_identifier: crate::server::db::enums::UserIdentifier::Username(username), 
            password, 
            auth_type: crate::server::backend::enums::AuthType::Ldap
        };
        async move {
            if let Err(e) = login_user(creds).await 
            && e == AppError::BadRequest("invalid credentials".to_string()) {
                spawn_local(async move {
                    push_new_toast(ToastMessageType::InvalidCredentials);
                });
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

    Effect::new(move || {
        if let Some(Some(_)) = logged_in_user.get() {
            let nav = use_navigate();
            nav("/", Default::default());
        }

        if let Some(enabled) = ldap_enabled_resource.get() {
            if !enabled {
                let nav = use_navigate();
                nav("/login", Default::default());
            }
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
                    let username = username.get();
                    let password = password.get();
                    login.dispatch((username, password));
                }
            >
                <label class=r#"block mb-1 text-sm font-medium text-text"#>"Username"</label>
                <input
                    class=r#"py-2 px-3 w-full text-sm rounded-md border border-input-border 
                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                    name="username"
                    required
                    bind:value=username
                />

                <label class=r#"block mb-1 text-sm font-medium text-text"#>"Password"</label>
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

                <button
                    type="submit"
                    class=r#"py-2 px-4 text-sm rounded-md border border-input-border hover:bg-background-hover"#
                    disabled=move || login.pending().get()
                >
                    {move || if login.pending().get() {
                        view! { <Spinner component_size=ComponentSize::Small /> }.into_any()
                    } else {
                        "Login".into_any()
                    }}
                </button>
            </form>
            <a 
                class="text-blue-600"
                href="/login"
            >
                "Go back to /login"
            </a>
        </div>
    }
}
