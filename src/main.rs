use cfg_if::cfg_if;
use pkhk_ctf::server::auth::logout_user;

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use axum::{
            body::Body as AxumBody, 
            extract::{Path, State}, 
            http::Request, 
            response::{IntoResponse, Response}, 
            routing::get
        };
        use leptos::prelude::provide_context;
        use leptos_axum::handle_server_fns_with_context;
        use pkhk_ctf::{app::shell, logging::{init_tracing, logs_sse}};
        use pkhk_ctf::server::{backend::structs::Backend, db::get_db, structs::AppState, init_env};

        pub type AuthSession = axum_login::AuthSession<Backend>;
    }
}

#[cfg(feature = "ssr")]
async fn server_fn_handler(
    auth_session: AuthSession,
    State(app_state): State<AppState>,
    //Path(path): Path<String>,
    request: Request<AxumBody>,
) -> impl IntoResponse {
    handle_server_fns_with_context(
        move || {
            provide_context(auth_session.clone());
            provide_context(app_state.pool.clone());
        },
        request,
    )
    .await
}

#[cfg(feature = "ssr")]
async fn leptos_routes_handler(
    auth_session: AuthSession,
    State(app_state): State<AppState>,
    //Path(path): Path<String>,
    req: Request<AxumBody>,
) -> Response {
    let state = app_state.clone();
    let handler = leptos_axum::render_route_with_context(
        app_state.routes.clone(),
        move || {
            provide_context(auth_session.clone());
            provide_context(app_state.pool.clone());
        },
        move || shell(app_state.leptos_options.clone()),
    );
    handler(State(state), req).await.into_response()
}

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use axum_login::{AuthManagerLayerBuilder, login_required, tower_sessions::{Expiry, SessionManagerLayer}};
    use leptos::{logging::log, prelude::*};
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use pkhk_ctf::{app::*, pages, server::{backend::structs::Backend, db}};
    use tower_sessions_sqlx_store::MySqlStore;

    _ = init_tracing();
    _ = init_env();
    _ = db::init_db().await;
    let pool = get_db();

    tracing::info!(app = "tracing-sse-logs", version = "0.1.0", "server starting");
    tracing::debug!(details = "this is debug detail", "startup debug");

    let session_store = MySqlStore::new(pool.clone()).with_schema_name("ctfpkhk").unwrap().with_table_name("sessions").unwrap();
    _ = session_store.migrate().await;
    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(time::Duration::seconds(86400)));

    let backend = Backend::new(pool.clone());
    let auth_session_layer = AuthManagerLayerBuilder::new(
        backend,
        session_layer,
    )
    .build();

    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);
    
    let app_state = AppState {
        leptos_options,
        pool: pool.clone(),
        routes: routes.clone(),
    };

    let app = Router::new()
        .route(
            "/api/{*fn_name}",
            get(server_fn_handler).post(server_fn_handler),
        )
        .route(
            "/logout",
            get(logout_user)
        )
        .route(
            "/admin/logs", 
            get(logs_sse)
        )
        .leptos_routes_with_handler(
            routes, 
            get(leptos_routes_handler)
        )
        .fallback(
            leptos_axum::file_and_error_handler::<AppState, _>(shell)
        )
        // .merge(pages::user::router())
        .layer(auth_session_layer)
        .with_state(app_state);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    log!("listening on http://{}", &addr);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
}
