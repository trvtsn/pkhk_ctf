#[cfg(feature = "ssr")]
use crate::server::{backend::{AuthSession, structs::{Credentials}}};
use crate::{error_template::AppError, server::{db::{enums::{UserIdentifier, UserRole}, structs::{Challenge, ChallengeWithAttachments, DbUser, Event, Submission, SubmissionWithData}}, enums::ResultStatus, structs::{ApiResult, LeaderboardData, PivotRow, User}}};
#[cfg(feature = "ssr")]
use argon2::{Argon2, PasswordVerifier};
// #[cfg(feature = "ssr")]
// use password_hash::PasswordHash;
#[cfg(feature = "ssr")]
use axum::extract::Path;
#[cfg(feature = "ssr")]
use axum_login::AuthnBackend;
use cfg_if::cfg_if;
use chrono::{DateTime, NaiveDateTime, Utc};
use leptos::{
    logging::log, prelude::{
        ServerFnError, ServerFnErrorErr, use_context
    }, server
};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use std::collections::{BTreeSet, HashMap};
#[cfg(feature = "ssr")]
use sqlx::MySqlPool;
#[cfg(feature = "ssr")]
use axum::{response::IntoResponse, http::{StatusCode, header}};
use crate::server::db::enums::AttachmentIdentifier;

pub mod admin;
#[cfg(feature = "ssr")]
pub mod auth;
pub mod backend;
pub mod db;
pub mod structs {
    use crate::server::{UserRole, enums::ResultStatus};
    use chrono::{DateTime, Utc};
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
        pub id: u32,

        /// User-facing username, has a unique constraint in the db so we can use it to id users
        pub username: String,

        /// This is computed with Argon2id, but it's only a *piece* of the entire thing returned
        /// by the hash function. You should be able to use whatever you want here as long as you
        /// can keep it stable between page loads. Personally, I don't like using the password hash
        /// but that's how they do it in the example so it's probably fine.
        pub session_auth_hash: Vec<u8>,

        pub role: UserRole
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub struct ApiResult<T> {
        pub result: ResultStatus,
        pub details: T
    }

    #[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
    pub struct PivotRow {
        pub ts: DateTime<Utc>,
        pub values: HashMap<String, f64>,
    }

    #[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
    pub struct LeaderboardData {
        pub event_name: String,
        pub x_min: DateTime<Utc>,
        pub x_max: DateTime<Utc>,
        pub y_max: f64,
        pub users: Vec<String>,
        pub rows: Vec<PivotRow>,
    }
}
pub mod enums {
    use serde::{Deserialize, Serialize};
    
    #[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
    #[serde(rename_all = "lowercase")]
    pub enum ResultStatus {
        Success,
        Fail
    }
}
//     use serde::{Deserialize, Serialize};

//     pub enum ApiActions {
//         Challenge {
//             action: ChallengeAction
//         },
//         Leaderboard {
//             action: LeaderboardAction
//         },
//         User {
//             action: UserAction
//         }
//     }

//     // #[serde(rename_all = "lowercase")]
//     #[derive(Debug, Clone, Deserialize, Serialize)]
//     pub enum LeaderboardAction {
//         Build
//     }

//     // #[serde(rename_all = "lowercase")]
//     #[derive(Debug, Clone, Deserialize, Serialize)]
//     pub enum UserAction {
//         Build,
//         IsAdmin,
//         Get,
//         Login,
//         Register,
//         GetAll,
//         GetPoints
//     }
// }

#[cfg(feature = "ssr")]
pub fn pool() -> Result<MySqlPool, AppError> {
    use_context::<MySqlPool>().ok_or_else(|| {
        AppError::DatabaseError("Pool missing.".to_string())
    })
}

#[cfg(feature = "ssr")]
pub fn init_env() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    Ok(())
}

#[server(name=Challenges, prefix="/api", endpoint="challenges")]
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
                let attachments = db::structs::Attachment::get_filenames(&AttachmentIdentifier::ChallengeId(challenge.id), &auth.backend.pool).await?;
                cwa.push(ChallengeWithAttachments { challenge, attachments });
            }
            Ok(cwa)
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

// #[server(name=Challenges, prefix="/api", endpoint="challenges")]
// pub async fn get_all_challenges() -> Result<Vec<Challenge>, AppError> {
//     cfg_if! {
//         if #[cfg(feature = "ssr")] {
//             let auth = use_context::<AuthSession>().unwrap();
//             match db::structs::Challenge::get_all(&auth.backend.pool).await {
//                 Ok(challenges) => Ok(challenges),
//                 Err(e) => Err(e)?
//             }
//         } else {
//             Err(AppError::NoServerConnection)
//         }
//     }
// }

#[server(name=Leaderboard, prefix="/api", endpoint="leaderboard")]
pub async fn build_leaderboard_data() -> Result<LeaderboardData, AppError> {
    cfg_if! {
        if #[cfg(feature = "ssr")] {
            let event_id: &u32 = &1;
            let pool = &pool()?;

            let meta = match db::structs::Event::get_metadata(event_id, pool).await {
                Ok(meta) => meta,
                Err(e) => Err(e)?
            };

            let event_name = meta.name.unwrap_or("".to_string());
            let x_min = DateTime::from_timestamp(meta.first_submission.unwrap().unix_timestamp(), meta.first_submission.unwrap().nanosecond()).unwrap();
            let x_max = DateTime::from_timestamp(meta.last_submission.unwrap().unix_timestamp(), meta.last_submission.unwrap().nanosecond()).unwrap();

            let y_max = db::structs::Event::get_total_possible_points(event_id, pool).await.unwrap();

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
                event_id
            )
            .fetch_all(pool)
            .await?;

            let users: Vec<String> = solves.iter().map(|r| r.username.clone()).collect();

            let mut timestamps = BTreeSet::new();

            #[derive(Debug)]
            struct Solve { username: String, ts: DateTime<Utc>, points: f64 }

            let mut solves_parsed: Vec<Solve> = Vec::new();
            for r in solves {
                let unix_timestamp = match r.solved_at {
                    Some(timestamp) => timestamp.unix_timestamp(),
                    None => 0 as i64
                };

                let nanoseconds = match r.solved_at {
                    Some(timestamp) => timestamp.nanosecond(),
                    None => 0 as u32
                };

                let ts = match DateTime::from_timestamp(unix_timestamp, nanoseconds) {
                    Some(ts) => ts,
                    None => chrono::Local::now().to_utc()
                };
                timestamps.insert(ts);
                solves_parsed.push(Solve {
                    username: r.username,
                    ts,
                    points: r.points as f64,
                });
            }

            let mut times: Vec<DateTime<Utc>> = timestamps.into_iter().collect();
            times.sort();

            let mut user_cumulative: HashMap<String, f64> = users.iter().map(|u| (u.clone(), 0.0)).collect();
            let mut solves_by_ts: HashMap<DateTime<Utc>, Vec<&Solve>> = HashMap::new();
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

#[server]
pub async fn is_user_admin() -> Result<bool, AppError> {
    cfg_if! {
        if #[cfg(feature = "ssr")] {
            let session = use_context::<AuthSession>().unwrap();
            if session.user.unwrap().role == UserRole::Admin {
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[server(name=LoginUser, prefix="/api", endpoint="login")]
pub async fn login_user(email: String, password: String) -> Result<ApiResult<Option<User>>, AppError> {
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
                    Ok(_) => Ok(ApiResult { result: ResultStatus::Fail, details: Some(user.clone()) }),
                    Err(e) => Err(AppError::InternalError(e.to_string()))
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
pub async fn get_user() -> Result<Option<User>, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            match use_context::<AuthSession>() {
                Some(session) => Ok(session.user.clone()),
                None => Ok(None)
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[server(name=GetUserPoints, prefix="/api/user", endpoint="points")]
pub async fn get_user_points() -> Result<u32, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let session: AuthSession = use_context().expect("session not provided");
            match db::structs::Submission::get_user_points(&session.user.unwrap_or_default().id, &session.backend.pool).await {
                Ok(points) => Ok(points),
                Err(e) => Err(AppError::InternalError(e.to_string()))
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[server(name=GetDbUser, prefix="/api/user", endpoint="info")]
pub async fn get_db_user(username: String) -> Result<Option<DbUser>, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let pool = use_context::<MySqlPool>().expect("pool not provided");
            match DbUser::get(&UserIdentifier::Username(username), &pool).await {
                Ok(user) => Ok(user),
                Err(e) => Err(AppError::InternalError(e.to_string()))
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
pub async fn register_user(email: String, password: String) -> Result<ApiResult<Option<User>>, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            // Extract the auth_session and session. You could also use `leptos_axum::extract().await` here,
            // but this seems nicer.
            let mut auth_session: AuthSession = use_context().expect("auth-session not provided");

            // The backend handles all of the password hashing and whatnot. Just call add_user and then go write
            // the backend, and it's all done!
            let user: Option<User> = auth_session.backend.add_user(email.clone(), password).await?;

            log!("get_user returned {user:#?}");
            if let Some(user) = user {
                // Tell the AuthSession that we're logged-in now and it should behave accordingly. This will set the
                // session id and send it to the browser as a side-effect (before now you likely had no session id in the browser).
                auth_session.login(&user).await?;
                log!("AuthSession user after register: {}", auth_session.user.as_ref().unwrap().username);
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
pub async fn check_flag(flag: String, challenge: crate::server::db::structs::Challenge) -> Result<ApiResult<String>, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            // Note that you can still use `leptos_axum::extract().await?` if you want, but since we
            // called `provide_context` from the `server_fn_handler` in `main`, we can do it this way
            // and it feels faster. Get the AuthSession.
            let auth = use_context::<AuthSession>().unwrap();
            let user = auth.user.unwrap_or_default();

            // check if the challenge is already solved, and if so, return Error
            match db::structs::Submission::get_user_solved_challenges(&user.id, &auth.backend.pool).await {
                Ok(solved) => {
                    if solved.contains(&challenge.id) {
                        return Ok(ApiResult { result: ResultStatus::Fail, details: "challenge already solved".to_string() });
                    }
                },
                Err(e) => return Err(AppError::InternalError("failed to check flag".to_string()))
            }

            let challenge_flag_hash = match db::structs::Challenge::get_flag_hash(&challenge.id, &auth.backend.pool).await {
                Ok(flag_hash) => flag_hash,
                Err(e) => return Err(AppError::InternalError("Failed to get flag hash".to_string())),
            };

            let hasher = Argon2::default();
            let hash = argon2::PasswordHash::parse(challenge_flag_hash.as_ref(), argon2::password_hash::Encoding::B64)
                .map_err(|e| AppError::InternalError("Failed to parse flag hash".to_string()))?;
            // Use the existing implementation to verify the password. I was doing this myself until
            // I noticed that there is a PasswordVerifier trait, so this is better in every way.
            if let Ok(()) = hasher.verify_password(flag.as_bytes(), &hash) {
                match db::structs::Submission::add(&challenge.id, &challenge.event_id, &user.id, &challenge.points, &OffsetDateTime::now_utc(), &auth.backend.pool).await {
                    Ok(_) => Ok(ApiResult { result: ResultStatus::Success, details: "correct solution".to_string() }),
                    Err(e) => Err(e.into())
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
pub async fn edit_username(username: String) -> Result<ApiResult<String>, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let auth = use_context::<AuthSession>().unwrap();
            let user = auth.user.unwrap_or_default();
            match DbUser::edit_username(&user.id, &username, &auth.backend.pool).await {
                Ok(_) => Ok(ApiResult { result: ResultStatus::Success, details: "changed username".to_string() }),
                Err(e) => Err(e.into())
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[server(name=EditAvatar, prefix="/api/user", endpoint="avatar")]
pub async fn edit_avatar(avatar: Vec<u8>) -> Result<ApiResult<String>, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let auth = use_context::<AuthSession>().unwrap();
            let user = auth.user.unwrap_or_default();
            match DbUser::edit_avatar(&user.id, &avatar, &auth.backend.pool).await {
                Ok(_) => Ok(ApiResult { result: ResultStatus::Success, details: "changed avatar".to_string() }),
                Err(e) => Err(e.into())
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[server(name=SolvedChallenges, prefix="/api/challenges", endpoint="solved")]
pub async fn get_user_solved_challenges() -> Result<Vec<u32>, AppError> {
    cfg_if! {
        if #[cfg(feature = "ssr")] {
            let auth = use_context::<AuthSession>().unwrap();
            let user = auth.user.unwrap_or_default();
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
pub async fn download_blob(
    auth_session: AuthSession,
    Path(filename): Path<String>,
) -> impl IntoResponse {
    let pool = auth_session.backend.pool;
    let file = match db::structs::Attachment::get(AttachmentIdentifier::FileName(filename.clone()), &pool).await {
        Ok(Some(f)) => f,
        Ok(None) => return (StatusCode::NOT_FOUND).into_response(),
        Err(e) => {
            tracing::error!("db error: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "db error").into_response();
        }
    };

    let bytes = file.file_blob;
    let disposition = format!(
        "attachment; filename=\"{}\"",
        // sanitize(&filename)
        filename
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
pub async fn get_active_events() -> Result<Vec<Event>, AppError> {
    cfg_if! {
        if #[cfg(feature = "ssr")] {
            let auth = use_context::<AuthSession>().unwrap();
            let events = match db::structs::Event::get_all(&auth.backend.pool).await {
                Ok(events) => events,
                Err(e) => return Err(e.into())
            };

            let mut active_events = Vec::new();
            let now = OffsetDateTime::now_utc();
            for event in events.into_iter() {
                if now >= event.start_date && now <= event.end_date {
                    active_events.push(event);
                } 
            }

            Ok(active_events)
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}
