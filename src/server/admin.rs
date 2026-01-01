use async_trait::async_trait;
use cfg_if::cfg_if;
use chrono::NaiveDateTime;
use leptos::{prelude::*, server_fn::error::NoCustomError};
use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use crate::server::AuthSession;
use crate::{error_template::AppError, server::{UserRole, db::{self, structs::DbUser}, enums::ResultStatus, structs::{ApiResult, User}}};
#[cfg(feature = "ssr")]
use password_hash::rand_core::OsRng;
use argon2::{Argon2, PasswordHash, PasswordVerifier, password_hash};
use argon2::PasswordHasher;
use password_hash::SaltString;

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
#[serde(rename_all = "lowercase")]
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
    }
}

#[server(name=AdminChallengeApi, prefix="/api/admin", endpoint="challenge")]
pub async fn challenge(action: ChallengeAction) -> Result<ApiResult<Option<String>>, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            // Note that you can still use `leptos_axum::extract().await?` if you want, but since we
            // called `provide_context` from the `server_fn_handler` in `main`, we can do it this way
            // and it feels faster. Get the AuthSession.
            let auth = use_context::<AuthSession>().unwrap();
            let user = auth.user.unwrap_or_default();
            if user.role != UserRole::Admin {
                return Err(AppError::Unauthorized);
            }

            match action {
                ChallengeAction::Create { event_id, name, description, category, difficulty, points, flag } => {
                    let argon2 = Argon2::default();
                    // The salt is used to prevent certain attacks against stored passwords (see the Internet for more)
                    let salt = SaltString::generate(&mut OsRng);
                    // This gives back a data structure with various parts, which can be encoded using
                    // a standard format into a string that's suitable for use in plain-text environments. Argon2id is the
                    // recommended hashing algorithm at the time of this code being published (2024)
                    let flag_hash: PasswordHash = argon2.hash_password(flag.as_bytes(), &salt)
                        .map_err(|e| AppError::InternalError(format!("Password hashing error: {e}")))?;
                    // Now *this* part is what will be put directly into the database as the user's password hash. This is not just
                    // the 32-byte hash function output, it also has other data attached (like the salt). It has to have
                    // a let-binding outside of the macro or the compiler complains.
                    let flag_hash_string = flag_hash.to_string();

                    match db::structs::Challenge::add(&event_id, &name, &description, &category, &difficulty, &points, &flag_hash_string, &auth.backend.pool).await {
                        Ok(_) => Ok(ApiResult { result: ResultStatus::Success, details: Some("created challenge".to_string()) }),
                        Err(e) => Ok(ApiResult { result: ResultStatus::Fail, details: Some(e.to_string()) })
                    }
                }
                ChallengeAction::Delete { id } => {
                    match db::structs::Challenge::delete(&id, &auth.backend.pool).await {
                        Ok(_) => Ok(ApiResult { result: ResultStatus::Success, details: Some("deleted challenge".to_string()) }),
                        Err(e) => Ok(ApiResult { result: ResultStatus::Fail, details: Some(e.to_string()) })
                    }
                }
                ChallengeAction::Edit { id, event_id, name, description, category, difficulty, points, flag } => {
                    match db::structs::Challenge::edit(&id, &event_id, &name, &description, &category, &difficulty, &points, &flag, &auth.backend.pool).await {
                        Ok(_) => Ok(ApiResult { result: ResultStatus::Success, details: Some("edited challenge".to_string()) }),
                        Err(e) => Ok(ApiResult { result: ResultStatus::Fail, details: Some(e.to_string()) })
                    }
                }
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

// #[serde(rename_all = "lowercase")]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
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
pub async fn event(action: EventAction) -> Result<ApiResult<Option<String>>, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            // Note that you can still use `leptos_axum::extract().await?` if you want, but since we
            // called `provide_context` from the `server_fn_handler` in `main`, we can do it this way
            // and it feels faster. Get the AuthSession.
            let auth = use_context::<AuthSession>().unwrap();
            let user = auth.user.unwrap_or_default();
            if user.role != UserRole::Admin {
                return Err(AppError::Unauthorized);
            }

            match action {
                EventAction::Create { name, description, start_date, end_date } => {
                    match db::structs::Event::add(&name, &description, &start_date, &end_date, &auth.backend.pool).await {
                        Ok(_) => Ok(ApiResult { result: ResultStatus::Success, details: Some("created event".to_string()) }),
                        Err(e) => Ok(ApiResult { result: ResultStatus::Fail, details: Some(e.to_string()) })
                    }
                }
                EventAction::Delete { id } => {
                    match db::structs::Event::delete(&id, &auth.backend.pool).await {
                        Ok(_) => Ok(ApiResult { result: ResultStatus::Success, details: Some("deleted event".to_string()) }),
                        Err(e) => Ok(ApiResult { result: ResultStatus::Fail, details: Some(e.to_string()) })
                    }
                }
                EventAction::Edit { id, name, description, start_date, end_date } => {
                    match db::structs::Event::edit(&id, &name, &description, &start_date, &end_date, &auth.backend.pool).await {
                        Ok(_) => Ok(ApiResult { result: ResultStatus::Success, details: Some("edited event".to_string()) }),
                        Err(e) => Ok(ApiResult { result: ResultStatus::Fail, details: Some(e.to_string()) })
                    }
                }
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[server(name=AdminUsersGetAll, prefix="/api/admin", endpoint="users")]
pub async fn get_all_users() -> Result<ApiResult<Vec<DbUser>>, AppError> {
    cfg_if! {
        if #[cfg(feature = "ssr")] {
            let auth = use_context::<AuthSession>().unwrap();
            let user = auth.user.unwrap_or_default();

            if user.role != UserRole::Admin {
                return Err(AppError::Unauthorized);
            }

            match DbUser::get_all(&auth.backend.pool).await {
                Ok(users) => Ok(ApiResult { result: ResultStatus::Success, details: users }),
                Err(e) => Err(AppError::InternalError(e.to_string()))
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}
