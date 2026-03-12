#[cfg(feature = "ssr")]
use crate::server::{backend::{AuthSession, hash_string, verify_hash}, structs::AppState};
use crate::{error_template::AppError, server::{backend::enums::AuthType, db::{enums::{FileType, UserIdentifier, UserRole}, structs::{AttachmentWithoutBlob, Challenge, ChallengeWithAttachments, DbUser, DbUserWithoutPII, Event, HintWithoutHint, HintsUsed, LdapArgs, UserAvatar}}, enums::ResultStatus, proxmox::{ProxmoxVMInstance, ProxmoxVMTemplate}, structs::{ApiResult, Credentials, LeaderboardData, PivotRow, User}}, utils::{get_context, offset_to_datetime}};
#[cfg(feature = "ssr")]
use axum::{extract::Path, Router, routing::get};
#[cfg(feature = "ssr")]
use axum_login::AuthnBackend;
use cfg_if::cfg_if;
use chrono::{DateTime, Local};
use leptos::{prelude::{use_context}, server, server_fn::codec::{MultipartData, MultipartFormData}};
#[cfg(feature = "ssr")]
use leptos_axum::ResponseOptions;
use tracing::instrument;
use std::collections::{BTreeSet, HashMap};
#[cfg(feature = "ssr")]
use tokio::net::TcpStream;
use std::time::Duration;
#[cfg(feature = "ssr")]
use sqlx::MySqlPool;
#[cfg(feature = "ssr")]
use axum::{response::IntoResponse, http::{StatusCode, header}};
use crate::server::{enums::AdminEventPayloadKind, db::enums::AttachmentIdentifier};

pub mod admin;
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
        .route("/events", get(admin_sse))
        .route("/file/{id}", get(download_blob))
        .route("/avatar/{id}", get(serve_image))
        .route("/image/{id}", get(serve_image))
}

#[server(name=Challenges, prefix="/api", endpoint="challenges")]
#[instrument]
pub async fn get_all_challenges_with_attachments() -> Result<Vec<ChallengeWithAttachments>, AppError> {
    cfg_if! {
        if #[cfg(feature = "ssr")] {
            let (user, pool) = authenticated_check().await?;

            let db_user = get_db_user(&user, &pool).await?;

            let challenges = match db::structs::Challenge::get_all(&pool).await {
                Ok(challenges) => challenges,
                Err(e) => Err(e)?
            };

            let user_groups_vec = db_user.groups.split(',').map(|g| g.trim().to_string()).collect::<Vec<String>>();

            let all_attachments = AttachmentWithoutBlob::get_all(&None, &pool).await?;

            let mut attachments_by_challenge = HashMap::<String, Vec<AttachmentWithoutBlob>>::new();
            let mut illustration_by_challenge = HashMap::<String, AttachmentWithoutBlob>::new();

            for att in all_attachments {
                if let Some(event_id) = &att.challenge_id {
                    if att.file_type == FileType::Illustration {
                        illustration_by_challenge.insert(event_id.clone(), att);
                    } else if att.file_type == FileType::Attachment {
                        attachments_by_challenge.entry(event_id.clone()).or_default().push(att);
                    }
                }
            }

            let mut cwa: Vec<ChallengeWithAttachments> = Vec::new();
            for challenge in challenges {
                let visible_to_groups_vec = challenge.visible_to_groups.split(",").map(|v| v.to_string()).collect::<Vec<String>>();
                let is_visible = db_user.role == UserRole::Admin
                    || visible_to_groups_vec.iter().any(|g| g == "all" || user_groups_vec.contains(g));

                if is_visible {
                    let attachments = attachments_by_challenge.remove(&challenge.id).unwrap_or_default();
                    let illustration = illustration_by_challenge.remove(&challenge.id);
                    cwa.push(ChallengeWithAttachments { challenge, attachments, illustration });
                }
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
            let (user, pool) = authenticated_check().await?;

            let db_user = get_db_user(&user, &pool).await?;

            // should be active_event_ids in the future
            let active_event_id = match get_active_events().await {
                Ok(active_events) => {
                    let mut active_event_id = "".to_string();
                    for active_event in active_events {
                        let visible_to_groups_vec = active_event.visible_to_groups.split(",").map(|v| v.to_string()).collect::<Vec<String>>();
                        let user_groups_vec = db_user.groups.split(',').map(|g| g.trim().to_string()).collect::<Vec<String>>();
                        for group in visible_to_groups_vec {
                            if user_groups_vec.contains(&group) || db_user.role == UserRole::Admin || group == "all" {
                                active_event_id = active_event.id;
                                break;
                            }
                        }
                    }
                    active_event_id
                },
                Err(e) => {
                    tracing::error!(error = ?e);
                    return Err(AppError::InternalError("failed to fetch active events".to_string()));
                }
            };

            let meta = match db::structs::Event::get_metadata(&active_event_id, &pool).await {
                Ok(meta) => meta,
                Err(e) => {
                    tracing::error!(error = ?e);
                    return Err(AppError::InternalError("failed to fetch event metadata".to_string()));
                }
            };

            let event_name = meta.name.unwrap_or_default();
            if let Some(first_submission) = meta.first_submission && let Some(last_submission) = meta.last_submission {
                let x_min = offset_to_datetime(first_submission);
                let x_max = offset_to_datetime(last_submission);

                let y_max = match db::structs::Event::get_total_possible_points(&active_event_id, &pool).await {
                    Ok(y_max) => y_max,
                    Err(_) => return Err(AppError::InternalError("internal error".to_string()))
                };

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
                .fetch_all(&pool)
                .await?;

                let users: Vec<String> = solves.iter().map(|r| r.username.clone()).collect::<BTreeSet<_>>().into_iter().collect();

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

                let times = timestamps.into_iter().collect::<Vec<DateTime<Local>>>();

                let mut solves_by_ts = HashMap::<DateTime<Local>, Vec<&Solve>>::new();
                for s in &solves_parsed {
                    solves_by_ts.entry(s.ts).or_default().push(s);
                }

                let user_index = users.iter().enumerate().map(|(i, u)| (u.as_str(), i)).collect::<HashMap<&str, usize>>();
                let mut cumulative = vec![0.0_f64; users.len()];
                let mut rows = Vec::<PivotRow>::with_capacity(times.len());

                for ts in times {
                    if let Some(slist) = solves_by_ts.get(&ts) {
                        for s in slist {
                            if let Some(&idx) = user_index.get(s.username.as_str()) {
                                cumulative[idx] += s.points;
                            }
                        }
                    }

                    let values = users.iter().enumerate()
                        .map(|(i, u)| (u.clone(), cumulative[i]))
                        .collect::<HashMap<String, f64>>();
                    rows.push(PivotRow { ts, values });
                }

                Ok(LeaderboardData { event_name, x_min, x_max, y_max: y_max as f64, users, rows })
            } else {
                Ok(LeaderboardData::default())
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[server(name=LoginUser, prefix="/api", endpoint="login")]
#[instrument(skip(creds))]
pub async fn login_user(creds: Credentials) -> Result<ApiResult<Option<User>>, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let mut auth = get_context::<AuthSession>()?;
            let user: Option<User> = auth.backend.authenticate(creds.clone()).await?;

            if let Some(user) = user.as_ref() {
                match auth.login(user).await {
                    Ok(_) => {
                        let db_user = get_db_user(&user, &auth.backend.pool).await?;
                        let last_active_at = chrono::Local::now();
                        _ = DbUser::edit_last_active(&user.id, &last_active_at, &auth.backend.pool).await;
                        
                        // normal users have a user + pool created on register,
                        // ldap users already have a user, but no pool, so create one on login
                        if creds.auth_type == AuthType::Ldap {
                            _ = crate::server::proxmox::create_user_pool(&db_user).await;
                        }
                        
                        Ok(ApiResult { result: ResultStatus::Success, details: Some(user.clone()) })
                    },
                    Err(e) => {
                        tracing::error!(error = ?e);
                        Err(AppError::InternalError("internal error".to_string()))
                    }
                }
            } else {
                Err(AppError::BadRequest("invalid credentials".to_string()))
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
            let response = get_context::<ResponseOptions>()?;
            match use_context::<AuthSession>() {
                Some(session) => Ok(session.user),
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
            let (user, pool) = authenticated_check().await?;
            match db::structs::Submission::get_user_points(&user.id, &pool).await {
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

#[server(name=GetDbUserWithoutPII, prefix="/api/user", endpoint="info")]
#[instrument]
pub async fn get_db_user_without_pii(username: Option<String>) -> Result<Option<DbUserWithoutPII>, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let (user, pool) = authenticated_check().await?;
            if username.is_some() {
                match DbUserWithoutPII::get(&UserIdentifier::Username(username.unwrap_or_default()), &pool).await {
                    Ok(user) => Ok(user),
                    Err(e) => {
                        tracing::error!(error = ?e);
                        Err(AppError::InternalError("internal error".to_string()))
                    }
                }    
            } else {
                match DbUserWithoutPII::get(&UserIdentifier::Id(user.id), &pool).await {
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

#[server(name=Register, prefix="/api", endpoint="register")]
#[instrument(skip(password))]
pub async fn register_user(email: String, password: String, confirm_password: String) -> Result<ApiResult<Option<User>>, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let mut auth_session = get_context::<AuthSession>()?;

            if password != confirm_password {
                return Err(AppError::BadRequest("password and confirm password must match".to_string()));
            }

            let user: Option<User> = auth_session.backend.add_user(&email, &password).await?;

            if let Some(user) = user {
                auth_session.login(&user).await?;
                let db_user = get_db_user(&user, &auth_session.backend.pool).await?;
                _ = crate::server::proxmox::create_user(&email, &db_user.username, &password).await;
                _ = crate::server::proxmox::create_user_pool(&db_user).await;
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
            let (user, pool) = authenticated_check().await?;
            let db_user = get_db_user(&user, &pool).await?;

            let mut tx = pool.begin().await?;

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

            if let Ok(()) = verify_hash(&flag, &challenge_flag_hash).await {
                match db::structs::Submission::add(&challenge.id, &challenge.event_id, &user.id, &challenge.points, &chrono::Local::now(), &mut *tx).await {
                    Ok(_) => {
                        tx.commit().await?;

                        if let Some(vm_ids_string) = challenge.vm_ids {
                            let template_ids = vm_ids_string.split(",").map(|c| c.parse::<u32>().unwrap_or_default()).collect::<Vec<u32>>();
                            for template_id in template_ids {
                                _ = crate::server::proxmox::destroy_vm(&db_user, &template_id).await;
                            }
                        };

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
            let (user, pool) = authenticated_check().await?;
            let db_user = get_db_user(&user, &pool).await?;

            if username == db_user.username {
                return Ok(ApiResult { result: ResultStatus::Fail, details: "username already exists".to_string() });
            } else if username.is_empty() || !username.is_ascii() {
                return Err(AppError::InternalError("invalid username".to_string()));
            } else {
                match DbUser::edit_username(&user.id, &username, &pool).await {
                    Ok(_) => Ok(ApiResult { result: ResultStatus::Success, details: "changed username".to_string() }),
                    Err(e) => Err(e.into())
                }
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
            let (user, pool) = authenticated_check().await?;
            
            let mut data = match avatar.into_inner() {
                Some(inner_data) => inner_data,
                None => {
                    return Err(AppError::InternalError("Failed to extract inner data from avatar".to_string()));
                }
            };
            let mut file_name = String::new();
            let mut file_blob = Vec::<u8>::new();
            let mut mime_type = String::new();
            while let Ok(Some(mut field)) = data.next_field().await {
                if let Some(field_file_name) = field.file_name() {
                    file_name = field_file_name.to_string();
                } else {
                    return Err(AppError::InternalError("Failed to extract file name".to_string()))
                }

                if let Some(field_content_type) = field.content_type() {
                    mime_type = field_content_type.to_string();
                } else {
                    return Err(AppError::InternalError("Failed to extract content type".to_string()))
                }

                while let Ok(Some(chunk)) = field.chunk().await {
                    file_blob.extend_from_slice(&chunk);
                }
            }

            let mut tx = pool.begin().await?;

            if let Err(e) = DbUser::delete_avatar(&user.id, &mut *tx).await {
                tx.rollback().await?;
                return Err(e.into());
            }

            match DbUser::edit_avatar(&user.id, &file_name, &file_blob, &mime_type, &mut *tx).await {
                Ok(_) => {
                    tx.commit().await?;
                    Ok(ApiResult { result: ResultStatus::Success, details: "changed avatar".to_string() })
                },
                Err(e) => {
                    tx.rollback().await?;
                    Err(e.into())
                }
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[server(name=GetAvatarId, prefix="/api/user", endpoint="avatar_id")]
#[instrument]
pub async fn get_avatar_id(identifier: UserIdentifier) -> Result<Option<String>, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let (_, pool) = authenticated_check().await?;

            match DbUser::get_avatar_id(&identifier, &pool).await {
                Ok(id) => Ok(id),
                Err(e) => Err(e.into())
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[server(name=GetAllUserAvatarIds, prefix="/api/users", endpoint="avatar_ids")]
#[instrument]
pub async fn get_all_user_avatar_ids() -> Result<Vec<UserAvatar>, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let (_, pool) = authenticated_check().await?;

            match DbUser::get_all_avatar_ids(&pool).await {
                Ok(ids) => Ok(ids),
                Err(e) => Err(e.into())
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[server(name=GetAttachmentId, prefix="/api", endpoint="attachment_id")]
#[instrument]
pub async fn get_attachment_id(identifier: AttachmentIdentifier) -> Result<Option<String>, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let (_, pool) = authenticated_check().await?;

            match AttachmentWithoutBlob::get_id(&identifier, &pool).await {
                Ok(id) => Ok(id),
                Err(e) => Err(e.into())
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[server(name=GetAllIllustrations, prefix="/api", endpoint="get_all_illustrations")]
#[instrument]
pub async fn get_all_illustrations() -> Result<Vec<AttachmentWithoutBlob>, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let (_, pool) = authenticated_check().await?;

            match AttachmentWithoutBlob::get_all_illustrations(&pool).await {
                Ok(illustrations) => Ok(illustrations),
                Err(e) => Err(e.into())
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[server(name=GetIllustrationId, prefix="/api", endpoint="illustration_id")]
#[instrument]
pub async fn get_illustration_id(identifier: AttachmentIdentifier) -> Result<Option<String>, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let (_, pool) = authenticated_check().await?;

            match AttachmentWithoutBlob::get_illustration_id(&identifier, &pool).await {
                Ok(id) => Ok(id),
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
            let (user, pool) = authenticated_check().await?;

            match db::structs::Submission::get_user_solved_challenges(&user.id, &pool).await {
                Ok(solved) => Ok(solved),
                Err(e) => Err(e.into())
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
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

#[server(name=GetActiveEvents, prefix="/api", endpoint="active_events")]
#[instrument]
pub async fn get_active_events() -> Result<Vec<Event>, AppError> {
    cfg_if! {
        if #[cfg(feature = "ssr")] {
            let auth = get_context::<AuthSession>()?;
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
pub async fn edit_password(old_password: String, new_password: String, confirm_new_password: String) -> Result<ApiResult<String>, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let (user, pool) = authenticated_check().await?;

            if new_password != confirm_new_password {
                return Err(AppError::BadRequest("new password and confirm new password must match".to_string()));
            }

            if old_password == new_password {
                return Ok(ApiResult { result: ResultStatus::Fail, details: "new password is same as old password".to_string() });
            }

            let pw_hash = hash_string(&new_password).await?;
            match DbUser::edit_password(&user.id, &pw_hash, &pool).await {
                Ok(_) => Ok(ApiResult { result: ResultStatus::Success, details: "changed password".to_string() }),
                Err(e) => return Err(e.into())
            }

            // match crate::server::proxmox::change_user_password(db_user, new_password).await {
            //     Ok(_) => Ok(ApiResult { result: ResultStatus::Success, details: "changed password".to_string() }),
            //     Err(e) => Err(e.into())
            // }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[server(name=UserExists, prefix="/api",endpoint="user_exists")]
pub async fn user_exists(email: String) -> Result<bool, AppError> {
    let auth = get_context::<AuthSession>()?;

    match DbUser::is_user_available(&email, &auth.backend.pool).await {
        Ok(result) => Ok(result),
        Err(e) => Err(e.into())
    }
}

#[server(name=LogoutUser, prefix="/api",endpoint="logout")]
pub async fn logout_user() -> Result<(), AppError> {
    let mut auth = get_context::<AuthSession>()?;

    match auth.logout().await {
        Ok(_) => {
            leptos_axum::redirect("/");
            Ok(())
        }
        Err(e) => Err(e.into())
    }
}

#[server(name=IsLdapEnabled, prefix="/api", endpoint="ldap_enabled")]
#[instrument]
pub async fn is_ldap_enabled() -> Result<bool, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let auth = get_context::<AuthSession>()?;
            
            match LdapArgs::get_status(&auth.backend.pool).await {
                Ok(enabled) => Ok(enabled),
                Err(e) => Err(e.into())
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[server(name=StartVM, prefix="/api", endpoint="start_vm")]
#[instrument]
pub async fn start_vm(template_id: u32, challenge: Challenge) -> Result<ApiResult<String>, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let (user, pool) = authenticated_check().await?;
            let db_user = get_db_user(&user, &pool).await?;

            match crate::server::proxmox::start_vm(&template_id, &challenge, &db_user).await {
                Ok(vm_id) => Ok(ApiResult { result: ResultStatus::Success, details: format!("Successfully started VM (ID: {vm_id})") }),
                Err(e) => return Err(e.into())
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[server(name=RestartVM, prefix="/api", endpoint="restart_vm")]
#[instrument]
pub async fn restart_vm(template_id: u32) -> Result<ApiResult<String>, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let (user, pool) = authenticated_check().await?;
            let db_user = get_db_user(&user, &pool).await?;

            match crate::server::proxmox::restart_vm(&db_user, &template_id).await {
                Ok(vm_id) => Ok(ApiResult { result: ResultStatus::Success, details: format!("Successfully restarted VM (ID: {vm_id})") }),
                Err(e) => Err(e)
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[server(name=DestroyVM, prefix="/api", endpoint="destroy_vm")]
#[instrument]
pub async fn destroy_vm(template_id: u32) -> Result<ApiResult<String>, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let (user, pool) = authenticated_check().await?;
            let db_user = get_db_user(&user, &pool).await?;

            match crate::server::proxmox::destroy_vm(&db_user, &template_id).await {
                Ok(vm_id) => Ok(ApiResult { result: ResultStatus::Success, details: format!("Successfully destroyed VM (ID: {vm_id})") }),
                Err(e) => return Err(e)
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[server(name=AddVMTime, prefix="/api", endpoint="add_vm_time")]
#[instrument]
pub async fn add_vm_time(template_id: u32) -> Result<ApiResult<String>, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let (user, pool) = authenticated_check().await?;
            let db_user = get_db_user(&user, &pool).await?;
            
            match crate::server::proxmox::add_vm_time(&db_user, &template_id).await {
                Ok(vm_id) => Ok(ApiResult { result: ResultStatus::Success, details: format!("Successfully added time to VM (ID: {vm_id})") }),
                Err(e) => Err(e.into())
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[server(name=GetUserActiveVMs, prefix="/api", endpoint="get_active_vms")]
#[instrument]
pub async fn get_user_active_vms() -> Result<Vec<ProxmoxVMInstance>, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let (user, pool) = authenticated_check().await?;
            let db_user = get_db_user(&user, &pool).await?;

            let vms = match crate::server::proxmox::get_user_vms(&db_user).await {
                Ok(vms) => vms,
                Err(e) => return Err(e)
            };

            let mut active_vms = Vec::<ProxmoxVMInstance>::new();
            for vm in vms {
                if vm.running { active_vms.push(vm) }
            }
            Ok(active_vms)
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[server(name=GetUserVMs, prefix="/api", endpoint="get_vms")]
#[instrument]
pub async fn get_user_vms() -> Result<Vec<ProxmoxVMInstance>, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let (user, pool) = authenticated_check().await?;
            let db_user = get_db_user(&user, &pool).await?;

            match crate::server::proxmox::get_user_vms(&db_user).await {
                Ok(vms) => Ok(vms),
                Err(e) => return Err(e)
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[server(name=GetUsedHints, prefix="/api/user", endpoint="hints_used")]
#[instrument]
pub async fn get_used_hints() -> Result<Vec<HintsUsed>, AppError> {
    cfg_if! {
        if #[cfg(feature = "ssr")] {
            let (user, pool) = authenticated_check().await?;
            let db_user = get_db_user(&user, &pool).await?;

            match HintsUsed::get(&db_user, &pool).await {
                Ok(hints_used) => Ok(hints_used),
                Err(e) => Err(e.into())
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[server(name=GetHint, prefix="/api/challenge", endpoint="get_hint")]
#[instrument]
pub async fn get_hint(challenge_id: String, hint_id: String) -> Result<crate::server::db::structs::Hint, AppError> {
    cfg_if! {
        if #[cfg(feature = "ssr")] {
            let (user, pool) = authenticated_check().await?;
            let db_user = get_db_user(&user, &pool).await?;

            let used_hints = HintsUsed::get(&db_user, &pool).await?;
            let used_hint_ids = used_hints.into_iter().map(|h| h.hint_id).collect::<Vec<String>>();
            if used_hint_ids.contains(&hint_id) {
                match crate::server::db::structs::Hint::get(&hint_id, &pool).await {
                    Ok(hint) => Ok(hint),
                    Err(e) => {
                        return Err(e.into());
                    }
                }
            } else {
                let mut tx = pool.begin().await?;
                if let Err(e) = HintsUsed::add(&challenge_id, &user.id, &hint_id, &mut *tx).await {
                    tx.rollback().await?;
                    return Err(e.into());
                }

                let hint = match crate::server::db::structs::Hint::get(&hint_id, &mut *tx).await {
                    Ok(hint) => hint,
                    Err(e) => {
                        tx.rollback().await?;
                        return Err(e.into());
                    }
                };
                
                match db_user.deduct_points(&hint.points_penalty, &mut *tx).await {
                    Ok(_) => tx.commit().await?,
                    Err(e) => {
                        tx.rollback().await?;
                        return Err(e.into());
                    }
                }

                Ok(hint)
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[server(name=GetAllHintsWithoutHints, prefix="/api/challenges", endpoint="get_hints")]
#[instrument]
pub async fn get_all_hints_without_hints() -> Result<Vec<HintWithoutHint>, AppError> {
    cfg_if! {
        if #[cfg(feature = "ssr")] {
            let (_, pool) = authenticated_check().await?;

            match HintWithoutHint::get_all_hints(&pool).await {
                Ok(hints) => Ok(hints),
                Err(e) => Err(e.into())
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[server(name=GetChallengeHintsWithoutHints, prefix="/api/challenge", endpoint="get_hints")]
#[instrument]
pub async fn get_challenge_hints_without_hints(challenge_id: String) -> Result<Vec<HintWithoutHint>, AppError> {
    cfg_if! {
        if #[cfg(feature = "ssr")] {
            let (_, pool) = authenticated_check().await?;

            match HintWithoutHint::get_challenge_hints(&challenge_id, &pool).await {
                Ok(hints) => Ok(hints),
                Err(e) => Err(e.into())
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[server(name=GetProxmoxBaseUrl, prefix="/api", endpoint="get_proxmox_url")]
#[instrument]
pub async fn get_proxmox_base_url() -> Result<String, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let (_, _) = authenticated_check().await?;

            let base_url = crate::server::proxmox::get_proxmox_base_url().await?;
            Ok(base_url)
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[server(name=GetTemplateInfo, prefix="/api", endpoint="get_template_info")]
#[instrument]
pub async fn get_all_templates() -> Result<Vec<ProxmoxVMTemplate>, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let (_, _) = authenticated_check().await?;

            match crate::server::proxmox::get_all_templates().await {
                Ok(templates) => Ok(templates),
                Err(e) => return Err(e)
            }
        } else {
            Err(AppError::NoServerConnection)
        }
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
            // let payload = AdminEventPayload {
            //     kind: payload_kind,
            // };
            match serde_json::to_string(&payload_kind) {
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
