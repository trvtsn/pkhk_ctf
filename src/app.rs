use leptos::{prelude::*};
use leptos_meta::{provide_meta_context, MetaTags, Title};
use leptos_router::{
    components::*,
    path
};
use leptos_use::{UseColorModeOptions, UseColorModeReturn, use_color_mode_with_options};
use serde::{Deserialize, Serialize};

// Top-Level pages
use crate::{
    pages::{
        admin::Admin, challenges::Challenges, home::Home, leaderboard::Leaderboard, login::Login,
        not_found::NotFound, register::Register, user,
    }, server::{db::enums::UserRole, get_db_user}
};


pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone() />
                <HydrationScripts options/>
                <MetaTags/>
                <link rel="stylesheet" href="/pkg/pkhk_ctf.css"/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct RefreshUser {
    pub iteration: u32
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    let UseColorModeReturn { mode, set_mode, .. } = use_color_mode_with_options(
        UseColorModeOptions::default().cookie_enabled(true)
    );
    provide_context(mode);
    provide_context(set_mode);

    let refresh_user = RwSignal::new(RefreshUser { iteration: 0 });
    let user = RwSignal::new(None);
    let user_resource = Resource::new(move || refresh_user.get(), |_| async move {
        get_db_user(None).await.unwrap_or(None)
    });

    // effects arent actually intended to synchronize with the reactive system,
    // find another solution
    Effect::new(move |_| {
        let user_value = user_resource.get().unwrap_or(None);
        // only set the signal if it's different, avoiding infinite loops
        if user_value != user.get() {
            user.set(user_value);
        }
    });

    provide_context(user);
    provide_context(refresh_user);

    view! {
        <ErrorBoundary fallback=|errors| {
            view! {
                <h1>"Uh oh! Something went wrong!"</h1>

                <p>"Errors: "</p>
                <ul>
                    {move || {
                        errors
                            .get()
                            .into_iter()
                            .map(|(_, e)| view! { <li>{e.to_string()}</li> })
                            .collect_view()
                    }}

                </ul>
            }
        }>
            <Title text="PKHK CTF" />

            <Router>
                <Routes fallback=NotFound>
                    <Route path=path!("/") view=Home ssr=leptos_router::SsrMode::InOrder/>
                    <Route path=path!("/login") view=Login ssr=leptos_router::SsrMode::InOrder/>
                    <Route path=path!("/register") view=Register ssr=leptos_router::SsrMode::InOrder/>
                    <ProtectedRoute
                        path=path!("/admin")
                        redirect_path=|| "/login"
                        condition=move || user.get().map(|u| u.role == UserRole::Admin)
                        view=Admin
                        ssr=leptos_router::SsrMode::InOrder
                    ></ProtectedRoute>
                    <Route path=path!("/challenges") view=Challenges ssr=leptos_router::SsrMode::InOrder/>
                    <Route path=path!("/leaderboard") view=Leaderboard ssr=leptos_router::SsrMode::InOrder/>
                    <ProtectedRoute
                        path=path!("/settings")
                        redirect_path=|| "/login"
                        condition=move || Some(user.get().is_some())
                        view=user::settings::Settings
                        ssr=leptos_router::SsrMode::InOrder
                    >
                    </ProtectedRoute>
                    <ParentRoute
                        path=path!("/profile")
                        view=user::User
                        ssr=leptos_router::SsrMode::InOrder
                    >
                        <Route path=path!(":username") view=user::User ssr=leptos_router::SsrMode::InOrder/>
                    </ParentRoute>
                </Routes>
            </Router>
        </ErrorBoundary>
    }
}
