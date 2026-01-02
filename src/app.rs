use leptos::{prelude::*, task::spawn_local};
use leptos_meta::{provide_meta_context, MetaTags, Title};
use leptos_router::{
    components::*,
    path
};

// Top-Level pages
use crate::{
    pages::{
        admin::Admin, challenges::Challenges, home::Home, leaderboard::Leaderboard, login::Login,
        not_found::NotFound, register::Register, user,
    }, server::{db::enums::UserRole, get_user, structs::ApiResult}
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

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    let user = Resource::new(move|| (), |_| async move {
        get_user().await.unwrap_or(None)
    });

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
                        condition=move || user.get().map(|u| match u {
                            Some(user) => user.role == UserRole::Admin,
                            None => false
                        })
                        view=Admin
                        ssr=leptos_router::SsrMode::InOrder
                    ></ProtectedRoute>
                    <Route path=path!("/challenges") view=Challenges ssr=leptos_router::SsrMode::InOrder/>
                    <Route path=path!("/leaderboard") view=Leaderboard ssr=leptos_router::SsrMode::InOrder/>
                    <ProtectedRoute
                        path=path!("/settings")
                        redirect_path=|| "/login"
                        condition=move || user.get().map(|u| u.is_some())
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
