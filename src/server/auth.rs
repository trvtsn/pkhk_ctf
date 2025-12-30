use cfg_if::cfg_if;
cfg_if! {
    if #[cfg(feature = "ssr")] {
        use crate::server::AuthSession;
        use axum::response::{IntoResponse};
        use http::StatusCode;
    }
}

pub async fn logout_user(mut auth_session: AuthSession) -> impl IntoResponse {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            match auth_session.logout().await {
                Ok(_) => axum::response::Redirect::to("/").into_response(),
                Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            }
        } else {
            leptos_router::Redirect::to("/").into_response()
        }
    }
}

// use anyhow::anyhow;
// use argon2::{Argon2, PasswordHash, PasswordVerifier};
// use axum::{Router, extract::{Json, State}, http::{Response, StatusCode}, response::{IntoResponse, Redirect}, routing::{get, post}};
// use axum_extra::extract::cookie::{Cookie, CookieJar};
// use cfg_if::cfg_if;
// use chrono::{Local, NaiveDateTime};
// use leptos::{prelude::ServerFnError, server};
// use serde::Deserialize;
// use uuid::Uuid;

// use crate::{server::{db::{enums::UserIdentifier, structs::{Session, User}}, structs::{Backend, LoginPayload}}, state::AppState};

// pub mod structs {
//     use crate::server::db::enums::UserIdentifier;
//     use serde::{Deserialize, Serialize};

//     #[derive(Deserialize, Serialize)]
//     pub struct RegisterPayload {
//         pub email: String,
//         pub password: String
//     }

//     #[derive(Debug, Clone, Deserialize, Serialize)]
//     pub struct LoginPayload {
//         pub user_identifier: UserIdentifier,
//         pub password: String
//     }
// }

// pub mod ldap {
//     use axum::{response::IntoResponse, Router, routing::{get, post}};
//     use crate::state::AppState;

//     pub fn router() -> Router<AppState> {
//         Router::<AppState>::new()
//             .route("/ldap/login", post(self::post::login))
//             .route("/ldap/logout", get(self::get::logout))
//     }

//     mod post {
//         // use tracing::instrument;

//         // use super::*;

//         // #[instrument]
//         // pub async fn login(
//         //     auth_session: AuthSession,
//         //     session: Session,
//         //     Form(NextUrl { next }): Form<NextUrl>,
//         // ) -> impl IntoResponse {
//         //     let (auth_url, csrf_state) =
//         //         auth_session.backend.authorize_url();

//         //     session
//         //         .insert(CSRF_STATE_KEY, csrf_state.secret())
//         //         .await
//         //         .expect("Serialization should not fail.");

//         //     session
//         //         .insert(NEXT_URL_KEY, next)
//         //         .await
//         //         .expect("Serialization should not fail.");

//         //     Redirect::to(auth_url.as_str()).into_response()
//         // }

//         use crate::state::AppState;
//         use axum::{extract::State, response::IntoResponse};

//         pub async fn login(
//             State(state): State<AppState>,
//             axum_session::Session<SessionPgPool>
//             axum::Json(payload): axum::Json<LoginPayload>,
//         ) -> impl IntoResponse {
//             // implement login system for ldap
//             //
//             // validate credentials (very minimal here)
//             // let rec = sqlx::query!("SELECT id, password_hash FROM users WHERE email = $1", payload.email)
//             //     .fetch_optional(&state.pool)
//             //     .await
//             //     .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

//             // if rec.is_none() {
//             //     return (StatusCode::UNAUTHORIZED, "bad credentials").into_response();
//             // }
//             // let id = rec.unwrap().id as i64;

//             // // CRITICAL: store the user id in the session under a known key (we choose "user_id")
//             // // reason: axum_session persists arbitrary serde-serializable data server-side;
//             // // axum_session_auth / or custom code can read this key to know who's logged in.
//             // axum_session.set("user_id", id).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

//             // // mark session to be persisted (depending on session-mode this may be required)
//             // axum_session.set_store(true);

//             // (StatusCode::OK, "ok").into_response()
//         }
//     }

//     mod get {
//         use tracing::instrument;

//         use super::*;

//         #[instrument]
//         pub async fn logout(
//             mut auth_session: AuthSession,
//         ) -> impl IntoResponse {
//             match auth_session.logout().await {
//                 Ok(_) => Redirect::to("/login").into_response(),
//                 Err(_) => StatusCode::INTERNAL_SERVER_ERROR
//                     .into_response(),
//             }
//         }
//     }
// }

// // cfg_if! {
// //     if #[cfg(feature = "ssr")] {
// //         /// Hash Argon2 password
// //         pub fn hash_password(password: &[u8]) -> Result<String, BenwisAppError> {
// //             let argon2 = Argon2::default();
// //             let salt = SaltString::generate(&mut OsRng);
// //             let password_hash = argon2.hash_password(password, &salt)?.to_string();
// //             Ok(password_hash)
// //         }
// //
// //         /// Verify Password
// //         pub fn verify_password(password: &str, password2: &str) -> Result<(), BenwisAppError> {
// //             let argon2 = Argon2::default();
// //             // Verify password against PHC string
// //             let parsed_hash = PasswordHash::new(&password)?;
// //             Ok(argon2.verify_password(password2.as_bytes(), &parsed_hash)?)
// //         }
// //
// //         pub fn get_session_cookie_value(req_parts: &Parts)-> Result<Option<String>, BenwisAppError>{
// //             let cookie_jar = CookieJar::from_headers(&req_parts.headers);
// //             let session_cookie = match cookie_jar.get("benwis_session") {
// //                         Some(c) => Some(c.value().to_owned()),
// //                         None => None,
// //                     };
// //
// //             Ok(session_cookie)
// //         }
// //
// //         pub async fn auth_session(req_parts: &Parts, con: &SqlitePool)-> Result<User, BenwisAppError>{
// //             let store = expect_context::<SqliteStore>();
// //             let session_val = match get_session_cookie_value(req_parts)? {
// //                 Some(sv) => sv,
// //                 None => return Err(BenwisAppError::AuthError),
// //             };
// //
// //             let Some(session) = store.load_session(session_val).await? else{
// //                 return Err(BenwisAppError::InternalServerError);
// //             }; 
// //             let Some(user_id) = session.get("user_id") else{
// //                 return Err(BenwisAppError::AuthError);
// //             };
// //
// //             let user = match User::get(user_id, con).await{
// //                 Some(u) => u,
// //                 None => return Err(BenwisAppError::AuthError)
// //             };  
// //             Ok(user)
// //         }
// //
// //         /// Create a new Session and store User id in it
// //         pub async fn create_session(user_id: i64)-> Result<String, BenwisAppError>{
// //             let mut session = Session::new();
// //             session.insert("user_id", user_id)?;
// //
// //             let session_store = expect_context::<SqliteStore>();
// //             let cookie_value = session_store.store_session(session).await?.unwrap();
// //             Ok(cookie_value)
// //         }
// //
// //         /// Destroy the Session if it exists
// //         pub async fn logout_session(cookie_value: &str)-> Result<(), BenwisAppError>{
// //             let store = expect_context::<SqliteStore>();
// //             let session = match store.load_session(cookie_value.to_string()).await?{
// //                 Some(s) =>s,
// //                 None => return Ok(())
// //             };
// //             store.destroy_session(session).await?;
// //             Ok(())
// //         }
// //     }
// // }
// //
// //
// // // #[tracing::instrument(level = "info", fields(error), ret, err)]
// // // #[server(Login, "/api")]
// // // pub async fn login(
// // //     username: String,
// // //     password: String,
// // //     remember: Option<String>,
// // // ) -> Result<(), ServerFnError> {
// // //     let Some(parts) = use_context::<Parts>() else {
// // //         return Ok(());
// // //     };
// // //     let con = pool()?;
// // //     let user = auth_user(&username, &password, &con).await?;
// // //     let session_cookie = create_session(user.id).await?;
// //
// // //     let res_options = expect_context::<leptos_axum::ResponseOptions>();
// // //     let cookie_val = format!("benwis_session={session_cookie};Path=/;SameSite=Strict;");
// // //     res_options.insert_header(SET_COOKIE, HeaderValue::from_str(&cookie_val).unwrap());
// // //     leptos_axum::redirect("/");
// // //     Ok(())
// // // }
// //
// // // #[tracing::instrument(level = "info", fields(error), ret, err)]
// // // #[server(Signup, "/api")]
// // // pub async fn signup(
// // //     username: String,
// // //     display_name: String,
// // //     password: String,
// // //     password_confirmation: String,
// // //     remember: Option<String>,
// // // ) -> Result<(), ServerFnError> {
// // //     let pool = pool()?;
// //
// //
// // //     if password != password_confirmation {
// // //         return Err(ServerFnError::ServerError(
// // //             "Passwords did not match.".to_string(),
// // //         ));
// // //     }
// // //     // Don't want anyone signing up but me!
// // //     if username != "benwis" {
// // //         leptos_axum::redirect("/nedry");
// // //         return Ok(());
// // //     }
// //
// // //     let password_hashed = hash_password(password.as_bytes()).unwrap();
// //
// // //     sqlx::query("INSERT INTO users (username, display_name, password) VALUES (?,?, ?)")
// // //         .bind(username.clone())
// // //         .bind(display_name.clone())
// // //         .bind(password_hashed)
// // //         .execute(&pool)
// // //         .await?;
// //
// // //     let user = User::get_from_username(&username, &pool)
// // //         .await
// // //         .ok_or("Signup failed: User does not exist.")
// // //         .map_err(ServerFnError::new)?;
// //
// //
// // //     Ok(())
// // // }
// //
// //
// // // #[tracing::instrument(level = "info", fields(error), ret, err)]
// // // #[server(Logout, "/api")]
// // // pub async fn logout() -> Result<(), ServerFnError> {
// // //     println!("LOGGING OUT");
// // //     let Some(parts) = use_context::<Parts>() else {
// // //         return Ok(());
// // //     };
// // //     let con = pool()?;
// // //     let Some(session) = get_session_cookie_value(&parts)? else{
// // //         return Ok(());
// // //     };
// // //     logout_session(&session).await?;
// //
// // //     // Delete session cookie by expiring it
// // //     let res_parts = expect_context::<leptos_axum::ResponseOptions>();
// // //     res_parts.insert_header(SET_COOKIE, HeaderValue::from_static("benwis_session=no;Path=/; Expires=Thu, 01 Jan 1970 00:00:01 GMT;"));
// // //     res_parts.insert_header(SET_COOKIE, HeaderValue::from_static("sessionid=no;Path=/; Expires=Thu, 01 Jan 1970 00:00:01 GMT;"));
// // //     leptos_axum::redirect("/");
// //
// // //     Ok(())
// // // }
// //
// // #[server(Register, "/api")]
// // pub async fn register(email: String, password: String) -> Result<(), ServerFnError> {
// //     let mut user = User::default();
// //     let pw_hash = get_md5(password.clone()).await;
// //     user.email = email;
// //     user.pw_hash = pw_hash;
// //     user.created_at = Local::now().to_utc();
// //     user.last_active_at = Local::now().to_utc();
// //     user.role = "user".to_string();
// //
// //     user.add().await;
// //
// //     Ok(())
// // }
// //
// // #[server(Login, "/api")]
// // pub async fn login(
// //     State(state): State<AppState>, 
// //     jar: CookieJar,
// //     Json(payload): Json<LoginPayload>
// // ) -> Result<(), ServerFnError> {
// //     let db_user = match User::get(payload.user_identifier).await {
// //         Ok(user) => user,
// //         Err(e) => {
// //             Err(Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR))?
// //         }
// //     };
// //     let parsed = PasswordHash::new(&db_user.pw_hash).map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "hash parse error"))?;
// //     Argon2::default().verify_password(payload.password.as_bytes(), &parsed).map_err(|_| (StatusCode::UNAUTHORIZED, "bad credentials"))?;
// //
// //     let session_id = Uuid::new_v4().to_string();
// //     let created_at = chrono::Local::now().to_utc();
// //     let expires_at = created_at.clone() + chrono::Duration::seconds(state.session_ttl_secs);
// //
// //     let session = Session { id: session_id, user_id: db_user.id, expires_at, created_at };
// //     session.add(axum::extract::State(state));
// //
// //     let mut cookie = Cookie::build(("session", session_id.clone())).http_only(true).path("/").same_site(cookie::SameSite::Lax).build();
// //
// //     #[cfg(not(debug_assertions))]
// //     cookie.set_secure(true);
// //
// //     let jar = jar.add(cookie);
// //
// //     (jar, Redirect::to("/")).into_response()
// // }
// //
// // pub async fn get_current_user(State(state): State<AppState>, jar: CookieJar) -> Result<Option<User>, sqlx::Error> {
// //     if let Some(c) = jar.get("session") {
// //         let sid = c.value().to_string();
// //         return Ok(Session::get_user(sid, axum::extract::State(state)).await?);
// //     }
// //
// //     Ok(None)
// // }

// pub mod ldap;
// pub mod structs;

// pub fn router() -> Router<AppState> {
//     Router::<AppState>::new()
//         .route("/login", post(self::post::login))
//         .route("/logout", get(self::get::logout))
// }

// mod post {
//     use axum::Form;
//     use tracing::instrument;

//     use super::*;

//     #[instrument]
//     pub async fn login(
//         auth_session: AuthSession,
//         session: Session,
//         Form(NextUrl { next }): Form<NextUrl>,
//     ) -> impl IntoResponse {
//         let (auth_url, csrf_state) =
//             auth_session.backend.authorize_url();

//         session
//             .insert(CSRF_STATE_KEY, csrf_state.secret())
//             .await
//             .expect("Serialization should not fail.");

//         session
//             .insert(NEXT_URL_KEY, next)
//             .await
//             .expect("Serialization should not fail.");

//         Redirect::to(auth_url.as_str()).into_response()
//     }
// }

// mod get {
//     use tracing::instrument;

//     use super::*;

//     #[instrument]
//     pub async fn logout(
//         mut auth_session: AuthSession,
//     ) -> impl IntoResponse {
//         match auth_session.logout().await {
//             Ok(_) => Redirect::to("/login").into_response(),
//             Err(_) => StatusCode::INTERNAL_SERVER_ERROR
//                 .into_response(),
//         }
//     }
// }

