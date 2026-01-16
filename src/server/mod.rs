#[cfg(feature = "ssr")]
use crate::server::{backend::{AuthSession, structs::{Credentials}, hash_string, verify_hash}};
use crate::{error_template::AppError, server::{db::{enums::{UserIdentifier, UserRole}, structs::{ChallengeWithAttachments, DbUser, Event}}, enums::ResultStatus, structs::{ApiResult, LeaderboardData, PivotRow, User}}, utils::offset_to_datetime};
#[cfg(feature = "ssr")]
use axum::extract::Path;
#[cfg(feature = "ssr")]
use axum_login::AuthnBackend;
use cfg_if::cfg_if;
use chrono::{DateTime, Local};
use leptos::{prelude::{expect_context, use_context}, server, server_fn::codec::{MultipartData, MultipartFormData}};
#[cfg(feature = "ssr")]
use leptos_axum::ResponseOptions;
use tracing::instrument;
use std::collections::{BTreeSet, HashMap};
#[cfg(feature = "ssr")]
use sqlx::MySqlPool;
#[cfg(feature = "ssr")]
use axum::{response::IntoResponse, http::{StatusCode, header}};
use crate::server::{enums::AdminEventPayloadKind, db::enums::AttachmentIdentifier};

pub mod admin;
#[cfg(feature = "ssr")]
pub mod auth;
pub mod backend;
pub mod db;
pub mod structs {
    use crate::server::{enums::ResultStatus};
    use chrono::{DateTime, Local};
    use leptos::prelude::LeptosOptions;
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            use axum::extract::FromRef;
            use leptos_axum::AxumRouteListing;
            use sqlx::MySqlPool;

            #[derive(FromRef, Debug, Clone)]
            pub struct AppState {
                pub leptos_options: LeptosOptions,
                pub pool: MySqlPool,
                pub routes: Vec<AxumRouteListing>,
            }
        }
    }

    #[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
    pub struct User {
        /// The database id for this user
        pub id: String,
        /// This is computed with Argon2id, but it's only a *piece* of the entire thing returned
        /// by the hash function. You should be able to use whatever you want here as long as you
        /// can keep it stable between page loads. Personally, I don't like using the password hash
        /// but that's how they do it in the example so it's probably fine.
        pub session_auth_hash: Vec<u8>,
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct ApiResult<T> {
        pub result: ResultStatus,
        pub details: T
    }

    #[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
    pub struct PivotRow {
        pub ts: DateTime<Local>,
        pub values: HashMap<String, f64>,
    }

    #[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
    pub struct LeaderboardData {
        pub event_name: String,
        pub x_min: DateTime<Local>,
        pub x_max: DateTime<Local>,
        pub y_max: f64,
        pub users: Vec<String>,
        pub rows: Vec<PivotRow>,
    }
}
pub mod enums {
    use serde::{Deserialize, Serialize};
    use std::str::FromStr;
    
    #[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
    #[serde(rename_all = "lowercase")]
    pub enum ResultStatus {
        Success,
        Fail
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub enum AdminEventPayloadKind {
        ChallengeDeleted,
        ChallengeEdited,
        NewChallengeCreated,
        NewEventCreated,
        EventDeleted,
        EventEdited,
        UserCreated,
        UserDeleted,
        UserEdited,
        ChallengeSolved
    }

    impl FromStr for AdminEventPayloadKind {
        type Err = ();

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "ChallengeDeleted" => Ok(AdminEventPayloadKind::ChallengeDeleted),
                "ChallengeEdited"  => Ok(AdminEventPayloadKind::ChallengeEdited),
                "NewChallengeCreated" => Ok(AdminEventPayloadKind::NewChallengeCreated),
                "NewEventCreated" => Ok(AdminEventPayloadKind::NewEventCreated),
                "EventDeleted" => Ok(AdminEventPayloadKind::EventDeleted),
                "EventEdited" => Ok(AdminEventPayloadKind::EventEdited),
                "UserCreated" => Ok(AdminEventPayloadKind::UserCreated),
                "UserDeleted" => Ok(AdminEventPayloadKind::UserDeleted),
                "UserEdited" => Ok(AdminEventPayloadKind::UserEdited),
                "ChallengeSolved" => Ok(AdminEventPayloadKind::ChallengeSolved),
                _ => Err(()),
            }
        }
    }
}

#[cfg(feature = "ssr")]
pub fn pool() -> Result<MySqlPool, AppError> {
    use_context::<MySqlPool>().ok_or_else(|| {
        AppError::DatabaseError("Pool missing.".to_string())
    })
}

#[cfg(feature = "ssr")]
pub fn init_env() {
    dotenvy::dotenv().ok();
}

#[server(name=Challenges, prefix="/api", endpoint="challenges")]
#[instrument]
pub async fn get_all_challenges_with_attachments() -> Result<Vec<ChallengeWithAttachments>, AppError> {
    cfg_if! {
        if #[cfg(feature = "ssr")] {
            let auth = use_context::<AuthSession>().unwrap();
            let challenges = match db::structs::Challenge::get_all(&auth.backend.pool).await {
                Ok(challenges) => challenges,
                Err(e) => Err(e)?
            };
            let mut cwa: Vec<ChallengeWithAttachments> = Vec::new();
            for challenge in challenges {
                let attachments = db::structs::AttachmentWithoutBlob::get_all(&Some(AttachmentIdentifier::ChallengeId(challenge.id.clone())), &auth.backend.pool).await?;
                cwa.push(ChallengeWithAttachments { challenge, attachments });
            }
            Ok(cwa)
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[server(name=Leaderboard, prefix="/api", endpoint="leaderboard")]
#[instrument]
pub async fn build_leaderboard_data() -> Result<LeaderboardData, AppError> {
    cfg_if! {
        if #[cfg(feature = "ssr")] {
            let auth = use_context::<AuthSession>().unwrap();
            let active_event_id = match get_active_events().await {
                Ok(active_events) => active_events.first().unwrap().id.clone(),
                Err(e) => {
                    tracing::error!(error = ?e);
                    "05ea8233-b9f1-4b29-bc3a-025161bddf6d".to_string() // perpetual event in db
                }
            };

            let meta = match db::structs::Event::get_metadata(&active_event_id, &auth.backend.pool).await {
                Ok(meta) => meta,
                Err(e) => Err(e)?
            };

            let event_name = meta.name.unwrap_or("".to_string());
            let x_min = offset_to_datetime(meta.first_submission.unwrap());
            let x_max = offset_to_datetime(meta.last_submission.unwrap());

            let y_max = db::structs::Event::get_total_possible_points(&active_event_id, &auth.backend.pool).await.unwrap();

            let solves = sqlx::query!(
                r#"
                WITH first_solves AS (
                    SELECT user_id, challenge_id, MIN(solved_at) AS solved_at
                    FROM submissions
                    WHERE event_id = ?
                    GROUP BY user_id, challenge_id
                )
                SELECT fs.user_id, u.username, fs.solved_at, c.points
                FROM first_solves fs
                JOIN challenges c ON c.id = fs.challenge_id
                JOIN users u ON u.id = fs.user_id
                ORDER BY fs.solved_at
                "#,
                active_event_id
            )
            .fetch_all(&auth.backend.pool)
            .await?;

            let users: Vec<String> = solves.iter().map(|r| r.username.clone()).collect();

            let mut timestamps = BTreeSet::new();

            #[derive(Debug)]
            struct Solve { username: String, ts: DateTime<Local>, points: f64 }

            let mut solves_parsed: Vec<Solve> = Vec::new();
            for r in solves {
                let ts = match r.solved_at {
                    Some(ts) => offset_to_datetime(ts),
                    None => chrono::Local::now()
                };
                timestamps.insert(ts);
                solves_parsed.push(Solve {
                    username: r.username,
                    ts,
                    points: r.points as f64,
                });
            }

            let mut times: Vec<DateTime<Local>> = timestamps.into_iter().collect();
            times.sort();

            let mut user_cumulative: HashMap<String, f64> = users.iter().map(|u| (u.clone(), 0.0)).collect();
            let mut solves_by_ts: HashMap<DateTime<Local>, Vec<&Solve>> = HashMap::new();
            for s in &solves_parsed {
                solves_by_ts.entry(s.ts).or_default().push(s);
            }

            let mut rows: Vec<PivotRow> = Vec::new();
            for ts in times {
                if let Some(slist) = solves_by_ts.get(&ts) {
                    for s in slist {
                        if let Some(v) = user_cumulative.get_mut(&s.username) {
                            *v += s.points;
                        } else {
                            user_cumulative.insert(s.username.clone(), s.points);
                        }
                    }
                }
                let mut values = HashMap::new();
                for u in &users {
                    values.insert(u.clone(), *user_cumulative.get(u).unwrap_or(&0.0_f64));
                }
                rows.push(PivotRow { ts, values });
            }

            Ok(LeaderboardData { event_name, x_min, x_max, y_max: y_max as f64, users, rows })
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[server(name=LoginUser, prefix="/api", endpoint="login")]
#[instrument(skip(password))]
pub async fn login_user(email: String, password: String) -> Result<ApiResult<Option<User>>, AppError> { // impl IntoResponse (can serve 403 that way)
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            // Note that you can still use `leptos_axum::extract().await?` if you want, but since we
            // called `provide_context` from the `server_fn_handler` in `main`, we can do it this way
            // and it feels faster. Get the AuthSession.
            let mut auth = use_context::<AuthSession>().unwrap();

            // The SqliteBackend we defined has the `Self::Credential` type set to a `(String,String)` tuple
            // which is meant to be the username/password pair. This is just an example, you probably want
            // something more robust to handle different auth scenarios like Oauth and whatnot. Maybe I'll add
            // those in later if I can figure out how.
            let creds = Credentials { user_identifier: UserIdentifier::Email(email), password };
            let user: Option<User> = auth.backend.authenticate(creds).await?;

            // If the authentication was successful, we actually have to tell the AuthSession that the user
            // is now logged in. This happens when we call `auth.login(user)`. This will also be the first
            // place where you actually get a session id sent back to the browser unless you've done other stuff
            // with your sessions elsewhere.
            if let Some(user) = user.as_ref() {
                match auth.login(user).await {
                    Ok(_) => Ok(ApiResult { result: ResultStatus::Success, details: Some(user.clone()) }), // update last_active_date in db
                    Err(e) => {
                        tracing::error!(error = ?e);
                        Err(AppError::InternalError("internal error".to_string()))
                    }
                }
            } else {
                Ok(ApiResult { result: ResultStatus::Fail, details: None })
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[server(name=GetUser, prefix="/api", endpoint="user")]
#[instrument]
pub async fn get_user() -> Result<Option<User>, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let response = expect_context::<ResponseOptions>();
            match use_context::<AuthSession>() {
                Some(session) => Ok(session.user.clone()),
                None => {
                    response.set_status(StatusCode::FORBIDDEN);
                    Ok(None)
                }
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[server(name=GetUserPoints, prefix="/api/user", endpoint="points")]
#[instrument]
pub async fn get_user_points() -> Result<u32, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let auth: AuthSession = use_context().unwrap();
            let response = expect_context::<ResponseOptions>();
            let user = match auth.user {
                Some(user) => user,
                None => {
                    response.set_status(StatusCode::FORBIDDEN);
                    return Err(AppError::Forbidden);
                }
            };
            match db::structs::Submission::get_user_points(&user.id, &auth.backend.pool).await {
                Ok(points) => Ok(points),
                Err(e) => {
                    tracing::error!(error = ?e);
                    Err(AppError::InternalError("internal error".to_string()))
                }
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[server(name=GetDbUser, prefix="/api/user", endpoint="info")]
#[instrument]
pub async fn get_db_user(username: Option<String>) -> Result<Option<DbUser>, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let auth = use_context::<AuthSession>().unwrap();
            let response = expect_context::<ResponseOptions>();
            let user = match auth.user {
                Some(user) => user,
                None => {
                    response.set_status(StatusCode::FORBIDDEN);
                    return Err(AppError::Forbidden);
                }
            };
            if username.is_some() {
                match DbUser::get(&UserIdentifier::Username(username.unwrap_or_default().clone()), &auth.backend.pool).await {
                    Ok(user) => Ok(user),
                    Err(e) => {
                        tracing::error!(error = ?e);
                        Err(AppError::InternalError("internal error".to_string()))
                    }
                }    
            } else {
                match DbUser::get(&UserIdentifier::Id(user.id.clone()), &auth.backend.pool).await {
                    Ok(user) => Ok(user),
                    Err(e) => {
                        tracing::error!(error = ?e);
                        Err(AppError::InternalError("internal error".to_string()))
                    }
                }
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

/// Add a user to the database and log them in, because I get annoyed by sites that let me register and then
/// make me log in separately after that. Give me a break! This function is called from the Register component
/// which is in pages/register.rs.
#[server(name=Register, prefix="/api", endpoint="register")]
#[instrument(skip(password))]
pub async fn register_user(email: String, password: String, confirm_password: String) -> Result<ApiResult<Option<User>>, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let mut auth_session: AuthSession = use_context().expect("auth-session not provided");

            if password != confirm_password {
                return Err(AppError::BadRequest("password and confirm password must be the same".to_string()));
            }

            let user: Option<User> = auth_session.backend.add_user(email.clone(), password).await?;

            if let Some(user) = user {
                // Tell the AuthSession that we're logged-in now and it should behave accordingly. This will set the
                // session id and send it to the browser as a side-effect (before now you likely had no session id in the browser).
                auth_session.login(&user).await?;
                Ok(ApiResult { result: ResultStatus::Success, details: Some(user) })
            } else {
                Err(AppError::InternalError("".to_string()))
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[server(name=CheckFlag, prefix="/api", endpoint="check_flag")]
#[instrument(skip(challenge))]
pub async fn check_flag(flag: String, challenge: crate::server::db::structs::Challenge) -> Result<ApiResult<String>, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let auth = use_context::<AuthSession>().unwrap();
            let response = expect_context::<ResponseOptions>();
            let user = match auth.user {
                Some(user) => user,
                None => {
                    response.set_status(StatusCode::FORBIDDEN);
                    return Err(AppError::Forbidden);
                }
            };

            let mut tx = auth.backend.pool.begin().await?;

            // check if the challenge is already solved, and if so, return Error
            match db::structs::Submission::get_user_solved_challenges(&user.id, &mut *tx).await {
                Ok(solved) => {
                    if solved.contains(&challenge.id) {
                        return Ok(ApiResult { result: ResultStatus::Fail, details: "challenge already solved".to_string() });
                    }
                },
                Err(e) => {
                    tracing::error!(error = ?e);
                    tx.rollback().await?;
                    return Err(AppError::InternalError("failed to check flag".to_string()));
                }
            }

            let challenge_flag_hash = match db::structs::Challenge::get_flag_hash(&challenge.id, &mut *tx).await {
                Ok(flag_hash) => flag_hash,
                Err(e) => {
                    tracing::error!(error = ?e);
                    tx.rollback().await?;
                    return Err(AppError::InternalError("Failed to get flag hash".to_string()));
                }
            };

            if let Ok(()) = verify_hash(flag.clone(), challenge_flag_hash) {                
                match db::structs::Submission::add(&challenge.id, &challenge.event_id, &user.id, &challenge.points, &chrono::Local::now(), &mut *tx).await {
                    Ok(_) => {
                        tx.commit().await?;
                        _ = build_and_broadcast(AdminEventPayloadKind::ChallengeSolved).await;
                        Ok(ApiResult { result: ResultStatus::Success, details: "correct solution".to_string() })
                    }
                    Err(e) => {
                        tx.rollback().await?;
                        Err(e.into())
                    }
                }
            } else {
                Ok(ApiResult { result: ResultStatus::Fail, details: "incorrect solution".to_string() })
            }

        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[server(name=EditUsername, prefix="/api/user", endpoint="username")]
#[instrument]
pub async fn edit_username(username: String) -> Result<ApiResult<String>, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let auth = use_context::<AuthSession>().unwrap();
            let response = expect_context::<ResponseOptions>();
            let user = match auth.user {
                Some(user) => user,
                None => {
                    response.set_status(StatusCode::FORBIDDEN);
                    return Err(AppError::Forbidden);
                }
            };
            match DbUser::edit_username(&user.id, &username, &auth.backend.pool).await {
                Ok(_) => Ok(ApiResult { result: ResultStatus::Success, details: "changed username".to_string() }),
                Err(e) => Err(e.into())
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[server(input=MultipartFormData, name=EditAvatar, prefix="/api/user", endpoint="edit_avatar")]
#[instrument(skip(avatar))]
pub async fn edit_avatar(avatar: MultipartData) -> Result<ApiResult<String>, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let auth = use_context::<AuthSession>().unwrap();
            let response = expect_context::<ResponseOptions>();
            let user = match auth.user {
                Some(user) => user,
                None => {
                    response.set_status(StatusCode::FORBIDDEN);
                    return Err(AppError::Forbidden);
                }
            };
            
            let mut data = avatar.into_inner().unwrap();
            let mut file_name = String::new();
            let mut file_blob = Vec::<u8>::new();
            let mut mime_type = String::new();
            while let Ok(Some(mut field)) = data.next_field().await {
                file_name = field.file_name().unwrap().to_string();
                mime_type = field.content_type().unwrap().to_string();

                while let Ok(Some(chunk)) = field.chunk().await {
                    file_blob.append(&mut chunk.to_vec());
                }
            }

            match DbUser::edit_avatar(&user.id, &file_name, &file_blob, &mime_type, &auth.backend.pool).await {
                Ok(_) => Ok(ApiResult { result: ResultStatus::Success, details: "changed avatar".to_string() }),
                Err(e) => Err(e.into())
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[server(name=GetAvatar, prefix="/api/user", endpoint="avatar")]
#[instrument]
pub async fn get_avatar(username: String) -> Result<Vec<u8>, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let auth = use_context::<AuthSession>().unwrap();
            let response = expect_context::<ResponseOptions>();
            let user = match auth.user {
                Some(user) => user,
                None => {
                    response.set_status(StatusCode::FORBIDDEN);
                    return Err(AppError::Forbidden);
                }
            };

            match DbUser::get_avatar(&UserIdentifier::Id(user.id.clone()), &auth.backend.pool).await {
                Ok(avatar) => Ok(avatar),
                Err(e) => Err(e.into())
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[server(name=SolvedChallenges, prefix="/api/challenges", endpoint="solved")]
#[instrument]
pub async fn get_user_solved_challenges() -> Result<Vec<String>, AppError> {
    cfg_if! {
        if #[cfg(feature = "ssr")] {
            let auth = use_context::<AuthSession>().unwrap();
            let response = expect_context::<ResponseOptions>();
            let user = match auth.user {
                Some(user) => user,
                None => {
                    response.set_status(StatusCode::FORBIDDEN);
                    return Err(AppError::Forbidden);
                }
            };
            match db::structs::Submission::get_user_solved_challenges(&user.id, &auth.backend.pool).await {
                Ok(solved) => Ok(solved),
                Err(e) => Err(e.into())
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[cfg(feature = "ssr")]
#[instrument(skip(auth_session))]
pub async fn download_blob(
    auth_session: AuthSession,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match auth_session.user {
        Some(_) => {},
        None => return (StatusCode::FORBIDDEN).into_response(),
    } 

    let pool = auth_session.backend.pool;
    let file = match db::structs::Attachment::get(AttachmentIdentifier::Id(id), &pool).await {
        Ok(Some(f)) => f,
        Ok(None) => return (StatusCode::NOT_FOUND).into_response(),
        Err(e) => {
            tracing::error!(error = ?e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "db error").into_response();
        }
    };

    let bytes = file.file_blob;
    let disposition = format!(
        "attachment; filename=\"{}\"",
        // sanitize(&filename)
        file.file_name
    );

    (
        [
            (header::CONTENT_TYPE, "application/octet-stream".into()),
            (header::CONTENT_DISPOSITION, disposition),
        ],
        bytes,
    ).into_response()
}

#[server(name=GetActiveEvents, prefix="/api", endpoint="active_events")]
#[instrument]
pub async fn get_active_events() -> Result<Vec<Event>, AppError> {
    cfg_if! {
        if #[cfg(feature = "ssr")] {
            let auth = use_context::<AuthSession>().unwrap();
            let events = match db::structs::Event::get_all(&auth.backend.pool).await {
                Ok(events) => events,
                Err(e) => return Err(e.into())
            };

            let mut active_events = Vec::new();
            let now = chrono::Local::now();
            for event in events.into_iter() {
                if now >= event.start_at && now <= event.end_at {
                    active_events.push(event);
                } 
            }

            Ok(active_events)
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[server(name=EditPassword, prefix="/api/user", endpoint="password")]
#[instrument(skip(old_password, new_password))]
pub async fn edit_password(old_password: String, new_password: String) -> Result<ApiResult<String>, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let auth = use_context::<AuthSession>().unwrap();
            let response = expect_context::<ResponseOptions>();
            let user = match auth.user {
                Some(user) => user,
                None => {
                    response.set_status(StatusCode::FORBIDDEN);
                    return Err(AppError::Forbidden);
                }
            };

            if old_password == new_password {
                return Ok(ApiResult { result: ResultStatus::Fail, details: "new password is same as old password".to_string() });
            }

            let pw_hash = hash_string(new_password.clone())?;

            match DbUser::edit_password(&user.id, &pw_hash, &auth.backend.pool).await {
                Ok(_) => Ok(ApiResult { result: ResultStatus::Success, details: "changed password".to_string() }),
                Err(e) => Err(e.into())
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[server(name=UserExists, prefix="/api",endpoint="user_exists")]
pub async fn user_exists(email: String) -> Result<bool, AppError> {
    let auth = use_context::<AuthSession>().unwrap();

    match DbUser::is_user_available(&email, &auth.backend.pool).await {
        Ok(result) => Ok(result),
        Err(e) => Err(e.into())
    }
}

#[server(name=LogoutUser, prefix="/api/",endpoint="logout")]
pub async fn logout_user() -> Result<(), AppError> {
    let mut auth = use_context::<AuthSession>().unwrap();

    match auth.logout().await {
        Ok(_) => Ok(()),
        Err(e) => Err(e.into())
    }
}

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use axum::response::sse;
        // use chrono::{Local, DateTime};
        use futures::stream::{Stream, StreamExt};
        use once_cell::sync::Lazy;
        use serde::{Serialize, Deserialize};
        use std::{convert::Infallible, fmt::Debug};
        use tokio::sync::broadcast;
        use tokio_stream::wrappers::BroadcastStream;

        pub static ADMIN_TX: Lazy<broadcast::Sender<String>> = Lazy::new(|| {
            broadcast::channel::<String>(1024).0
        });

        #[derive(Debug, Serialize, Deserialize)]
        pub struct AdminEventPayload {
            kind: AdminEventPayloadKind,
            // timestamp: DateTime<Local>
        }

        #[instrument]
        pub async fn admin_sse() -> sse::Sse<impl Stream<Item = Result<sse::Event, Infallible>>> {
            let rx = ADMIN_TX.subscribe();

            let stream = BroadcastStream::new(rx)
                .filter_map(|res| async move { res.ok() })
                .map(|msg: String| sse::Event::default().data(msg))
                .map(Ok);

            sse::Sse::new(stream)
        }

        #[instrument]
        pub async fn build_and_broadcast(payload_kind: AdminEventPayloadKind) -> Result<(), AppError> {
            let payload = AdminEventPayload {
                kind: payload_kind,
            };
            match serde_json::to_string(&payload) {
                Ok(json) => {
                    if let Err(e) = ADMIN_TX.send(json) {
                        tracing::warn!(error = ?e, "admin event broadcast failed");
                        return Err(AppError::InternalError(e.to_string()));
                    }

                    Ok(())
                }
                Err(e) => {
                    tracing::error!(error = ?e, "serializing admin event failed");
                    Err(AppError::InternalError(e.to_string()))
                }
            }
        }
    }
}
