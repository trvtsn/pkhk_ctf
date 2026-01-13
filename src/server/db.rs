#![allow(clippy::too_many_arguments)]

use crate::{constants, error_template::AppError, server::{db::{enums::{AttachmentIdentifier, FileType, SubmissionIdentifier, UserIdentifier}, structs::{AttachmentWithoutBlob, EventMetadata}}}};
use super::db::structs::{Attachment, Challenge, Event, DbUser, Submission};
use cfg_if::cfg_if;
use chrono::{DateTime, Local};
use leptos::prelude::ServerFnError;

cfg_if! {
    if #[cfg(feature = "ssr")] {
        use crate::server::hash_string;
        use sqlx::MySqlExecutor;
        use sqlx::{MySqlPool, mysql::MySqlPoolOptions};
        use tokio::sync::OnceCell;

        pub static DB: OnceCell<MySqlPool> = OnceCell::const_new();

        pub fn get_db() -> MySqlPool {
            DB.get().expect("DB not initialized").clone()
        }

        pub fn get_db_ref() -> &'static MySqlPool {
            DB.get().expect("DB not initialized")
        }

        pub async fn init_db() -> Result<(), ServerFnError> {
            let pool = connect().await?;
            DB.set(pool).expect("DB already initialized");
            add_admin().await?;
            
            Ok(())
        }

        pub async fn add_admin() -> Result<(), AppError> {
            let username = &constants::config::ADMIN_USERNAME.to_string();
            let email = &constants::config::ADMIN_EMAIL.to_string();
            let password = &constants::config::ADMIN_PASSWORD.to_string();
            let pw_hash = hash_string(password.clone())?;

            match DbUser::get(&UserIdentifier::Email(email.clone()), get_db_ref()).await {
                Ok(Some(_)) => return Ok(()),
                Ok(None) => {},
                Err(e) => return Err(e.into())
            }

            match DbUser::add_admin(username, email, &pw_hash, get_db_ref()).await {
                Ok(_) => Ok(()),
                Err(e) => Err(e.into())
            }
        }

        async fn connect() -> Result<MySqlPool, sqlx::Error> {
            let pool = MySqlPoolOptions::new()
                .max_connections(20)
                .connect(&constants::config::DATABASE_URL)
                .await
                .map_err(|e| {
                    log::error!("Failed to connect to the database: {e}");
                    e
                })?;

            log::info!("Connection to the database is successful!");
            Ok(pool)
        }
    }
}

pub mod structs {
    use crate::server::db::enums::{FileType, UserRole};
    use chrono::{DateTime, Local};
    use serde::{Deserialize, Serialize};
    use time::OffsetDateTime;

    pub type DbUser = User;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct User {
        pub id: String,
        pub username: String,
        pub email: String,
        pub pw_hash: String,
        pub created_at: DateTime<Local>,
        pub last_active_at: DateTime<Local>,
        pub role: UserRole
    }

    #[derive(Clone, PartialEq, Serialize, Deserialize, Default, Eq)]
    pub struct Attachment {
        pub id: String,
        pub challenge_id: Option<String>,
        pub event_id: Option<String>,
        pub user_id: Option<String>,
        pub file_name: String,
        pub file_blob: Vec<u8>,
        pub file_type: FileType,
        pub mime_type: Option<String>,
        pub file_size: Option<i32>
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, Eq)]
    pub struct AttachmentWithoutBlob {
        pub id: String,
        pub challenge_id: Option<String>,
        pub event_id: Option<String>,
        pub user_id: Option<String>,
        pub file_name: String,
        pub file_type: FileType,
        pub mime_type: Option<String>,
        pub file_size: Option<i32>
    }

    #[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, Eq)]
    pub struct Challenge {
        pub id: String,
        pub event_id: String,
        pub name: String,
        pub description: Option<String>,
        pub category: Option<String>,
        pub difficulty: i8,
        pub points: u32
    }

    #[derive(Clone, Default, PartialEq, Serialize, Deserialize, Eq)]
    pub struct ChallengeWithAttachments {
        pub challenge: Challenge,
        pub attachments: Vec<AttachmentWithoutBlob>,
    }

     #[derive(Eq, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Event {
        pub id: String,
        pub name: String,
        pub description: Option<String>,
        pub start_at: DateTime<Local>,
        pub end_at: DateTime<Local>
    }

    #[derive(Clone, PartialEq, Serialize, Deserialize)]
    pub struct Submission {
        pub id: String,
        pub challenge_id: String,
        pub event_id: String,
        pub user_id: String,
        pub points: u32,
        pub solved_at: OffsetDateTime
    }

    #[derive(Clone, PartialEq, Serialize, Deserialize)]
    pub struct SubmissionWithData {
        pub submission: Submission,
        pub user: User,
        pub event: Event,
        pub challenge: Challenge,
        pub solved_at: OffsetDateTime
    }

    #[derive(Clone, PartialEq, Serialize, Deserialize)]
    pub struct EventMetadata {
        pub name: Option<String>,
        pub first_submission: Option<OffsetDateTime>,
        pub last_submission: Option<OffsetDateTime>,
    }
}

pub mod enums {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub enum UserIdentifier {
        Id(String),
        Email(String),
        Username(String),
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub enum AttachmentIdentifier {
        Id(String),
        ChallengeId(String),
        EventId(String),
        FileName(String),
        IdFileName((String, String))
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub enum SubmissionIdentifier {
        Id(String),
        ChallengeId(String),
        EventId(String),
        UserId(String),
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub enum FileIdentifier {
        Id(String),
        FileName(String)
    }

    #[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default)]
    pub enum UserRole {
        Admin,
        #[default]
        Competitor
    }

    impl From<String> for UserRole {
        fn from(s: String) -> Self {
            match s.to_lowercase().as_str() {
                "admin" => UserRole::Admin,
                "competitor" => UserRole::Competitor,
                _ => UserRole::Competitor
            }
        }
    }

    impl std::fmt::Display for UserRole {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let s = match self {
                UserRole::Admin => "admin",
                UserRole::Competitor => "competitor",
            };
            write!(f, "{s}")
        }
    }

    #[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Default, Eq)]
    pub enum FileType {
        #[default]
        Attachment,
        Illustration
    }

    impl From<String> for FileType {
        fn from(s: String) -> Self {
            match s.to_lowercase().as_str() {
                "attachment" => FileType::Attachment,
                "illustration" => FileType::Illustration,
                _ => FileType::Attachment
            }
        }
    }

    impl std::fmt::Display for FileType {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let s = match self {
                FileType::Attachment => "attachment",
                FileType::Illustration => "illustration",
            };
            write!(f, "{s}")
        }
    }
}

cfg_if! {
    if #[cfg(feature = "ssr")] {
        impl DbUser {
            pub async fn add_admin(username: &String, email: &String, pw_hash: &String, executor: impl MySqlExecutor<'_>) -> Result<String, sqlx::Error> {
                let id = uuid::Uuid::new_v4();
                match sqlx::query!(
                    "
                    INSERT INTO users
                    (id, username, email, pw_hash, created_at, last_active_at, role)
                    VALUES
                    (?, ?, ?, ?, ?, ?, ?)
                    ",
                    id.to_string(),
                    username,
                    email,
                    pw_hash,
                    sqlx::types::chrono::Local::now(),
                    sqlx::types::chrono::Local::now(),
                    "admin"
                )
                    .execute(executor)
                    .await {
                        Ok(_) => Ok(id.to_string()),
                        Err(e) => {
                            //log::error!("Failed to get user (ID: {id}): {e}");
                            Err(e)?
                        }
                    }
            }

            pub async fn edit_avatar(user_id: &String, file_name: &String, file_blob: &Vec<u8>, mime_type: &String, executor: impl MySqlExecutor<'_>) -> Result<(), sqlx::Error> {
                let id = uuid::Uuid::new_v4();
                match sqlx::query!(
                    "
                    INSERT INTO attachments
                    (id, user_id, file_name, file_blob, file_type, mime_type)
                    VALUES
                    (?, ?, ?, ?, ?, ?)
                    ",
                    id.to_string(), 
                    user_id,
                    file_name,
                    file_blob,
                    "avatar".to_string(),
                    mime_type
                )
                    .execute(executor)
                    .await {
                        Ok(_) => Ok(()),
                        Err(e) => {
                            //log::error!("Failed to get user (ID: {id}): {e}");
                            Err(e)?
                        }
                    }
            }

            pub async fn edit_password(id: &String, pw_hash: &String, executor: impl MySqlExecutor<'_>) -> Result<(), sqlx::Error> {
                match sqlx::query!(
                    "
                    UPDATE users
                    SET pw_hash = ?
                    WHERE id = ?
                    ",
                    pw_hash,
                    id
                )
                    .execute(executor)
                    .await {
                        Ok(_) => Ok(()),
                        Err(e) => {
                            //log::error!("Failed to get user (ID: {id}): {e}");
                            Err(e)?
                        }
                    }
            }

            pub async fn edit_username(id: &String, username: &String, executor: impl MySqlExecutor<'_>) -> Result<(), sqlx::Error> {
                match sqlx::query!(
                    "
                    UPDATE users
                    SET username = ?
                    WHERE id = ?
                    ",
                    username,
                    id
                )
                    .execute(executor)
                    .await {
                        Ok(_) => Ok(()),
                        Err(e) => {
                            //log::error!("Failed to get user (ID: {id}): {e}");
                            Err(e)?
                        }
                    }
            }

            pub async fn edit_email(id: &String, email: &String, executor: impl MySqlExecutor<'_>) -> Result<(), sqlx::Error> {
                match sqlx::query!(
                    "
                    UPDATE users
                    SET email = ?
                    WHERE id = ?
                    ",
                    email,
                    id
                )
                    .execute(executor)
                    .await {
                        Ok(_) => Ok(()),
                        Err(e) => {
                            //log::error!("Failed to get user (ID: {id}): {e}");
                            Err(e)?
                        }
                    }
            }

            pub async fn get(identifier: &UserIdentifier, executor: impl MySqlExecutor<'_>) -> Result<Option<Self>, sqlx::Error> {
                match identifier {
                    UserIdentifier::Id(id) => {
                        match sqlx::query_as!(
                            Self,
                            "
                            SELECT id, username, email, pw_hash, created_at AS `created_at!: DateTime<Local>`, last_active_at AS `last_active_at!: DateTime<Local>`, role
                            FROM users 
                            WHERE id = ?
                            ", 
                            id
                        )
                            .fetch_optional(executor)
                            .await {
                                Ok(user) => Ok(user),
                                Err(e) => {
                                    //log::error!("Failed to get user (ID: {id}): {e}");
                                    Err(e)?
                                }
                            }
                    }
                    UserIdentifier::Email(email) => {
                        match sqlx::query_as!(
                            Self,
                            "
                            SELECT id, username, email, pw_hash, created_at AS `created_at!: DateTime<Local>`, last_active_at AS `last_active_at!: DateTime<Local>`, role
                            FROM users 
                            WHERE email = ?
                            ", 
                            email
                        )
                            .fetch_optional(executor)
                            .await {
                                Ok(user) => Ok(user),
                                Err(e) => {
                                    //log::error!("Failed to get user (ID: {id}): {e}");
                                    Err(e)?
                                }
                            }
                    }
                    UserIdentifier::Username(username) => {
                        let pattern = format!("%{username}%");
                        match sqlx::query_as!(
                            Self,
                            "
                            SELECT id, username, email, pw_hash, created_at AS `created_at!: DateTime<Local>`, last_active_at AS `last_active_at!: DateTime<Local>`, role
                            FROM users 
                            WHERE username LIKE ?
                            ", 
                            pattern
                        )
                            .fetch_optional(executor)
                            .await {
                                Ok(user) => Ok(user),
                                Err(e) => {
                                    //log::error!("Failed to get user (ID: {id}): {e}");
                                    Err(e)?
                                }
                            }
                    }
                }
            }

            pub async fn get_all(executor: impl MySqlExecutor<'_>) -> Result<Vec<Self>, sqlx::Error> {
                match sqlx::query_as!(
                    Self,
                    "
                    SELECT id, username, email, pw_hash, created_at AS `created_at!: DateTime<Local>`, last_active_at AS `last_active_at!: DateTime<Local>`, role
                    FROM users 
                    "
                )
                    .fetch_all(executor)
                    .await {
                        Ok(users) => Ok(users),
                        Err(e) => {
                            //log::error!("Failed to get user (ID: {id}): {e}");
                            Err(e)?
                        }
                    }
            }

            pub async fn get_avatar(identifier: &UserIdentifier, executor: impl MySqlExecutor<'_>) -> Result<Vec<u8>, sqlx::Error> {
                match identifier {
                    UserIdentifier::Id(id) => {
                        match sqlx::query!(
                            "
                            SELECT file_blob
                            FROM attachments
                            WHERE user_id = ?
                            ",
                            id
                        )
                            .fetch_one(executor)
                            .await {
                                Ok(row) => Ok(row.file_blob),
                                Err(e) => {
                                    //log::error!("Failed to get user (ID: {id}): {e}");
                                    Err(e)?
                                }
                            }
                    }
                    UserIdentifier::Email(email) => {
                        match sqlx::query!(
                            "
                            SELECT file_blob
                            FROM attachments
                            WHERE user_id = (SELECT id FROM users WHERE email = ?)
                            ",
                            email
                        )
                            .fetch_one(executor)
                            .await {
                                Ok(row) => Ok(row.file_blob),
                                Err(e) => {
                                    //log::error!("Failed to get user (ID: {id}): {e}");
                                    Err(e)?
                                }
                            }
                    }
                    UserIdentifier::Username(username) => {
                        match sqlx::query!(
                            "
                            SELECT file_blob
                            FROM attachments
                            WHERE user_id = (SELECT id FROM users WHERE username = ?)
                            ",
                            username
                        )
                            .fetch_one(executor)
                            .await {
                                Ok(row) => Ok(row.file_blob),
                                Err(e) => {
                                    //log::error!("Failed to get user (ID: {id}): {e}");
                                    Err(e)?
                                }
                            }
                    }
                }
            }

            pub async fn is_username_available(username: &String, executor: impl MySqlExecutor<'_>) -> Result<bool, sqlx::Error> {
                match sqlx::query!(
                    "
                    SELECT username
                    FROM users
                    WHERE username = ?
                    ",
                    username
                )
                    .fetch_optional(executor)
                    .await {
                        Ok(row) => {
                            match row {
                                Some(_) => Ok(false),
                                None => Ok(true)
                            }
                        },
                        Err(e) => {
                            //log::error!("Failed to get user (ID: {id}): {e}");
                            Err(e)?
                        }
                    }
            }

            pub async fn is_user_available(email: &String, executor: impl MySqlExecutor<'_>) -> Result<bool, sqlx::Error> {
                match sqlx::query!(
                    "
                    SELECT email
                    FROM users
                    WHERE email = ?
                    ",
                    email
                )
                    .fetch_optional(executor)
                    .await {
                        Ok(row) => {
                            match row {
                                Some(_) => Ok(true),
                                None => Ok(false)
                            }
                        },
                        Err(e) => {
                            //log::error!("Failed to get user (ID: {id}): {e}");
                            Err(e)?
                        }
                    }
            }

            pub async fn add(&self, executor: impl MySqlExecutor<'_>) -> anyhow::Result<String> {
                let id = uuid::Uuid::new_v4();
                match sqlx::query!(
                    "
                    INSERT INTO users (id, username, email, pw_hash, created_at, last_active_at, role) 
                    VALUES (?, ?, ?, ?, ?, ?, ?)
                    ", 
                    id.to_string(),
                    self.username,
                    self.email,
                    self.pw_hash,
                    self.created_at,
                    self.last_active_at,
                    self.role.to_string()
                )
                    .execute(executor)
                    .await {
                        Ok(_) => Ok(id.to_string()),
                        Err(e) => {
                            //log::error!("Failed to get user (ID: {id}): {e}");
                            Err(e)?
                        }
                    }
            }

            pub async fn delete(id: &String, executor: impl MySqlExecutor<'_>) -> Result<(), sqlx::Error> {
                match sqlx::query!(
                    "
                    DELETE FROM users
                    WHERE id = ?
                    ",
                    id
                )
                    .execute(executor)
                    .await {
                        Ok(_) => Ok(()),
                        Err(e) => {
                            //log::error!("Failed to get user (ID: {id}): {e}");
                            Err(e)?
                        }
                    }
            }
        }

        impl Challenge {
            pub async fn add(
                event_id: &String, 
                name: &String, 
                description: &String, 
                category: &String,
                difficulty: &i8, 
                points: &u32, 
                flag_hash: &String, 
                executor: impl MySqlExecutor<'_>
            ) -> Result<String, sqlx::Error> {
                let id = uuid::Uuid::new_v4();
                match sqlx::query!(
                    "
                    INSERT INTO challenges
                    (id, event_id, name, description, category, difficulty, points, flag_hash)
                    VALUES 
                    (?, ?, ?, ?, ?, ?, ?, ?)
                    ", 
                    id.to_string(),
                    event_id,
                    name,
                    description,
                    category,
                    difficulty,
                    points,
                    flag_hash
                )
                    .execute(executor)
                    .await {
                        Ok(_) => Ok(id.to_string()),
                        Err(e) => {
                            //log::error!("Failed to get user (ID: {id}): {e}");
                            Err(e)?
                        }
                    }
            }

            pub async fn delete(id: &String, executor: impl MySqlExecutor<'_>) -> Result<(), sqlx::Error> {
                match sqlx::query!(
                    "
                    DELETE
                    FROM challenges 
                    WHERE id = ?
                    ", 
                    id
                )
                    .execute(executor)
                    .await {
                        Ok(_) => Ok(()),
                        Err(e) => {
                            //log::error!("Failed to get user (ID: {id}): {e}");
                            Err(e)?
                        }
                    }
            }

            pub async fn edit(
                id: &String,
                event_id: &String, 
                name: &String, 
                description: &String, 
                category: &String,
                difficulty: &i8, 
                points: &u32, 
                flag_hash: &String, 
                executor: impl MySqlExecutor<'_>
            ) -> Result<(), sqlx::Error> {
                match sqlx::query!(
                    "
                    UPDATE challenges
                    SET
                    event_id = ?, name = ?, description = ?, category = ?, difficulty = ?, points = ?, flag_hash = ?
                    WHERE 
                    id = ?
                    ", 
                    event_id,
                    name,
                    description,
                    category,
                    difficulty,
                    points,
                    flag_hash,
                    id
                )
                    .execute(executor)
                    .await {
                        Ok(_) => Ok(()),
                        Err(e) => {
                            //log::error!("Failed to get user (ID: {id}): {e}");
                            Err(e)?
                        }
                    }
            }

            pub async fn get(id: &String, executor: impl MySqlExecutor<'_>) -> anyhow::Result<Self> {
                match sqlx::query_as!(
                    Self,
                    "
                    SELECT id, event_id, name, description, category, difficulty, points
                    FROM challenges 
                    WHERE id = ?
                    ", 
                    id
                )
                    .fetch_one(executor)
                    .await {
                        Ok(challenge) => Ok(challenge),
                        Err(e) => {
                            //log::error!("Failed to get user (ID: {id}): {e}");
                            Err(e)?
                        }
                    }
            }

            pub async fn get_all(executor: impl MySqlExecutor<'_>) -> Result<Vec<Self>, sqlx::Error> {
                match sqlx::query_as!(
                    Self,
                    "
                    SELECT id, event_id, name, description, category, difficulty, points
                    FROM challenges
                    "
                )
                    .fetch_all(executor)
                    .await {
                        Ok(challenges) => Ok(challenges),
                        Err(e) => {
                            //log::error!("Failed to get challenges (ID: {id}): {e}");
                            Err(e)?
                        }
                    }
            }

            pub async fn get_all_categories(executor: impl MySqlExecutor<'_>) -> Result<Vec<String>, sqlx::Error> {
                match sqlx::query!(
                    "
                    SELECT category
                    FROM challenges 
                    "
                )
                    .fetch_all(executor)
                    .await {
                        Ok(rows) => {
                            let mut categories = Vec::<String>::new();
                            for row in rows.iter() {
                                let category = row.category.clone().unwrap_or_default().clone();
                                if !category.is_empty() {
                                    categories.push(category);
                                }
                            }
                            categories.dedup();
                            Ok(categories)
                        },
                        Err(e) => {
                            //log::error!("Failed to get user (ID: {id}): {e}");
                            Err(e)?
                        }
                    }
            }

            pub async fn get_attachments(&self, executor: impl MySqlExecutor<'_>) -> Result<Vec<Attachment>, sqlx::Error> {
                match sqlx::query_as!(
                    Attachment,
                    "
                    SELECT id, challenge_id, event_id, user_id, file_name, file_blob, file_type, mime_type, file_size
                    FROM attachments 
                    WHERE challenge_id = ?
                    ", 
                    self.id
                )
                    .fetch_all(executor)
                    .await {
                        Ok(challenges) => Ok(challenges),
                        Err(e) => {
                            //log::error!("Failed to get user (ID: {id}): {e}");
                            Err(e)?
                        }
                    }
            }

            pub async fn get_flag_hash(id: &String, executor: impl MySqlExecutor<'_>) -> Result<String, sqlx::Error> {
                match sqlx::query!(
                    "
                    SELECT flag_hash
                    FROM challenges 
                    WHERE id = ?
                    ", 
                    id
                )
                    .fetch_one(executor)
                    .await {
                        Ok(row) => Ok(row.flag_hash),
                        Err(e) => {
                            //log::error!("Failed to get user (ID: {id}): {e}");
                            Err(e)?
                        }
                    }
            }
        }

        impl Attachment {
            pub async fn add(
                challenge_id: &Option<String>, 
                event_id: &Option<String>, 
                file_name: &String,
                file_blob: &Vec<u8>,
                file_type: &FileType,
                mime_type: &Option<String>,
                executor: impl MySqlExecutor<'_>
            ) -> Result<String, sqlx::Error> {
                let id = uuid::Uuid::new_v4();
                match sqlx::query!(
                    "
                    INSERT INTO attachments
                    (id, challenge_id, event_id, file_name, file_blob, file_type, mime_type)
                    VALUES (?, ?, ?, ?, ?, ?, ?)
                    ", 
                    id.to_string(), 
                    challenge_id,
                    event_id,
                    file_name,
                    file_blob,
                    file_type.to_string(),
                    mime_type
                )
                    .execute(executor)
                    .await {
                        Ok(_) => Ok(id.to_string()),
                        Err(e) => {
                            //log::error!("Failed to get user (ID: {id}): {e}");
                            Err(e)?
                        }
                    }
            }

            pub async fn delete(identifier: &AttachmentIdentifier, executor: impl MySqlExecutor<'_>) -> Result<Option<()>, sqlx::Error> {
                match identifier {
                    AttachmentIdentifier::Id(id) => {
                        match sqlx::query!(
                            "
                            DELETE
                            FROM attachments 
                            WHERE id = ?
                            ", 
                            id
                        )
                            .execute(executor)
                            .await {
                                Ok(_) => Ok(Some(())),
                                Err(e) => {
                                    tracing::error!("db query error (Attachment::delete): {}", e);
                                    Ok(None)
                                }
                            }
                    }
                    AttachmentIdentifier::ChallengeId(challenge_id) => {
                        match sqlx::query!(
                            "
                            DELETE
                            FROM attachments 
                            WHERE challenge_id = ?
                            ", 
                            challenge_id
                        )
                            .execute(executor)
                            .await {
                                Ok(_) => Ok(Some(())),
                                Err(e) => {
                                    tracing::error!("db query error (Attachment::delete): {}", e);
                                    Ok(None)
                                }
                            }
                    }
                    AttachmentIdentifier::EventId(event_id) => {
                        match sqlx::query!(
                            "
                            DELETE
                            FROM attachments 
                            WHERE event_id = ?
                            ", 
                            event_id
                        )
                            .execute(executor)
                            .await {
                                Ok(_) => Ok(Some(())),
                                Err(e) => {
                                    tracing::error!("db query error (Attachment::delete): {}", e);
                                    Ok(None)
                                }
                            }
                    }
                    AttachmentIdentifier::FileName(file_name) => {
                        match sqlx::query!(
                            "
                            DELETE
                            FROM attachments 
                            WHERE file_name = ?
                            ", 
                            file_name
                        )
                            .execute(executor)
                            .await {
                                Ok(_) => Ok(Some(())),
                                Err(e) => {
                                    tracing::error!("db query error (Attachment::delete): {}", e);
                                    Ok(None)
                                }
                            }
                    }
                    AttachmentIdentifier::IdFileName((id, file_name)) => {
                        match sqlx::query!(
                            "
                            DELETE
                            FROM attachments 
                            WHERE id = ? AND file_name = ?
                            ", 
                            id,
                            file_name
                        )
                            .execute(executor)
                            .await {
                                Ok(_) => Ok(Some(())),
                                Err(e) => {
                                    tracing::error!("db query error (Attachment::delete): {}", e);
                                    Ok(None)
                                }
                            }
                    }
                }
            }

            pub async fn edit_event(
                id: &String,
                event_id: &String, 
                executor: impl MySqlExecutor<'_>
            ) -> Result<(), sqlx::Error> {
                match sqlx::query!(
                    "
                    UPDATE attachments
                    SET event_id = ?
                    WHERE id = ?
                    ", 
                    event_id,
                    id
                )
                    .execute(executor)
                    .await {
                        Ok(_) => Ok(()),
                        Err(e) => {
                            //log::error!("Failed to get user (ID: {id}): {e}");
                            Err(e)?
                        }
                    }
            }

            pub async fn edit_challenge(
                id: &String,
                challenge_id: &String, 
                executor: impl MySqlExecutor<'_>
            ) -> Result<(), sqlx::Error> {
                match sqlx::query!(
                    "
                    UPDATE attachments
                    SET challenge_id = ?
                    WHERE id = ?
                    ", 
                    challenge_id,
                    id
                )
                    .execute(executor)
                    .await {
                        Ok(_) => Ok(()),
                        Err(e) => {
                            //log::error!("Failed to get user (ID: {id}): {e}");
                            Err(e)?
                        }
                    }
            }

            pub async fn get_all(identifier: AttachmentIdentifier, executor: impl MySqlExecutor<'_>) -> Result<Vec<Self>, sqlx::Error> {
                match identifier {
                    AttachmentIdentifier::Id(id) => {
                        match sqlx::query_as!(
                            Self,
                            "
                            SELECT id, challenge_id, event_id, user_id, file_name, file_blob, file_type, mime_type, file_size
                            FROM attachments 
                            WHERE id = ?
                            ",
                            id
                        )
                            .fetch_all(executor)
                            .await {
                                Ok(attachments) => Ok(attachments),
                                Err(e) => {
                                    //log::error!("Failed to get user (ID: {id}): {e}");
                                    Err(e)?
                                }
                            }
                    }
                    AttachmentIdentifier::ChallengeId(challenge_id) => {
                        match sqlx::query_as!(
                            Self,
                            "
                            SELECT id, challenge_id, event_id, user_id, file_name, file_blob, file_type, mime_type, file_size
                            FROM attachments 
                            WHERE challenge_id = ?
                            ", 
                            challenge_id
                        )
                            .fetch_all(executor)
                            .await {
                                Ok(attachments) => Ok(attachments),
                                Err(e) => {
                                    //log::error!("Failed to get user (ID: {id}): {e}");
                                    Err(e)?
                                }
                            }
                    }
                    AttachmentIdentifier::EventId(event_id) => {
                        match sqlx::query_as!(
                            Self,
                            "
                            SELECT id, challenge_id, event_id, user_id, file_name, file_blob, file_type, mime_type, file_size
                            FROM attachments 
                            WHERE event_id = ?
                            ", 
                            event_id
                        )
                            .fetch_all(executor)
                            .await {
                                Ok(attachments) => Ok(attachments),
                                Err(e) => {
                                    //log::error!("Failed to get user (ID: {id}): {e}");
                                    Err(e)?
                                }
                            }
                    }
                    AttachmentIdentifier::FileName(file_name) => {
                        match sqlx::query_as!(
                            Self,
                            "
                            SELECT id, challenge_id, event_id, user_id, file_name, file_blob, file_type, mime_type, file_size
                            FROM attachments 
                            WHERE file_name = ?
                            ", 
                            file_name
                        )
                            .fetch_all(executor)
                            .await {
                                Ok(attachments) => Ok(attachments),
                                Err(e) => {
                                    //log::error!("Failed to get user (ID: {id}): {e}");
                                    Err(e)?
                                }
                            }
                    }
                    AttachmentIdentifier::IdFileName((id, file_name)) => {
                        match sqlx::query_as!(
                            Self,
                            "
                            SELECT id, challenge_id, event_id, user_id, file_name, file_blob, file_type, mime_type, file_size
                            FROM attachments 
                            WHERE id = ? AND file_name = ?
                            ", 
                            id,
                            file_name
                        )
                            .fetch_all(executor)
                            .await {
                                Ok(attachments) => Ok(attachments),
                                Err(e) => {
                                    //log::error!("Failed to get user (ID: {id}): {e}");
                                    Err(e)?
                                }
                            }
                    }
                }
            }

            pub async fn get_all_filenames(executor: impl MySqlExecutor<'_>) -> Result<Vec<String>, sqlx::Error> {
                match sqlx::query!(
                    "
                    SELECT file_name
                    FROM attachments 
                    "
                )
                    .fetch_all(executor)
                    .await {
                        Ok(rows) => {
                            let mut filenames = Vec::<String>::new();
                            for row in rows.iter() {
                                filenames.push(row.file_name.clone());
                            }
                            Ok(filenames)
                        },
                        Err(e) => {
                            //log::error!("Failed to get user (ID: {id}): {e}");
                            Err(e)?
                        }
                    }
            }

            pub async fn get_filenames(identifier: &AttachmentIdentifier, executor: impl MySqlExecutor<'_>) -> Result<Vec<String>, sqlx::Error> {
                match identifier {
                    AttachmentIdentifier::Id(id) => {
                        match sqlx::query!(
                            "
                            SELECT file_name
                            FROM attachments 
                            WHERE id = ?
                            ",
                            id
                        )
                            .fetch_all(executor)
                            .await {
                                Ok(rows) => {
                                    let mut filenames = Vec::<String>::new();
                                    for row in rows.iter() {
                                        filenames.push(row.file_name.clone());
                                    }
                                    Ok(filenames)
                                },
                                Err(e) => {
                                    //log::error!("Failed to get user (ID: {id}): {e}");
                                    Err(e)?
                                }
                            }
                    }
                    AttachmentIdentifier::ChallengeId(challenge_id) => {
                        match sqlx::query!(
                            "
                            SELECT file_name
                            FROM attachments 
                            WHERE challenge_id = ?
                            ",
                            challenge_id
                        )
                            .fetch_all(executor)
                            .await {
                                Ok(rows) => {
                                    let mut filenames = Vec::<String>::new();
                                    for row in rows.iter() {
                                        filenames.push(row.file_name.clone());
                                    }
                                    Ok(filenames)
                                },
                                Err(e) => {
                                    //log::error!("Failed to get user (ID: {id}): {e}");
                                    Err(e)?
                                }
                            }
                    }
                    AttachmentIdentifier::EventId(event_id) => {
                        match sqlx::query!(
                            "
                            SELECT file_name
                            FROM attachments 
                            WHERE event_id = ?
                            ",
                            event_id
                        )
                            .fetch_all(executor)
                            .await {
                                Ok(rows) => {
                                    let mut filenames = Vec::<String>::new();
                                    for row in rows.iter() {
                                        filenames.push(row.file_name.clone());
                                    }
                                    Ok(filenames)
                                },
                                Err(e) => {
                                    //log::error!("Failed to get user (ID: {id}): {e}");
                                    Err(e)?
                                }
                            }
                    }
                    AttachmentIdentifier::FileName(file_name) => {
                        match sqlx::query!(
                            "
                            SELECT file_name
                            FROM attachments 
                            WHERE file_name = ?
                            ",
                            file_name
                        )
                            .fetch_all(executor)
                            .await {
                                Ok(rows) => {
                                    let mut filenames = Vec::<String>::new();
                                    for row in rows.iter() {
                                        filenames.push(row.file_name.clone());
                                    }
                                    Ok(filenames)
                                },
                                Err(e) => {
                                    //log::error!("Failed to get user (ID: {id}): {e}");
                                    Err(e)?
                                }
                            }
                    }
                    AttachmentIdentifier::IdFileName((id, file_name)) => {
                        match sqlx::query!(
                            "
                            SELECT file_name
                            FROM attachments 
                            WHERE id = ? AND file_name = ?
                            ", 
                            id,
                            file_name
                        )
                            .fetch_all(executor)
                            .await {
                                Ok(rows) => {
                                    let mut filenames = Vec::<String>::new();
                                    for row in rows.iter() {
                                        filenames.push(row.file_name.clone());
                                    }
                                    Ok(filenames)
                                },
                                Err(e) => {
                                    //log::error!("Failed to get user (ID: {id}): {e}");
                                    Err(e)?
                                }
                            }
                    }
                }
            }

            pub async fn get(identifier: AttachmentIdentifier, executor: impl MySqlExecutor<'_>) -> Result<Option<Self>, sqlx::Error> {
                match identifier {
                    AttachmentIdentifier::Id(id) => {
                        match sqlx::query_as!(
                            Self,
                            "
                            SELECT id, challenge_id, event_id, user_id, file_name, file_blob, file_type, mime_type, file_size
                            FROM attachments 
                            WHERE id = ?
                            ",
                            id
                        )
                            .fetch_optional(executor)
                            .await {
                                Ok(attachment) => Ok(attachment),
                                Err(e) => {
                                    //log::error!("Failed to get user (ID: {id}): {e}");
                                    Err(e)?
                                }
                            }
                    }
                    AttachmentIdentifier::ChallengeId(challenge_id) => {
                        match sqlx::query_as!(
                            Self,
                            "
                            SELECT id, challenge_id, event_id, user_id, file_name, file_blob, file_type, mime_type, file_size
                            FROM attachments 
                            WHERE challenge_id = ?
                            ", 
                            challenge_id
                        )
                            .fetch_optional(executor)
                            .await {
                                Ok(attachment) => Ok(attachment),
                                Err(e) => {
                                    //log::error!("Failed to get user (ID: {id}): {e}");
                                    Err(e)?
                                }
                            }
                    }
                    AttachmentIdentifier::EventId(event_id) => {
                        match sqlx::query_as!(
                            Self,
                            "
                            SELECT id, challenge_id, event_id, user_id, file_name, file_blob, file_type, mime_type, file_size
                            FROM attachments 
                            WHERE event_id = ?
                            ", 
                            event_id
                        )
                            .fetch_optional(executor)
                            .await {
                                Ok(attachment) => Ok(attachment),
                                Err(e) => {
                                    //log::error!("Failed to get user (ID: {id}): {e}");
                                    Err(e)?
                                }
                            }
                    }
                    AttachmentIdentifier::FileName(file_name) => {
                        match sqlx::query_as!(
                            Self,
                            "
                            SELECT id, challenge_id, event_id, user_id, file_name, file_blob, file_type, mime_type, file_size
                            FROM attachments 
                            WHERE file_name = ?
                            ", 
                            file_name
                        )
                            .fetch_optional(executor)
                            .await {
                                Ok(attachment) => Ok(attachment),
                                Err(e) => {
                                    //log::error!("Failed to get user (ID: {id}): {e}");
                                    Err(e)?
                                }
                            }
                    }
                    AttachmentIdentifier::IdFileName((id, file_name)) => {
                        match sqlx::query_as!(
                            Self,
                            "
                            SELECT id, challenge_id, event_id, user_id, file_name, file_blob, file_type, mime_type, file_size
                            FROM attachments 
                            WHERE id = ? AND file_name = ?
                            ", 
                            id,
                            file_name
                        )
                            .fetch_optional(executor)
                            .await {
                                Ok(attachment) => Ok(attachment),
                                Err(e) => {
                                    //log::error!("Failed to get user (ID: {id}): {e}");
                                    Err(e)?
                                }
                            }
                    }
                }
            }
        }

        impl AttachmentWithoutBlob {
            pub async fn add(
                challenge_id: &Option<String>, 
                event_id: &Option<String>, 
                file_name: &String,
                file_blob: &Vec<u8>,
                file_type: &FileType,
                mime_type: &Option<String>,
                executor: impl MySqlExecutor<'_>
            ) -> Result<String, sqlx::Error> {
                let id = uuid::Uuid::new_v4();
                match sqlx::query!(
                    "
                    INSERT INTO attachments
                    (id, challenge_id, event_id, file_name, file_blob, file_type, mime_type)
                    VALUES (?, ?, ?, ?, ?, ?, ?)
                    ", 
                    id.to_string(), 
                    challenge_id,
                    event_id,
                    file_name,
                    file_blob,
                    file_type.to_string(),
                    mime_type
                )
                    .execute(executor)
                    .await {
                        Ok(_) => Ok(id.to_string()),
                        Err(e) => {
                            //log::error!("Failed to get user (ID: {id}): {e}");
                            Err(e)?
                        }
                    }
            }

            pub async fn delete(identifier: &AttachmentIdentifier, executor: impl MySqlExecutor<'_>) -> Result<Option<()>, sqlx::Error> {
                match identifier {
                    AttachmentIdentifier::Id(id) => {
                        match sqlx::query!(
                            "
                            DELETE
                            FROM attachments 
                            WHERE id = ?
                            ", 
                            id
                        )
                            .execute(executor)
                            .await {
                                Ok(_) => Ok(Some(())),
                                Err(e) => {
                                    tracing::error!("db query error (Attachment::delete): {}", e);
                                    Ok(None)
                                }
                            }
                    }
                    AttachmentIdentifier::ChallengeId(challenge_id) => {
                        match sqlx::query!(
                            "
                            DELETE
                            FROM attachments 
                            WHERE challenge_id = ?
                            ", 
                            challenge_id
                        )
                            .execute(executor)
                            .await {
                                Ok(_) => Ok(Some(())),
                                Err(e) => {
                                    tracing::error!("db query error (Attachment::delete): {}", e);
                                    Ok(None)
                                }
                            }
                    }
                    AttachmentIdentifier::EventId(event_id) => {
                        match sqlx::query!(
                            "
                            DELETE
                            FROM attachments 
                            WHERE event_id = ?
                            ", 
                            event_id
                        )
                            .execute(executor)
                            .await {
                                Ok(_) => Ok(Some(())),
                                Err(e) => {
                                    tracing::error!("db query error (Attachment::delete): {}", e);
                                    Ok(None)
                                }
                            }
                    }
                    AttachmentIdentifier::FileName(file_name) => {
                        match sqlx::query!(
                            "
                            DELETE
                            FROM attachments 
                            WHERE file_name = ?
                            ", 
                            file_name
                        )
                            .execute(executor)
                            .await {
                                Ok(_) => Ok(Some(())),
                                Err(e) => {
                                    tracing::error!("db query error (Attachment::delete): {}", e);
                                    Ok(None)
                                }
                            }
                    }
                    AttachmentIdentifier::IdFileName((id, file_name)) => {
                        match sqlx::query!(
                            "
                            DELETE
                            FROM attachments 
                            WHERE id = ? AND file_name = ?
                            ", 
                            id,
                            file_name
                        )
                            .execute(executor)
                            .await {
                                Ok(_) => Ok(Some(())),
                                Err(e) => {
                                    tracing::error!("db query error (Attachment::delete): {}", e);
                                    Ok(None)
                                }
                            }
                    }
                }
            }

            pub async fn edit_event(
                id: &String,
                event_id: &String, 
                executor: impl MySqlExecutor<'_>
            ) -> Result<(), sqlx::Error> {
                match sqlx::query!(
                    "
                    UPDATE attachments
                    SET event_id = ?
                    WHERE id = ?
                    ", 
                    event_id,
                    id
                )
                    .execute(executor)
                    .await {
                        Ok(_) => Ok(()),
                        Err(e) => {
                            //log::error!("Failed to get user (ID: {id}): {e}");
                            Err(e)?
                        }
                    }
            }

            pub async fn edit_challenge(
                id: &String,
                challenge_id: &String, 
                executor: impl MySqlExecutor<'_>
            ) -> Result<(), sqlx::Error> {
                match sqlx::query!(
                    "
                    UPDATE attachments
                    SET challenge_id = ?
                    WHERE id = ?
                    ", 
                    challenge_id,
                    id
                )
                    .execute(executor)
                    .await {
                        Ok(_) => Ok(()),
                        Err(e) => {
                            //log::error!("Failed to get user (ID: {id}): {e}");
                            Err(e)?
                        }
                    }
            }

            pub async fn get_all(identifier: &Option<AttachmentIdentifier>, executor: impl MySqlExecutor<'_>) -> Result<Vec<Self>, sqlx::Error> {
                match identifier {
                    Some(AttachmentIdentifier::Id(id)) => {
                        match sqlx::query_as!(
                            Self,
                            "
                            SELECT id, challenge_id, event_id, user_id, file_name, file_type, mime_type, file_size
                            FROM attachments 
                            WHERE id = ?
                            ",
                            id
                        )
                            .fetch_all(executor)
                            .await {
                                Ok(attachments) => Ok(attachments),
                                Err(e) => {
                                    //log::error!("Failed to get user (ID: {id}): {e}");
                                    Err(e)?
                                }
                            }
                    }
                    Some(AttachmentIdentifier::ChallengeId(challenge_id)) => {
                        match sqlx::query_as!(
                            Self,
                            "
                            SELECT id, challenge_id, event_id, user_id, file_name, file_type, mime_type, file_size
                            FROM attachments 
                            WHERE challenge_id = ?
                            ", 
                            challenge_id
                        )
                            .fetch_all(executor)
                            .await {
                                Ok(attachments) => Ok(attachments),
                                Err(e) => {
                                    //log::error!("Failed to get user (ID: {id}): {e}");
                                    Err(e)?
                                }
                            }
                    }
                    Some(AttachmentIdentifier::EventId(event_id)) => {
                        match sqlx::query_as!(
                            Self,
                            "
                            SELECT id, challenge_id, event_id, user_id, file_name, file_type, mime_type, file_size
                            FROM attachments 
                            WHERE event_id = ?
                            ", 
                            event_id
                        )
                            .fetch_all(executor)
                            .await {
                                Ok(attachments) => Ok(attachments),
                                Err(e) => {
                                    //log::error!("Failed to get user (ID: {id}): {e}");
                                    Err(e)?
                                }
                            }
                    }
                    Some(AttachmentIdentifier::FileName(file_name)) => {
                        match sqlx::query_as!(
                            Self,
                            "
                            SELECT id, challenge_id, event_id, user_id, file_name, file_type, mime_type, file_size
                            FROM attachments 
                            WHERE file_name = ?
                            ", 
                            file_name
                        )
                            .fetch_all(executor)
                            .await {
                                Ok(attachments) => Ok(attachments),
                                Err(e) => {
                                    //log::error!("Failed to get user (ID: {id}): {e}");
                                    Err(e)?
                                }
                            }
                    },
                    Some(AttachmentIdentifier::IdFileName((id, file_name))) => {
                        match sqlx::query_as!(
                            Self,
                            "
                            SELECT id, challenge_id, event_id, user_id, file_name, file_type, mime_type, file_size
                            FROM attachments 
                            WHERE id = ? AND file_name = ?
                            ", 
                            id,
                            file_name
                        )
                            .fetch_all(executor)
                            .await {
                                Ok(attachments) => Ok(attachments),
                                Err(e) => {
                                    //log::error!("Failed to get user (ID: {id}): {e}");
                                    Err(e)?
                                }
                            }
                    }
                    None => {
                        match sqlx::query_as!(
                            Self,
                            "
                            SELECT id, challenge_id, event_id, user_id, file_name, file_type, mime_type, file_size
                            FROM attachments 
                            "
                        )
                            .fetch_all(executor)
                            .await {
                                Ok(attachments) => Ok(attachments),
                                Err(e) => {
                                    //log::error!("Failed to get user (ID: {id}): {e}");
                                    Err(e)?
                                }
                            }
                    }
                }
            }

            pub async fn get_all_filenames(executor: impl MySqlExecutor<'_>) -> Result<Vec<String>, sqlx::Error> {
                match sqlx::query!(
                    "
                    SELECT file_name
                    FROM attachments 
                    "
                )
                    .fetch_all(executor)
                    .await {
                        Ok(rows) => {
                            let mut filenames = Vec::<String>::new();
                            for row in rows.iter() {
                                filenames.push(row.file_name.clone());
                            }
                            Ok(filenames)
                        },
                        Err(e) => {
                            //log::error!("Failed to get user (ID: {id}): {e}");
                            Err(e)?
                        }
                    }
            }

            pub async fn get_filenames(identifier: &AttachmentIdentifier, executor: impl MySqlExecutor<'_>) -> Result<Vec<String>, sqlx::Error> {
                match identifier {
                    AttachmentIdentifier::Id(id) => {
                        match sqlx::query!(
                            "
                            SELECT file_name
                            FROM attachments 
                            WHERE id = ?
                            ",
                            id
                        )
                            .fetch_all(executor)
                            .await {
                                Ok(rows) => {
                                    let mut filenames = Vec::<String>::new();
                                    for row in rows.iter() {
                                        filenames.push(row.file_name.clone());
                                    }
                                    Ok(filenames)
                                },
                                Err(e) => {
                                    //log::error!("Failed to get user (ID: {id}): {e}");
                                    Err(e)?
                                }
                            }
                    }
                    AttachmentIdentifier::ChallengeId(challenge_id) => {
                        match sqlx::query!(
                            "
                            SELECT file_name
                            FROM attachments 
                            WHERE challenge_id = ?
                            ",
                            challenge_id
                        )
                            .fetch_all(executor)
                            .await {
                                Ok(rows) => {
                                    let mut filenames = Vec::<String>::new();
                                    for row in rows.iter() {
                                        filenames.push(row.file_name.clone());
                                    }
                                    Ok(filenames)
                                },
                                Err(e) => {
                                    //log::error!("Failed to get user (ID: {id}): {e}");
                                    Err(e)?
                                }
                            }
                    }
                    AttachmentIdentifier::EventId(event_id) => {
                        match sqlx::query!(
                            "
                            SELECT file_name
                            FROM attachments 
                            WHERE event_id = ?
                            ",
                            event_id
                        )
                            .fetch_all(executor)
                            .await {
                                Ok(rows) => {
                                    let mut filenames = Vec::<String>::new();
                                    for row in rows.iter() {
                                        filenames.push(row.file_name.clone());
                                    }
                                    Ok(filenames)
                                },
                                Err(e) => {
                                    //log::error!("Failed to get user (ID: {id}): {e}");
                                    Err(e)?
                                }
                            }
                    }
                    AttachmentIdentifier::FileName(file_name) => {
                        match sqlx::query!(
                            "
                            SELECT file_name
                            FROM attachments 
                            WHERE file_name = ?
                            ",
                            file_name
                        )
                            .fetch_all(executor)
                            .await {
                                Ok(rows) => {
                                    let mut filenames = Vec::<String>::new();
                                    for row in rows.iter() {
                                        filenames.push(row.file_name.clone());
                                    }
                                    Ok(filenames)
                                },
                                Err(e) => {
                                    //log::error!("Failed to get user (ID: {id}): {e}");
                                    Err(e)?
                                }
                            }
                    }
                    AttachmentIdentifier::IdFileName((id, file_name)) => {
                        match sqlx::query!(
                            "
                            SELECT file_name
                            FROM attachments 
                            WHERE id = ? AND file_name = ?
                            ", 
                            id,
                            file_name
                        )
                            .fetch_all(executor)
                            .await {
                                Ok(rows) => {
                                    let mut filenames = Vec::<String>::new();
                                    for row in rows.iter() {
                                        filenames.push(row.file_name.clone());
                                    }
                                    Ok(filenames)
                                },
                                Err(e) => {
                                    //log::error!("Failed to get user (ID: {id}): {e}");
                                    Err(e)?
                                }
                            }
                    }
                }
            }

            pub async fn get(identifier: &AttachmentIdentifier, executor: impl MySqlExecutor<'_>) -> Result<Option<Self>, sqlx::Error> {
                match identifier {
                    AttachmentIdentifier::Id(id) => {
                        match sqlx::query_as!(
                            Self,
                            "
                            SELECT id, challenge_id, event_id, user_id, file_name, file_type, mime_type, file_size
                            FROM attachments 
                            WHERE id = ?
                            ",
                            id
                        )
                            .fetch_optional(executor)
                            .await {
                                Ok(attachment) => Ok(attachment),
                                Err(e) => {
                                    //log::error!("Failed to get user (ID: {id}): {e}");
                                    Err(e)?
                                }
                            }
                    }
                    AttachmentIdentifier::ChallengeId(challenge_id) => {
                        match sqlx::query_as!(
                            Self,
                            "
                            SELECT id, challenge_id, event_id, user_id, file_name, file_type, mime_type, file_size
                            FROM attachments 
                            WHERE challenge_id = ?
                            ", 
                            challenge_id
                        )
                            .fetch_optional(executor)
                            .await {
                                Ok(attachment) => Ok(attachment),
                                Err(e) => {
                                    //log::error!("Failed to get user (ID: {id}): {e}");
                                    Err(e)?
                                }
                            }
                    }
                    AttachmentIdentifier::EventId(event_id) => {
                        match sqlx::query_as!(
                            Self,
                            "
                            SELECT id, challenge_id, event_id, user_id, file_name, file_type, mime_type, file_size
                            FROM attachments 
                            WHERE event_id = ?
                            ", 
                            event_id
                        )
                            .fetch_optional(executor)
                            .await {
                                Ok(attachment) => Ok(attachment),
                                Err(e) => {
                                    //log::error!("Failed to get user (ID: {id}): {e}");
                                    Err(e)?
                                }
                            }
                    }
                    AttachmentIdentifier::FileName(file_name) => {
                        match sqlx::query_as!(
                            Self,
                            "
                            SELECT id, challenge_id, event_id, user_id, file_name, file_type, mime_type, file_size
                            FROM attachments 
                            WHERE file_name = ?
                            ", 
                            file_name
                        )
                            .fetch_optional(executor)
                            .await {
                                Ok(attachment) => Ok(attachment),
                                Err(e) => {
                                    //log::error!("Failed to get user (ID: {id}): {e}");
                                    Err(e)?
                                }
                            }
                    }
                    AttachmentIdentifier::IdFileName((id, file_name)) => {
                        match sqlx::query_as!(
                            Self,
                            "
                            SELECT id, challenge_id, event_id, user_id, file_name, file_type, mime_type, file_size
                            FROM attachments 
                            WHERE id = ? AND file_name = ?
                            ", 
                            id,
                            file_name
                        )
                            .fetch_optional(executor)
                            .await {
                                Ok(attachment) => Ok(attachment),
                                Err(e) => {
                                    //log::error!("Failed to get user (ID: {id}): {e}");
                                    Err(e)?
                                }
                            }
                    }
                }
            }
        }

        impl Event {
            pub async fn add(
                name: &String, 
                description: &String, 
                start_at: &DateTime<Local>, 
                end_at: &DateTime<Local>, 
                executor: impl MySqlExecutor<'_>
            ) -> Result<String, sqlx::Error> {
                let id = uuid::Uuid::new_v4();
                match sqlx::query!(
                    "
                    INSERT INTO events
                    (id, name, description, start_at, end_at)
                    VALUES 
                    (?, ?, ?, ?, ?)
                    ", 
                    id.to_string(), 
                    name,
                    description,
                    start_at,
                    end_at
                )
                    .execute(executor)
                    .await {
                        Ok(_) => Ok(id.to_string()),
                        Err(e) => {
                            //log::error!("Failed to get user (ID: {id}): {e}");
                            Err(e)?
                        }
                    }
            }

            pub async fn delete(id: &String, executor: impl MySqlExecutor<'_>) -> Result<(), sqlx::Error> {
                match sqlx::query!(
                    "
                    DELETE
                    FROM events 
                    WHERE id = ?
                    ", 
                    id
                )
                    .execute(executor)
                    .await {
                        Ok(_) => Ok(()),
                        Err(e) => {
                            //log::error!("Failed to get user (ID: {id}): {e}");
                            Err(e)?
                        }
                    }
            }

            pub async fn edit(
                id: &String,
                name: &String, 
                description: &String, 
                start_at: &DateTime<Local>, 
                end_at: &DateTime<Local>, 
                executor: impl MySqlExecutor<'_>
            ) -> Result<(), sqlx::Error> {
                match sqlx::query!(
                    "
                    UPDATE events
                    SET name = ?, description = ?, start_at = ?, end_at = ?
                    WHERE id = ?
                    ", 
                    name,
                    description,
                    start_at,
                    end_at,
                    id
                )
                    .execute(executor)
                    .await {
                        Ok(_) => Ok(()),
                        Err(e) => {
                            //log::error!("Failed to get user (ID: {id}): {e}");
                            Err(e)?
                        }
                    }
            }

            pub async fn get(id: &String, executor: impl MySqlExecutor<'_>) -> anyhow::Result<Self> {
                match sqlx::query_as!(
                    Self,
                    "
                    SELECT id, name, description, start_at AS `start_at!: DateTime<Local>`, end_at AS `end_at!: DateTime<Local>`
                    FROM events 
                    WHERE id = ?
                    ", 
                    id
                )
                    .fetch_one(executor)
                    .await {
                        Ok(event) => Ok(event),
                        Err(e) => {
                            //log::error!("Failed to get user (ID: {id}): {e}");
                            Err(e)?
                        }
                    }
            }

            pub async fn get_all(executor: impl MySqlExecutor<'_>) -> anyhow::Result<Vec<Self>> {
                match sqlx::query_as!(
                    Self,
                    "
                    SELECT id, name, description, start_at AS `start_at!: DateTime<Local>`, end_at AS `end_at!: DateTime<Local>`
                    FROM events 
                    "
                )
                    .fetch_all(executor)
                    .await {
                        Ok(events) => Ok(events),
                        Err(e) => {
                            //log::error!("Failed to get user (ID: {id}): {e}");
                            Err(e)?
                        }
                    }
            }

            pub async fn get_metadata(id: &String, executor: impl MySqlExecutor<'_>) -> Result<EventMetadata, sqlx::Error> {
                match sqlx::query_as!(
                    EventMetadata,
                    "
                    SELECT events.name,
                        COALESCE(MIN(submissions.solved_at), events.start_at) AS first_submission,
                        COALESCE(MAX(submissions.solved_at), events.end_at) AS last_submission
                    FROM events
                    LEFT JOIN submissions ON submissions.event_id = events.id
                    WHERE events.id = ?
                    ",
                    id
                )
                    .fetch_one(executor)
                    .await {
                        Ok(event_metadata) => Ok(event_metadata),
                        Err(e) => Err(e)
                    }
            }

            pub async fn get_total_possible_points(id: &String, executor: impl MySqlExecutor<'_>) -> Result<u32, sqlx::Error> {
                match sqlx::query!(
                    "
                    SELECT CAST(COALESCE(SUM(points), 0) AS UNSIGNED) AS total_possible_points
                    FROM challenges
                    WHERE event_id = ?
                    ",
                    id
                )
                    .fetch_one(executor)
                    .await {
                        Ok(row) => Ok(row.total_possible_points as u32),
                        Err(e) => Err(e)
                    }
            }
        }

        impl Submission {
            pub async fn add(
                challenge_id: &String, 
                event_id: &String, 
                user_id: &String, 
                points: &u32, 
                solved_at: &DateTime<Local>, 
                executor: impl MySqlExecutor<'_>
            ) -> Result<String, sqlx::Error> {
                let id = uuid::Uuid::new_v4();
                match sqlx::query!(
                    "
                    INSERT INTO submissions
                    (id, challenge_id, event_id, user_id, points, solved_at)
                    VALUES (?, ?, ?, ?, ?, ?)
                    ",
                    id.to_string(),
                    challenge_id,
                    event_id,
                    user_id,
                    points,
                    solved_at
                )
                    .execute(executor)
                    .await {
                        Ok(_) => Ok(id.to_string()),
                        Err(e) => {
                            //log::error!("Failed to get user (ID: {id}): {e}");
                            Err(e)?
                        }
                    }
            }

            pub async fn get_all(executor: impl MySqlExecutor<'_>) -> Result<Vec<Self>, sqlx::Error> {
                match sqlx::query_as!(
                    Self,
                    "
                    SELECT id, challenge_id, event_id, user_id, points, solved_at
                    FROM submissions
                    "
                )
                    .fetch_all(executor)
                    .await {
                        Ok(rows) => Ok(rows),
                        Err(e) => {
                            //log::error!("Failed to get user (ID: {id}): {e}");
                            Err(e)?
                        }
                    }
            }

            pub async fn delete(identifier: &SubmissionIdentifier, executor: impl MySqlExecutor<'_>) -> Result<(), sqlx::Error> {
                match identifier {
                    SubmissionIdentifier::Id(id) => {
                        match sqlx::query!(
                            "
                            DELETE FROM submissions
                            WHERE id = ?
                            ",
                            id
                        )
                            .execute(executor)
                            .await {
                                Ok(_) => Ok(()),
                                Err(e) => {
                                    //log::error!("Failed to get user (ID: {id}): {e}");
                                    Err(e)?
                                }
                            }
                    }
                    SubmissionIdentifier::ChallengeId(challenge_id) => {
                        match sqlx::query!(
                            "
                            DELETE FROM submissions
                            WHERE challenge_id = ?
                            ",
                            challenge_id
                        )
                            .execute(executor)
                            .await {
                                Ok(_) => Ok(()),
                                Err(e) => {
                                    //log::error!("Failed to get user (ID: {id}): {e}");
                                    Err(e)?
                                }
                            }
                    }
                    SubmissionIdentifier::EventId(event_id) => {
                        match sqlx::query!(
                            "
                            DELETE FROM submissions
                            WHERE event_id = ?
                            ",
                            event_id
                        )
                            .execute(executor)
                            .await {
                                Ok(_) => Ok(()),
                                Err(e) => {
                                    //log::error!("Failed to get user (ID: {id}): {e}");
                                    Err(e)?
                                }
                            }
                    }
                    SubmissionIdentifier::UserId(user_id) => {
                        match sqlx::query!(
                            "
                            DELETE FROM submissions
                            WHERE user_id = ?
                            ",
                            user_id
                        )
                            .execute(executor)
                            .await {
                                Ok(_) => Ok(()),
                                Err(e) => {
                                    //log::error!("Failed to get user (ID: {id}): {e}");
                                    Err(e)?
                                }
                            }
                    }
                }
            }

            pub async fn get_user_points(user_id: &String, executor: impl MySqlExecutor<'_>) -> Result<u32, sqlx::Error> {
                match sqlx::query!(
                    "
                    SELECT CAST(COALESCE(SUM(points), 0) AS UNSIGNED) as points
                    FROM submissions
                    WHERE user_id = ?
                    ",
                    user_id
                )
                    .fetch_one(executor)
                    .await {
                        Ok(row) => Ok(row.points as u32),
                        Err(e) => {
                            //log::error!("Failed to get user (ID: {id}): {e}");
                            Err(e)?
                        }
                    }
            }

            pub async fn get_user_solved_challenges(user_id: &String, executor: impl MySqlExecutor<'_>) -> Result<Vec<String>, sqlx::Error> {
                match sqlx::query!(
                    "
                    SELECT challenge_id
                    FROM submissions
                    WHERE user_id = ?
                    ",
                    user_id
                )
                    .fetch_all(executor)
                    .await {
                        Ok(rows) => {
                            let mut solved_challenge_ids = Vec::<String>::new();
                            for record in rows {
                                solved_challenge_ids.push(record.challenge_id)
                            }
                            Ok(solved_challenge_ids)
                        },
                        Err(e) => {
                            //log::error!("Failed to get user (ID: {id}): {e}");
                            Err(e)?
                        }
                    }
            }
        }
    }
}
