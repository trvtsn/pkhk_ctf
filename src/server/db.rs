use crate::{constants, server::{db::{enums::{AttachmentIdentifier, UserIdentifier, UserRole}, structs::EventMetadata}}};
use super::db::structs::{Attachment, Challenge, Event, DbUser, Submission};
use cfg_if::cfg_if;
use chrono::NaiveDateTime;
use leptos::prelude::ServerFnError;

cfg_if! {
    if #[cfg(feature = "ssr")] {
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
            
            Ok(())
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
    use crate::server::db::enums::UserRole;
    use chrono::{DateTime, Utc};
    use time::OffsetDateTime;
    use serde::{Deserialize, Serialize};
    #[cfg(feature = "ssr")]
    use sqlx::prelude::FromRow;

    pub type DbUser = User;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    #[cfg_attr(feature = "ssr", derive(FromRow))]
    pub struct User {
        pub id: u32,
        pub username: String,
        pub email: String,
        pub pw_hash: String,
        pub created_at: OffsetDateTime,
        pub last_active_at: OffsetDateTime,
        pub role: UserRole
    }

    #[derive(Clone, PartialEq, Serialize, Deserialize)]
    pub struct Attachment {
        pub id: u32,
        pub challenge_id: u32,
        pub file_name: String,
        pub file_blob: Vec<u8>,
    }

    #[derive(Clone, Default, PartialEq, Serialize, Deserialize)]
    pub struct Challenge {
        pub id: u32,
        pub event_id: u32,
        pub name: String,
        pub description: Option<String>,
        pub difficulty: i8,
        pub points: u32
    }

    #[derive(Clone, Default, PartialEq, Serialize, Deserialize)]
    pub struct ChallengeWithAttachments {
        pub challenge: Challenge,
        pub attachments: Vec<Attachment>,
    }

     #[derive(Clone, PartialEq, Serialize, Deserialize)]
    pub struct Event {
        pub id: u32,
        pub name: String,
        pub description: Option<String>,
        pub start_date: OffsetDateTime,
        pub end_date: OffsetDateTime
    }

    #[derive(Clone, PartialEq, Serialize, Deserialize)]
    pub struct Submission {
        pub id: u32,
        pub challenge_id: u32,
        pub event_id: u32,
        pub user_id: u32,
        pub points: u32,
        pub solved_at: OffsetDateTime
    }

    #[derive(Clone, PartialEq, Serialize, Deserialize)]
    pub struct SubmissionWithData {
        pub submission: Submission,
        pub user: User,
        pub event: Event,
        pub challenge: Challenge,
        pub solved_at: DateTime<Utc>
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
        Id(u32),
        Email(String),
        Username(String),
    }

    #[derive(Debug, Clone, Deserialize, Serialize)]
    pub enum AttachmentIdentifier {
        Id(u32),
        ChallengeId(u32)
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
}

cfg_if! {
    if #[cfg(feature = "ssr")] {
        impl DbUser {
            pub async fn get(identifier: &UserIdentifier, executor: impl MySqlExecutor<'_>) -> Result<Option<Self>, sqlx::Error> {
                match identifier {
                    UserIdentifier::Id(id) => {
                        match sqlx::query_as!(
                            Self,
                            "
                            SELECT id, username, email, pw_hash, created_at, last_active_at, role
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
                            SELECT id, username, email, pw_hash, created_at, last_active_at, role
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
                            SELECT id, username, email, pw_hash, created_at, last_active_at, role
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
                    SELECT id, username, email, pw_hash, created_at, last_active_at, role
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

            pub async fn get_optional(identifier: UserIdentifier, executor: impl MySqlExecutor<'_>) -> Result<Option<Self>, sqlx::Error> {
                match identifier {
                    UserIdentifier::Id(id) => {
                        match sqlx::query_as!(
                            Self,
                            "
                            SELECT id, username, email, pw_hash, created_at, last_active_at, role
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
                            SELECT id, username, email, pw_hash, created_at, last_active_at, role
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
                            SELECT id, username, email, pw_hash, created_at, last_active_at, role
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

            pub async fn is_admin(id: &u32, executor: impl MySqlExecutor<'_>) -> Result<bool, sqlx::Error> {
                match sqlx::query!(
                    "
                    SELECT role
                    FROM users
                    WHERE id = ?
                    ",
                    id
                )
                    .fetch_one(executor)
                    .await {
                        Ok(row) => {
                            match row.role.to_lowercase().as_str() {
                                "admin" => Ok(true),
                                "competitor" => Ok(false),
                                &_ => Ok(false)
                            }
                        },
                        Err(e) => {
                            //log::error!("Failed to get user (ID: {id}): {e}");
                            Err(e)?
                        }
                    }
            }

            pub async fn add(&self, executor: impl MySqlExecutor<'_>) -> anyhow::Result<u32> {
                match sqlx::query!(
                    "
                    INSERT INTO users (username, email, pw_hash, created_at, last_active_at, role) 
                    VALUES (?, ?, ?, ?, ?, ?)
                    ", 
                    self.username,
                    self.email,
                    self.pw_hash,
                    self.created_at,
                    self.last_active_at,
                    self.role.to_string()
                )
                    .execute(executor)
                    .await {
                        Ok(result) => Ok(result.last_insert_id() as u32),
                        Err(e) => {
                            //log::error!("Failed to get user (ID: {id}): {e}");
                            Err(e)?
                        }
                    }
            }
        }

        impl Challenge {
            pub async fn add(
                event_id: &u32, 
                name: &String, 
                description: &String, 
                difficulty: &i8, 
                points: &u32, 
                flag_hash: &String, 
                executor: impl MySqlExecutor<'_>
            ) -> Result<(), sqlx::Error> {
                match sqlx::query!(
                    "
                    INSERT INTO challenges
                    (event_id, name, description, difficulty, points, flag_hash)
                    VALUES 
                    (?, ?, ?, ?, ?, ?)
                    ", 
                    event_id,
                    name,
                    description,
                    difficulty,
                    points,
                    flag_hash
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

            pub async fn delete(id: &u32, executor: impl MySqlExecutor<'_>) -> Result<(), sqlx::Error> {
                match sqlx::query_as!(
                    Self,
                    "
                    DELETE
                    FROM challenges 
                    WHERE id = ?
                    ", 
                    id
                )
                    .fetch_one(executor)
                    .await {
                        Ok(_) => Ok(()),
                        Err(e) => {
                            //log::error!("Failed to get user (ID: {id}): {e}");
                            Err(e)?
                        }
                    }
            }

            pub async fn edit(
                id: &u32,
                event_id: &u32, 
                name: &String, 
                description: &String, 
                difficulty: &i8, 
                points: &u32, 
                flag_hash: &String, 
                executor: impl MySqlExecutor<'_>
            ) -> Result<(), sqlx::Error> {
                match sqlx::query!(
                    "
                    UPDATE challenges
                    SET
                    event_id = ?, name = ?, description = ?, difficulty = ?, points = ?, flag_hash = ?
                    WHERE 
                    id = ?
                    ", 
                    event_id,
                    name,
                    description,
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

            pub async fn get(id: &u32, executor: impl MySqlExecutor<'_>) -> anyhow::Result<Self> {
                match sqlx::query_as!(
                    Self,
                    "
                    SELECT id, event_id, name, description, difficulty, points
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
                    SELECT id, event_id, name, description, difficulty, points
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

            pub async fn get_attachments(&self, executor: impl MySqlExecutor<'_>) -> Result<Vec<Attachment>, sqlx::Error> {
                match sqlx::query_as!(
                    Attachment,
                    "
                    SELECT id, challenge_id, file_name, file_blob
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

            pub async fn get_flag_hash(id: &u32, executor: impl MySqlExecutor<'_>) -> Result<String, sqlx::Error> {
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

            pub async fn get_points(id: &u32, executor: impl MySqlExecutor<'_>) -> Result<u32, sqlx::Error> {
                match sqlx::query!(
                    "
                    SELECT points
                    FROM challenges 
                    WHERE id = ?
                    ", 
                    id
                )
                    .fetch_one(executor)
                    .await {
                        Ok(row) => Ok(row.points),
                        Err(e) => {
                            //log::error!("Failed to get user (ID: {id}): {e}");
                            Err(e)?
                        }
                    }
            }
        }

        impl Attachment {
            pub async fn add(
                challenge_id: &u32, 
                file_name: &String, 
                executor: impl MySqlExecutor<'_>
            ) -> Result<(), sqlx::Error> {
                match sqlx::query!(
                    "
                    INSERT INTO attachments
                    (challenge_id, file_name, file_blob)
                    VALUES 
                    (?, ?, LOAD_FILE(?))
                    ", 
                    challenge_id,
                    file_name,
                    file_name
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
                            SELECT id, challenge_id, file_name, file_blob
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
                            SELECT id, challenge_id, file_name, file_blob
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
                }
            }
        }

        impl Event {
            pub async fn add(
                name: &String, 
                description: &String, 
                start_date: &NaiveDateTime, 
                end_date: &NaiveDateTime, 
                executor: impl MySqlExecutor<'_>
            ) -> Result<(), sqlx::Error> {
                match sqlx::query!(
                    "
                    INSERT INTO events
                    (name, description, start_date, end_date)
                    VALUES 
                    (?, ?, ?, ?)
                    ", 
                    name,
                    description,
                    start_date,
                    end_date
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

            pub async fn delete(id: &u32, executor: impl MySqlExecutor<'_>) -> Result<(), sqlx::Error> {
                match sqlx::query_as!(
                    Self,
                    "
                    DELETE
                    FROM events 
                    WHERE id = ?
                    ", 
                    id
                )
                    .fetch_one(executor)
                    .await {
                        Ok(_) => Ok(()),
                        Err(e) => {
                            //log::error!("Failed to get user (ID: {id}): {e}");
                            Err(e)?
                        }
                    }
            }

            pub async fn edit(
                id: &u32,
                name: &String, 
                description: &String, 
                start_date: &NaiveDateTime, 
                end_date: &NaiveDateTime, 
                executor: impl MySqlExecutor<'_>
            ) -> Result<(), sqlx::Error> {
                match sqlx::query!(
                    "
                    UPDATE events
                    SET name = ?, description = ?, start_date = ?, end_date = ?
                    WHERE id = ?
                    ", 
                    name,
                    description,
                    start_date,
                    end_date,
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

            pub async fn get(id: &u32, executor: impl MySqlExecutor<'_>) -> anyhow::Result<Self> {
                match sqlx::query_as!(
                    Self,
                    "
                    SELECT id, name, description, start_date, end_date
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

            pub async fn get_metadata(id: &u32, executor: impl MySqlExecutor<'_>) -> Result<EventMetadata, sqlx::Error> {
                match sqlx::query_as!(
                    EventMetadata,
                    "
                    SELECT events.name,
                        COALESCE(MIN(submissions.solved_at), events.start_date) AS first_submission,
                        COALESCE(MAX(submissions.solved_at), events.end_date)   AS last_submission
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

            pub async fn get_total_possible_points(id: &u32, executor: impl MySqlExecutor<'_>) -> Result<u32, sqlx::Error> {
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
        }
    }
}
