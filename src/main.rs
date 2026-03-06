use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use axum::{
            body::Body as AxumBody, 
            extract::State, 
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
    use axum_login::{AuthManagerLayerBuilder, tower_sessions::{Expiry, SessionManagerLayer}};
    use leptos::{logging::log, prelude::*};
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use pkhk_ctf::{app::*, server::{self, backend::structs::Backend, db}};
    use tower_sessions_sqlx_store::MySqlStore;

    init_tracing();
    init_env();
    _ = db::init_db().await;
    _ = server::proxmox::create_user_role().await;
    _ = server::proxmox::create_realm().await;
    let pool = get_db();

    tracing::info!(app = "tracing-sse-logs", version = "0.1.0", "server starting");
    tracing::debug!(details = "this is debug detail", "startup debug");

    let session_store = match MySqlStore::new(pool.clone()).with_schema_name("ctfpkhk") {
        Ok(store) => match store.with_table_name("sessions") {
            Ok(store) => store,
            Err(e) => {
                tracing::error!("Failed to create MySqlStore with table name: \"sessions\": {e}");
                return
            }
        },
        Err(e) => {
            tracing::error!("Failed to create MySqlStore with schema name: \"ctfpkhk\": {e}");
            return
        }
    };
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

    let conf = match get_configuration(None) {
        Ok(conf) => conf,
        Err(e) => {
            tracing::error!("Failed get_configuration: {e}");
            return
        }
    };
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);
    
    let app_state = AppState {
        leptos_options,
        pool,
        routes: routes.clone(),
    };

    let app = Router::new()
        .route(
            "/api/{*fn_name}",
            get(server_fn_handler).post(server_fn_handler),
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
        .merge(pkhk_ctf::server::router())
        .layer(auth_session_layer)
        .with_state(app_state);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(listener) => listener,
        Err(e) => { 
            tracing::error!("Failed to bind TcpListener on {addr:?}: {e}");
            return
        }
    };
    log!("Listening on http://{}", &addr);
    match axum::serve(listener, app.into_make_service()).await {
        Ok(_) => {},
        Err(e) => {
            tracing::error!("Failed to serve app on TcpListener: {e}");
            return
        }
    }
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
}
