use async_trait::async_trait;
use cfg_if::cfg_if;
use chrono::NaiveDateTime;
use leptos::{prelude::*, server_fn::error::NoCustomError};
use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use crate::server::AuthSession;
use crate::server::{db::{self, structs::DbUser}, UserRole, structs::User};

// pub enum UserArgs {
//     Create {
//         user: DbUser
//     },
// }

// #[async_trait]
// pub trait Crud {
//     type Arguments: Send + Sync;
//     /// An error which can occur during authentication and authorization.
//     type Error: std::error::Error + Send + Sync;

//     async fn create(args: Self::Arguments, session: AuthSession) -> Result<(), Self::Error>;
//     async fn read(args: Self::Arguments, session: AuthSession) -> Result<Self, Self::Error>;
//     async fn update(args: Self::Arguments, session: AuthSession) -> Result<(), Self::Error>;
//     async fn delete(args: Self::Arguments, session: AuthSession) -> Result<(), Self::Error>;
// }

// impl Crud for User {
//     type Arguments = UserArgs;
//     type Error = Error;

//     async fn create(args: Self::Arguments, session: AuthSession) -> Result<(), Self::Error> {
//         Ok(())
//     }
// }

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ApiResult<T> {
    Success {
        result: T,
        message: Option<String>
    },
    Fail {
        message: Option<String>
    }
}

// #[serde(rename_all = "lowercase")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ChallengeAction {
    Create {
        event_id: u32, 
        name: String, 
        description: String, 
        category: String,
        difficulty: i8, 
        points: u32, 
        flag: String
    },
    Delete {
        id: u32
    },
    Edit {
        id: u32,
        event_id: u32, 
        name: String, 
        description: String, 
        category: String,
        difficulty: i8, 
        points: u32, 
        flag: String
    },
    Check {
        id: u32,
        flag: String
    }
}

#[server(name=AdminChallengeApi, prefix="/api/admin", endpoint="challenge")]
pub async fn challenge(action: ChallengeAction) -> Result<ApiResult<()>, ServerFnError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            // Note that you can still use `leptos_axum::extract().await?` if you want, but since we
            // called `provide_context` from the `server_fn_handler` in `main`, we can do it this way
            // and it feels faster. Get the AuthSession.
            let auth = use_context::<AuthSession>().unwrap();
            let user = auth.user.unwrap_or_default();
            if user.role != UserRole::Admin {
                return Ok(ApiResult::Fail { message: Some("unauthorized".to_string()) });
            }
            // hash flag

            match action {
                ChallengeAction::Create { event_id, name, description, category, difficulty, points, flag } => {
                    _ = db::structs::Challenge::add(&event_id, &name, &description, &category, &difficulty, &points, &flag, &auth.backend.pool).await;
                    Ok(ApiResult::Success { result: (), message: Some("created event".to_string()) })
                }
                ChallengeAction::Delete { id } => {
                    _ = db::structs::Challenge::delete(&id, &auth.backend.pool).await;
                    Ok(ApiResult::Success { result: (), message: Some("deleted challenge".to_string()) })
                }
                ChallengeAction::Edit { id, event_id, name, description, category, difficulty, points, flag } => {
                    _ = db::structs::Challenge::edit(&id, &event_id, &name, &description, &category, &difficulty, &points, &flag, &auth.backend.pool).await;
                    Ok(ApiResult::Success { result: (), message: Some("edited challenge".to_string()) })
                }
                ChallengeAction::Check { id, flag } => {
                    let challenge_flag_hash = match db::structs::Challenge::get_flag_hash(&id, &auth.backend.pool).await {
                        Ok(flag_hash) => flag_hash,
                        Err(e) => "".to_string()
                    };
                    Ok(ApiResult::Success { result: (), message: Some("correct flag".to_string()) })

                    // let hasher = Argon2::default();
                    // let hash = PasswordHash::parse(flag.as_ref(), password_hash::Encoding::B64)
                    //     .map_err(|e| Self::Error::InternalError(format!("Corrupted password hash: {e}")))?;
                    // // Use the existing implementation to verify the password. I was doing this myself until
                    // // I noticed that there is a PasswordVerifier trait, so this is better in every way.
                    // if let Ok(()) = hasher.verify_password(challenge_flag_hash.as_bytes(), &hash) {

                    // } else {

                    // }
                }
            }
        } else {
            Ok(())
        }
    }
}

// #[serde(rename_all = "lowercase")]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum EventAction {
    Create {
        name: String,  
        description: String, 
        start_date: NaiveDateTime, 
        end_date: NaiveDateTime
    },
    Delete {
        id: u32
    },
    Edit {
        id: u32,
        name: String,  
        description: String, 
        start_date: NaiveDateTime, 
        end_date: NaiveDateTime
    }
}

#[server(name=AdminEventApi, prefix="/api/admin", endpoint="event")]
pub async fn event(action: EventAction) -> Result<ApiResult<()>, ServerFnError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            // Note that you can still use `leptos_axum::extract().await?` if you want, but since we
            // called `provide_context` from the `server_fn_handler` in `main`, we can do it this way
            // and it feels faster. Get the AuthSession.
            let auth = use_context::<AuthSession>().unwrap();
            let user = auth.user.unwrap_or_default();
            if user.role != UserRole::Admin {
                return Ok(ApiResult::Fail { message: Some("unauthorized".to_string()) });
            }

            match action {
                EventAction::Create { name, description, start_date, end_date } => {
                    _ = db::structs::Event::add(&name, &description, &start_date, &end_date, &auth.backend.pool).await;
                    Ok(ApiResult::Success { result: (), message: Some("created event".to_string()) })
                }
                EventAction::Delete { id } => {
                    _ = db::structs::Event::delete(&id, &auth.backend.pool).await;
                    Ok(ApiResult::Success { result: (), message: Some("deleted event".to_string()) })
                }
                EventAction::Edit { id, name, description, start_date, end_date } => {
                    _ = db::structs::Event::edit(&id, &name, &description, &start_date, &end_date, &auth.backend.pool).await;
                    Ok(ApiResult::Success { result: (), message: Some("edited event".to_string()) })
                }
            }
        } else {
            Ok(ApiResult::Fail { message: None })
        }
    }
}

#[server(name=AdminUsersGetAll, prefix="/api/admin", endpoint="users")]
pub async fn get_all_users() -> Result<Vec<DbUser>, ServerFnError> {
    cfg_if! {
        if #[cfg(feature = "ssr")] {
            let auth = use_context::<AuthSession>().unwrap();
            let user = auth.user.unwrap_or_default();

            if user.role != UserRole::Admin {
                return Err(ServerFnError::Args("unauthorized".to_string()));
            }

            let users = DbUser::get_all(&auth.backend.pool).await.unwrap();
            Ok(users)
        } else {
            Ok(vec![db::structs::User::default()])
        }
    }
}
