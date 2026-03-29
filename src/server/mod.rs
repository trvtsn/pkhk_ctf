/// src/server/mod.rs 
/// contains code that is crucial for the server to essentially, operate.
/// It's the root module for all server-side logic.
/// Anything that isn't gated behind a `#[cfg(feature = "ssr")]` flag is needed for the client to communicate with the server.
/// Anything that is behind a `#[cfg(feature = "ssr")]` flag should only be run on the server.

#[cfg(feature = "ssr")]
use crate::server::{backend::{AuthSession, hash_string}, db::get_db, structs::AppState};
use crate::{error_template::AppError, server::{db::{enums::{FileType, UserIdentifier, UserRole}, structs::{AttachmentWithoutBlob, ChallengeWithAttachments, DbUser, EventWithAttachments}}, enums::ServerEventPayload, structs::{StatusData, User}}, utils::get_context};
#[cfg(feature = "ssr")]
use axum::{extract::Path, Router, routing::get};
use cfg_if::cfg_if;
#[cfg(feature = "ssr")]
use leptos_axum::ResponseOptions;
use tracing::instrument;
#[cfg(feature = "ssr")]
use tokio::net::TcpStream;
use std::time::Duration;
#[cfg(feature = "ssr")]
use sqlx::MySqlPool;
#[cfg(feature = "ssr")]
use axum::{response::IntoResponse, http::{StatusCode, header}};
use crate::server::db::enums::AttachmentIdentifier;

pub mod admin;
pub mod api;
pub mod backend;
pub mod db;
pub mod proxmox;
pub mod structs {
    use crate::server::{backend::enums::AuthType, db::enums::UserIdentifier, enums::ResultStatus};
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
    pub struct Credentials {
        pub user_identifier: UserIdentifier,
        pub password: String,
        pub auth_type: AuthType
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

    #[derive(Debug, Clone, Deserialize, Serialize, Default)]
    pub struct StatusData {
        pub uptime: String,
        pub active_users: u32,
        pub cpu_usage: f32,
        pub ram_usage: f32,
        pub ram_usage_mb: f32,
        pub traffic: String,
    }
}

pub mod enums {
    use crate::server::{db::structs::{AttachmentWithoutBlob, ChallengeWithAttachments, DbUser, EventWithAttachments}, structs::StatusData};
    use serde::{Deserialize, Serialize};
    
    #[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
    #[serde(rename_all = "lowercase")]
    pub enum ResultStatus {
        Success,
        Fail
    }

    #[derive(Debug, Serialize, Deserialize)]
    pub enum ServerEventPayload {
        ChallengeDeleted(String),
        ChallengeEdited(ChallengeWithAttachments),
        NewChallengeCreated(ChallengeWithAttachments),
        NewEventCreated(EventWithAttachments),
        EventDeleted(String),
        EventEdited(EventWithAttachments),
        UserCreated(DbUser),
        UserDeleted(String),
        UserEdited(DbUser),
        ChallengeSolved,
        StatusData(StatusData),
        FilesUploaded(Vec<AttachmentWithoutBlob>),
        FileRenamed((String, String))
    }
}

#[cfg(feature = "ssr")]
mod file_cache {
    use moka::future::Cache;
    use once_cell::sync::Lazy;
    use std::time::Duration;

    #[derive(Clone)]
    pub struct CachedFile {
        pub bytes: Vec<u8>,
        pub content_type: Option<String>,
        pub file_name: String,
    }

    static CACHE: Lazy<Cache<String, CachedFile>> = Lazy::new(|| {
        Cache::builder()
            .max_capacity(200)
            .time_to_live(Duration::from_secs(3600))
            .build()
    });

    pub async fn get(id: &str) -> Option<CachedFile> {
        CACHE.get(id).await
    }

    pub async fn insert(id: String, file: CachedFile) {
        CACHE.insert(id, file).await;
    }

    pub async fn remove(id: &str) {
        CACHE.remove(id).await;
    }
}

#[cfg(feature = "ssr")]
pub async fn invalidate_file_cache(id: &str) {
    file_cache::remove(id).await;
}

#[cfg(feature = "ssr")]
pub fn pool() -> Result<MySqlPool, AppError> {
    Ok(get_context::<MySqlPool>()?)
}

#[cfg(feature = "ssr")]
pub fn init_env() {
    dotenvy::dotenv().ok();
}

#[cfg(feature = "ssr")]
pub fn router() -> Router<AppState> {
    Router::<AppState>::new()
        .route("/events", get(events_sse))
        .route("/file/{id}", get(download_blob))
        .route("/avatar/{id}", get(serve_image))
        .route("/image/{id}", get(serve_image))
}

#[cfg(feature = "ssr")]
#[instrument(skip(auth_session, headers))]
pub async fn download_blob(
    auth_session: AuthSession,
    headers: axum::http::HeaderMap,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match auth_session.user {
        Some(_) => {},
        None => return (StatusCode::FORBIDDEN).into_response(),
    }

    // if the browser already has this version cached, return 304
    let etag = format!("\"{}\"", id);
    if let Some(if_none_match) = headers.get(header::IF_NONE_MATCH) {
        if if_none_match.as_bytes() == etag.as_bytes() {
            return StatusCode::NOT_MODIFIED.into_response();
        }
    }

    let cached = file_cache::get(&id).await;
    let (bytes, file_name) = if let Some(cached) = cached {
        (cached.bytes, cached.file_name)
    } else {
        let pool = auth_session.backend.pool;
        let file = match db::structs::Attachment::get(AttachmentIdentifier::Id(id.clone()), &pool).await {
            Ok(Some(f)) => f,
            Ok(None) => return (StatusCode::NOT_FOUND).into_response(),
            Err(e) => {
                tracing::error!(error = ?e);
                return (StatusCode::INTERNAL_SERVER_ERROR, "db error").into_response();
            }
        };

        let entry = file_cache::CachedFile {
            bytes: file.file_blob,
            content_type: file.mime_type,
            file_name: file.file_name,
        };
        let result = (entry.bytes.clone(), entry.file_name.clone());
        file_cache::insert(id.clone(), entry).await;
        result
    };

    let disposition = format!(
        "attachment; filename=\"{}\"",
        file_name
    );

    (
        [
            (header::CONTENT_TYPE, "application/octet-stream".to_string()),
            (header::CONTENT_DISPOSITION, disposition),
            (header::ETAG, etag),
            (header::CACHE_CONTROL, "private, max-age=3600".to_string()),
        ],
        bytes,
    ).into_response()
}

#[cfg(feature = "ssr")]
#[instrument(skip(auth_session, headers))]
pub async fn serve_image(
    auth_session: AuthSession,
    headers: axum::http::HeaderMap,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match auth_session.user {
        Some(_) => {},
        None => return (StatusCode::FORBIDDEN).into_response(),
    }

    // if the browser already has this version cached, return 304
    let etag = format!("\"{}\"", id);
    if let Some(if_none_match) = headers.get(header::IF_NONE_MATCH) {
        if if_none_match.as_bytes() == etag.as_bytes() {
            return StatusCode::NOT_MODIFIED.into_response();
        }
    }

    let cached = file_cache::get(&id).await;
    let (bytes, content_type, file_name) = if let Some(cached) = cached {
        (cached.bytes, cached.content_type, cached.file_name)
    } else {
        let pool = auth_session.backend.pool;
        let file = match db::structs::Attachment::get(AttachmentIdentifier::Id(id.clone()), &pool).await {
            Ok(Some(f)) => f,
            Ok(None) => return (StatusCode::NOT_FOUND).into_response(),
            Err(e) => {
                tracing::error!(error = ?e);
                return (StatusCode::INTERNAL_SERVER_ERROR, "db error").into_response();
            }
        };

        let entry = file_cache::CachedFile {
            bytes: file.file_blob,
            content_type: file.mime_type,
            file_name: file.file_name,
        };
        let result = (entry.bytes.clone(), entry.content_type.clone(), entry.file_name.clone());
        file_cache::insert(id.clone(), entry).await;
        result
    };

    let disposition = format!(
        "inline; image; filename=\"{}\"",
        file_name
    );

    (
        [
            {if let Some(ref content_type) = content_type {
                (header::CONTENT_TYPE, content_type.clone())
            } else {
                (header::CONTENT_TYPE, "application/octet-stream".to_string())
            }},
            (header::CONTENT_DISPOSITION, disposition),
            (header::ETAG, format!("\"{}\"", id)),
            (header::CACHE_CONTROL, "private, max-age=3600".to_string()),
        ],
        bytes,
    ).into_response()
}

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use axum::response::sse;
        use axum::{extract::Request, middleware::Next, response::Response};
        use futures::stream::{Stream, StreamExt};
        use once_cell::sync::Lazy;
        use std::convert::Infallible;
        use std::sync::atomic::{AtomicU64, Ordering};
        use std::time::Instant;
        use tokio::sync::broadcast;
        use tokio_stream::wrappers::BroadcastStream;

        #[derive(Debug, PartialEq)]
        pub enum BroadcastScope {
            Events,
            Admin,
            Status
        }

        pub static EVENTS_TX: Lazy<broadcast::Sender<String>> = Lazy::new(|| {
            broadcast::channel::<String>(1024).0
        });

        pub static ADMIN_TX: Lazy<broadcast::Sender<String>> = Lazy::new(|| {
            broadcast::channel::<String>(1024).0
        });

        pub static STATUS_TX: Lazy<broadcast::Sender<String>> = Lazy::new(|| {
            broadcast::channel::<String>(1024).0
        });

        pub static TOTAL_BYTES: AtomicU64 = AtomicU64::new(0);

        pub static START_TIME: Lazy<Instant> = Lazy::new(Instant::now);

        pub async fn track_traffic(req: Request, next: Next) -> Response {
            let response = next.run(req).await;
            if let Some(content_length) = response.headers()
                .get(header::CONTENT_LENGTH)
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok())
            {
                TOTAL_BYTES.fetch_add(content_length, Ordering::Relaxed);
            }
            response
        }

        #[instrument]
        pub async fn events_sse() -> sse::Sse<impl Stream<Item = Result<sse::Event, Infallible>>> {
            let rx = EVENTS_TX.subscribe();

            let stream = BroadcastStream::new(rx)
                .filter_map(|res| async move { res.ok() })
                .map(|msg: String| sse::Event::default().data(msg))
                .map(Ok);

            sse::Sse::new(stream)
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

        pub async fn status_data_sse() -> sse::Sse<impl Stream<Item = Result<sse::Event, Infallible>>> {
            let rx = STATUS_TX.subscribe();

            let stream = BroadcastStream::new(rx)
                .filter_map(|res| async move { res.ok() })
                .map(|msg: String| sse::Event::default().data(msg))
                .map(Ok);

            sse::Sse::new(stream)
        }

        #[instrument]
        pub async fn build_and_broadcast(payload: ServerEventPayload, scopes: Vec<BroadcastScope>) -> Result<(), AppError> {
            for scope in scopes.into_iter() {
                let sender = match scope {
                    BroadcastScope::Admin => { &ADMIN_TX },
                    BroadcastScope::Events => { &EVENTS_TX }
                    BroadcastScope::Status => { &STATUS_TX }
                };

                match serde_json::to_string(&payload) {
                    Ok(json) => {
                        if let Err(e) = sender.send(json) {
                            tracing::debug!(error = ?e, "broadcast had no receivers");
                        }
                    }
                    Err(e) => {
                        tracing::error!(error = ?e, "serializing server event failed");
                        return Err(AppError::InternalError(e.to_string()));
                    }
                }
            }

            Ok(())
        }

        #[instrument]
        pub fn init_status_querying() {
            let status_pool = get_db();
            tokio::spawn(async move {
                use crate::utils::{format_duration, format_traffic};
                use std::sync::atomic::Ordering;
                use sysinfo::{Pid, System};

                let pid = Pid::from_u32(std::process::id());
                let mut sys = System::new();

                let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
                let mut tick_count = 0_u32;
                let mut cached_active_users = 0_u32;
                loop {
                    interval.tick().await;

                    if STATUS_TX.receiver_count() == 0 {
                        continue;
                    }

                    sys.refresh_processes(sysinfo::ProcessesToUpdate::Some(&[pid]), true);
                    sys.refresh_memory();

                    let (cpu_usage, ram_bytes) = sys.process(pid)
                        .map(|p| (p.cpu_usage(), p.memory()))
                        .unwrap_or((0.0, 0));
                    let ram_total_bytes = sys.total_memory();
                    let ram_usage = if ram_total_bytes > 0 {
                        (ram_bytes as f32 / ram_total_bytes as f32) * 100.0
                    } else {
                        0.0
                    };

                    if tick_count % 10 == 0 {
                        let count: i64 = sqlx::query_scalar(
                            "SELECT COUNT(*) FROM ctfpkhk.sessions WHERE expiry_date > NOW()"
                        ).fetch_one(&status_pool).await.unwrap_or(0);
                        cached_active_users = count as u32;
                    }
                    tick_count = tick_count.wrapping_add(1);

                    let active_users = cached_active_users;

                    let traffic_bytes = TOTAL_BYTES.load(Ordering::Relaxed);

                    let status = StatusData {
                        uptime: format_duration(START_TIME.elapsed().as_secs()),
                        active_users,
                        cpu_usage: cpu_usage.clamp(0.0, 100.0),
                        ram_usage: ram_usage.clamp(0.0, 100.0),
                        ram_usage_mb: ram_bytes as f32 / (1024.0 * 1024.0),
                        traffic: format_traffic(traffic_bytes),
                    };

                    if let Err(e) = build_and_broadcast(ServerEventPayload::StatusData(status), vec![BroadcastScope::Status]).await {
                        tracing::debug!(error = ?e, "status broadcast had no receivers");
                    }
                }
            });
        }
    }
}

/// Used at the start of every API function. Checks if the user is logged in.
#[cfg(feature = "ssr")]
#[instrument]
async fn authenticated_check() -> Result<(User, MySqlPool), AppError> {
    let auth = get_context::<AuthSession>()?;
    let response = get_context::<ResponseOptions>()?;
    match auth.user {
        Some(user) => Ok((user, auth.backend.pool)),
        None => {
            response.set_status(StatusCode::FORBIDDEN);
            return Err(AppError::Forbidden);
        }
    }
}

#[cfg(feature = "ssr")]
#[instrument]
pub async fn is_host_reachable(url: &String) -> Result<bool, AppError> {
    let url = url::Url::parse(url)?;
    let host = url.host_str().unwrap_or_default();
    let port = url.port().unwrap_or_default();
    let timeout = Duration::from_millis(1000);
    let addrs = tokio::net::lookup_host(format!("{host}:{port}")).await?;

    for addr in addrs {
        match tokio::time::timeout(timeout, TcpStream::connect(addr)).await {
            Ok(Ok(_)) => return Ok(true),
            _ => continue,
        }
    }

    Err(AppError::NetworkError("host unreachable".to_string()))
}

/// Used whenever we want the real-time row record of a `server::structs::User`.
/// We could attach a separate field like 'db_row: DbUser' to `server::structs::User`
/// to eliminate the use of this function, but let's just keep this for now.
#[cfg(feature = "ssr")]
#[instrument]
async fn get_db_user(user: &User, pool: &MySqlPool) -> Result<DbUser, AppError> {
    match DbUser::get(&UserIdentifier::Id(user.id.clone()), pool).await {
        Ok(Some(user)) => Ok(user),
        Ok(None) => {
            Err(AppError::InternalError("internal error".to_string()))
        }
        Err(e) => {
            tracing::error!(error = ?e);
            Err(AppError::InternalError("internal error".to_string()))
        }
    }
}

/// Used to fetch a full instance of an `ChallengeWithAttachments` from the DB whenever we only have
/// the ID of the `Event` and an active pool instance.
/// Using sqlx to serialize data from 2 other tables into one struct is kind of difficult, 
/// but this should do for now.
#[cfg(feature = "ssr")]
#[instrument]
pub async fn fetch_cwa(id: &str, pool: &MySqlPool) -> Result<ChallengeWithAttachments, AppError> {
    let challenge = db::structs::Challenge::get(&id.to_string(), pool).await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    let attachments_all = AttachmentWithoutBlob::get_all(
        &Some(db::enums::AttachmentIdentifier::ChallengeId(id.to_string())), pool
    ).await?;

    let mut attachments = vec![];
    let mut illustration = None;
    for att in attachments_all {
        if att.file_type == FileType::Illustration {
            illustration = Some(att);
        } else if att.file_type == FileType::Attachment {
            attachments.push(att);
        }
    }

    Ok(ChallengeWithAttachments { challenge, attachments, illustration })
}

/// Used to fetch a full instance of an `EventWithAttachments` from the DB whenever we only have
/// the ID of the `Event` and an active pool instance.
/// Using sqlx to serialize data from 2 other tables into one struct is kind of difficult, 
/// but this should do for now.
#[cfg(feature = "ssr")]
#[instrument]
pub async fn fetch_ewa(id: &str, pool: &MySqlPool) -> Result<EventWithAttachments, AppError> {
    let event = db::structs::Event::get(&id.to_string(), pool).await
        .map_err(|e| AppError::InternalError(e.to_string()))?;

    let attachments_all = AttachmentWithoutBlob::get_all(
        &Some(db::enums::AttachmentIdentifier::EventId(id.to_string())), pool
    ).await?;

    let mut attachments = vec![];
    let mut illustration = None;
    for att in attachments_all {
        if att.file_type == FileType::Illustration {
            illustration = Some(att);
        } else if att.file_type == FileType::Attachment {
            attachments.push(att);
        }
    }

    Ok(EventWithAttachments { event, attachments, illustration })
}
