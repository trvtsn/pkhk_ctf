#![allow(clippy::too_many_arguments)]
use crate::server::db::enums::ProxmoxAuthType;
use crate::server::db::structs::DbHint;
use crate::server::db::structs::DbHintWithoutHint;
use crate::server::db::structs::HintsUsed;
use crate::server::db::structs::ProxmoxArgs;
use crate::server::db::structs::UserAvatar;
use crate::{constants, error_template::AppError, server::db::{enums::{AttachmentIdentifier, FileType, SubmissionIdentifier, UserIdentifier, UserRole}, structs::{AttachmentWithoutBlob, DbUserWithoutPII, EventMetadata, LdapArgs}}};
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
            add_empty_ldap_row().await?;
            add_empty_proxmox_row().await?;
            
            Ok(())
        }

        pub async fn add_admin() -> Result<(), AppError> {
            let username = &constants::config::ADMIN_USERNAME.to_string();
            let email = &constants::config::ADMIN_EMAIL.to_string();
            let password = &constants::config::ADMIN_PASSWORD.to_string();
            let pw_hash = hash_string(&password)?;

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

        pub async fn add_empty_ldap_row() -> Result<(), AppError> {
            match LdapArgs::get(get_db_ref()).await {
                Ok(Some(_)) => return Ok(()),
                Ok(None) => {},
                Err(e) => return Err(e.into())
            }

            match LdapArgs::insert(&"".to_string(), &"".to_string(), &"".to_string(), &"".to_string(), get_db_ref()).await {
                Ok(_) => Ok(()),
                Err(e) => Err(e.into())
            }
        }

        pub async fn add_empty_proxmox_row() -> Result<(), AppError> {
            match ProxmoxArgs::get(get_db_ref()).await {
                Ok(Some(_)) => return Ok(()),
                Ok(None) => {},
                Err(e) => return Err(e.into())
            }

            match ProxmoxArgs::insert(&"".to_string(), &"/api2/json".to_string(), &"templates".to_string(), &"".to_string(), &None, &None, &None, &ProxmoxAuthType::ApiToken, get_db_ref()).await {
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
    use crate::server::db::enums::{FileType, ProxmoxAuthType, UserRole};
    use chrono::{DateTime, Local};
    use leptos::prelude::RwSignal;
    use serde::{Deserialize, Serialize};
    use time::OffsetDateTime;

    pub type DbUser = User;
    pub type DbUserWithoutPII = UserWithoutPII;
    pub type DbHint = Hint;
    pub type DbHintWithoutHint = HintWithoutHint;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct User {
        pub id: String,
        pub username: String,
        pub email: String,
        pub pw_hash: String,
        pub created_at: DateTime<Local>,
        pub last_active_at: DateTime<Local>,
        pub role: UserRole,
        pub points: u32,
        pub groups: String,
        pub auth_type: String
    }

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    pub struct UserWithoutPII {
        pub username: String,
        pub created_at: DateTime<Local>,
        pub last_active_at: DateTime<Local>,
        pub role: UserRole,
        pub points: u32,
        pub groups: String
    }

    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default, Eq)]
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
        pub points: u32,
        pub visible_to_groups: String, // comma separated string
        pub vm_ids: Option<String> // comma separated string
    }

    #[derive(Clone, Default, PartialEq, Serialize, Deserialize, Eq)]
    pub struct ChallengeWithAttachments {
        pub challenge: Challenge,
        pub attachments: Vec<AttachmentWithoutBlob>,
        pub illustration: Option<AttachmentWithoutBlob>
    }

     #[derive(Eq, Clone, PartialEq, Serialize, Deserialize)]
    pub struct Event {
        pub id: String,
        pub name: String,
        pub description: Option<String>,
        pub start_at: DateTime<Local>,
        pub end_at: DateTime<Local>,
        pub visible_to_groups: String // comma separated string
    }

    #[derive(Clone, PartialEq, Serialize, Deserialize, Eq)]
    pub struct EventWithAttachments {
        pub event: Event,
        pub attachments: Vec<AttachmentWithoutBlob>,
        pub illustration: Option<AttachmentWithoutBlob>
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

    #[derive(Debug, Default, Clone, Deserialize, Serialize)]
    pub struct LdapArgs {
        pub url: String,
        pub bind_dn: String,
        pub bind_pw: String,
        pub base_dn: String,
        pub enabled: SqlBool
    }

    #[derive(Debug, Default, Clone, Deserialize, Serialize)]
    pub struct ProxmoxArgs {
        pub base_url: String,
        pub api_path: String,
        pub templates_pool_id: String,
        pub node: String,
        pub username: Option<String>,
        pub password: Option<String>,
        pub api_token: Option<String>,
        pub auth_type: ProxmoxAuthType
    }

    #[derive(Clone, PartialEq, Serialize, Deserialize)]
    pub struct EventMetadata {
        pub name: Option<String>,
        pub first_submission: Option<OffsetDateTime>,
        pub last_submission: Option<OffsetDateTime>,
    }

    #[derive(Default, Serialize, Deserialize, Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
    pub struct SqlBool(pub bool);

    impl From<i8> for SqlBool {
        fn from(value: i8) -> Self {
            match value {
                0 => Self(false),
                1 => Self(true),
                _ => panic!("invalid boolean value {value}"),
            }
        }
    }

    impl Into<bool> for SqlBool {
        fn into(self) -> bool {
            self.0
        }
    }

    #[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Eq)]
    pub struct Hint {
        pub id: String,
        pub hint: String,
        pub challenge_id: String,
        pub points_penalty: u32
    }

    impl Into<crate::pages::admin::challenges::Hint> for DbHint {
        fn into(self) -> crate::pages::admin::challenges::Hint {
            crate::pages::admin::challenges::Hint {
                id: self.id,
                value: RwSignal::new(self.hint),
                points_penalty: RwSignal::new(Some(self.points_penalty))
            }
        }
    }

    #[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Eq)]
    pub struct HintWithoutHint {
        pub id: String,
        pub challenge_id: String,
        pub points_penalty: u32
    }

    impl Into<Hint> for DbHintWithoutHint {
        fn into(self) -> Hint {
            Hint {
                id: self.id,
                hint: "".to_string(),
                challenge_id: self.challenge_id,
                points_penalty: self.points_penalty
            }
        }
    }

    #[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, Eq)]
    pub struct HintsUsed {
        pub id: u32,
        pub challenge_id: String,
        pub user_id: String,
        pub hint_id: String
    }

    #[derive(Debug, Clone, Deserialize, Serialize, Default, PartialEq)]
    pub struct UserAvatar {
        pub user_id: Option<String>,
        pub attachment_id: String,
        pub file_name: String
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
        Illustration,
        Avatar,
        Certificate
    }

    impl From<String> for FileType {
        fn from(s: String) -> Self {
            match s.to_lowercase().as_str() {
                "attachment" => FileType::Attachment,
                "illustration" => FileType::Illustration,
                "avatar" => FileType::Avatar,
                "certificate" => FileType::Certificate,
                _ => FileType::Attachment
            }
        }
    }

    impl std::fmt::Display for FileType {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let s = match self {
                FileType::Attachment => "attachment",
                FileType::Illustration => "illustration",
                FileType::Avatar => "avatar",
                FileType::Certificate => "certificate",
            };
            write!(f, "{s}")
        }
    }

    #[derive(Debug, Default, Clone, Deserialize, Serialize, PartialEq, Eq)]
    pub enum ProxmoxAuthType {
        Ticket,
        #[default]
        ApiToken
    }

    impl From<String> for ProxmoxAuthType {
        fn from(s: String) -> Self {
            match s.to_lowercase().as_str() {
                "ticket" => ProxmoxAuthType::Ticket,
                "api_token" => ProxmoxAuthType::ApiToken,
                _ => ProxmoxAuthType::Ticket
            }
        }
    }

    impl std::fmt::Display for ProxmoxAuthType {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let s = match self {
                ProxmoxAuthType::Ticket => "ticket",
                ProxmoxAuthType::ApiToken => "api_token",
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
                    (id, username, email, pw_hash, created_at, last_active_at, role, points, `groups`, auth_type)
                    VALUES
                    (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                    ",
                    id.to_string(),
                    username,
                    email,
                    pw_hash,
                    sqlx::types::chrono::Local::now(),
                    sqlx::types::chrono::Local::now(),
                    "admin",
                    0,
                    "administrator",
                    "normal"
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

            pub async fn delete_avatar(user_id: &String, executor: impl MySqlExecutor<'_>) -> Result<(), sqlx::Error> {
                match sqlx::query!(
                    "
                    DELETE FROM attachments
                    WHERE user_id = ? AND file_type = \"avatar\"
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

            pub async fn deduct_points(&self, points: &u32, executor: impl MySqlExecutor<'_>) -> Result<(), sqlx::Error> {
                match sqlx::query!(
                    "
                    UPDATE users
                    SET points = points - ?
                    WHERE id = ?
                    ",
                    points,
                    self.id
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

            pub async fn edit_role(id: &String, role: &UserRole, executor: impl MySqlExecutor<'_>) -> Result<(), sqlx::Error> {
                match sqlx::query!(
                    "
                    UPDATE users
                    SET role = ?
                    WHERE id = ?
                    ",
                    role.to_string(),
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

            pub async fn edit_points(id: &String, points: &u32, executor: impl MySqlExecutor<'_>) -> Result<(), sqlx::Error> {
                match sqlx::query!(
                    "
                    UPDATE users
                    SET points = ?
                    WHERE id = ?
                    ",
                    points,
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

            pub async fn edit_groups(id: &String, group: &String, executor: impl MySqlExecutor<'_>) -> Result<(), sqlx::Error> {
                match sqlx::query!(
                    "
                    UPDATE users
                    SET `groups` = ?
                    WHERE id = ?
                    ",
                    group,
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

            pub async fn edit_last_active(id: &String, last_active_at: &DateTime<Local>, executor: impl MySqlExecutor<'_>) -> Result<(), sqlx::Error> {
                match sqlx::query!(
                    "
                    UPDATE users
                    SET last_active_at = ?
                    WHERE id = ?
                    ",
                    last_active_at,
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
                            SELECT id, username, email, pw_hash, created_at AS `created_at!: DateTime<Local>`, last_active_at AS `last_active_at!: DateTime<Local>`, role, points, `groups`, auth_type
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
                            SELECT id, username, email, pw_hash, created_at AS `created_at!: DateTime<Local>`, last_active_at AS `last_active_at!: DateTime<Local>`, role, points, `groups`, auth_type
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
                        //let pattern = format!("%{username}%");
                        match sqlx::query_as!(
                            Self,
                            "
                            SELECT id, username, email, pw_hash, created_at AS `created_at!: DateTime<Local>`, last_active_at AS `last_active_at!: DateTime<Local>`, role, points, `groups`, auth_type
                            FROM users 
                            WHERE username = ?
                            ", 
                            username
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
                    SELECT id, username, email, pw_hash, created_at AS `created_at!: DateTime<Local>`, last_active_at AS `last_active_at!: DateTime<Local>`, role, points, `groups`, auth_type
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

            pub async fn get_all_groups(executor: impl MySqlExecutor<'_>) -> Result<Vec<String>, sqlx::Error> {
                match sqlx::query!(
                    "
                    SELECT DISTINCT `groups`
                    FROM users 
                    "
                )
                    .fetch_all(executor)
                    .await {
                        Ok(rows) => {
                            let groups = rows.into_iter()
                                .flat_map(|row| {
                                    row.groups
                                        .split(",")
                                        .map(str::trim)
                                        .filter(|g| !g.is_empty())
                                        .map(String::from)
                                        .collect::<Vec<String>>()
                                })
                                .collect::<Vec<String>>();

                            Ok(groups)
                        },
                        Err(e) => {
                            //log::error!("Failed to get user (ID: {id}): {e}");
                            Err(e)?
                        }
                    }
            }

            pub async fn get_avatar(identifier: &UserIdentifier, executor: impl MySqlExecutor<'_>) -> Result<Option<AttachmentWithoutBlob>, sqlx::Error> {
                match identifier {
                    UserIdentifier::Id(id) => {
                        match sqlx::query_as!(
                            AttachmentWithoutBlob,
                            "
                            SELECT id, challenge_id, event_id, user_id, file_name, file_type, mime_type, file_size
                            FROM attachments
                            WHERE user_id = ? AND file_type = \"avatar\"
                            ",
                            id
                        )
                            .fetch_optional(executor)
                            .await {
                                Ok(avatar) => Ok(avatar),
                                Err(e) => {
                                    //log::error!("Failed to get user (ID: {id}): {e}");
                                    Err(e)?
                                }
                            }
                    }
                    UserIdentifier::Email(email) => {
                        match sqlx::query_as!(
                            AttachmentWithoutBlob,
                            "
                            SELECT id, challenge_id, event_id, user_id, file_name, file_type, mime_type, file_size
                            FROM attachments
                            WHERE user_id = (SELECT id FROM users WHERE email = ?) AND file_type = \"avatar\"
                            ",
                            email
                        )
                            .fetch_optional(executor)
                            .await {
                                Ok(avatar) => Ok(avatar),
                                Err(e) => {
                                    //log::error!("Failed to get user (ID: {id}): {e}");
                                    Err(e)?
                                }
                            }
                    }
                    UserIdentifier::Username(username) => {
                        match sqlx::query_as!(
                            AttachmentWithoutBlob,
                            "
                            SELECT id, challenge_id, event_id, user_id, file_name, file_type, mime_type, file_size
                            FROM attachments
                            WHERE user_id = (SELECT id FROM users WHERE username = ?) AND file_type = \"avatar\"
                            ",
                            username
                        )
                            .fetch_optional(executor)
                            .await {
                                Ok(avatar) => Ok(avatar),
                                Err(e) => {
                                    //log::error!("Failed to get user (ID: {id}): {e}");
                                    Err(e)?
                                }
                            }
                    }
                }
            }

            pub async fn get_avatar_id(identifier: &UserIdentifier, executor: impl MySqlExecutor<'_>) -> Result<Option<String>, sqlx::Error> {
                match identifier {
                    UserIdentifier::Id(id) => {
                        match sqlx::query!(
                            "
                            SELECT id
                            FROM attachments
                            WHERE user_id = ? AND file_type = \"avatar\"
                            ",
                            id
                        )
                            .fetch_optional(executor)
                            .await {
                                Ok(row) => {
                                    match row {
                                        Some(row) => Ok(Some(row.id)),
                                        None => Ok(None)
                                    }
                                },
                                Err(e) => {
                                    //log::error!("Failed to get user (ID: {id}): {e}");
                                    Err(e)?
                                }
                            }
                    }
                    UserIdentifier::Email(email) => {
                        match sqlx::query!(
                            "
                            SELECT id
                            FROM attachments
                            WHERE user_id = (SELECT id FROM users WHERE email = ?) AND file_type = \"avatar\"
                            ",
                            email
                        )
                            .fetch_optional(executor)
                            .await {
                                Ok(row) => {
                                    match row {
                                        Some(row) => Ok(Some(row.id)),
                                        None => Ok(None)
                                    }
                                },
                                Err(e) => {
                                    //log::error!("Failed to get user (ID: {id}): {e}");
                                    Err(e)?
                                }
                            }
                    }
                    UserIdentifier::Username(username) => {
                        match sqlx::query!(
                            "
                            SELECT id
                            FROM attachments
                            WHERE user_id = (SELECT id FROM users WHERE username = ?) AND file_type = \"avatar\"
                            ",
                            username
                        )
                            .fetch_optional(executor)
                            .await {
                                Ok(row) => {
                                    match row {
                                        Some(row) => Ok(Some(row.id)),
                                        None => Ok(None)
                                    }
                                },
                                Err(e) => {
                                    //log::error!("Failed to get user (ID: {id}): {e}");
                                    Err(e)?
                                }
                            }
                    }
                }
            }

            pub async fn get_all_avatar_ids(executor: impl MySqlExecutor<'_>) -> Result<Vec<UserAvatar>, sqlx::Error> {
                match sqlx::query_as!(
                    UserAvatar,
                    "
                    SELECT id AS attachment_id, user_id, file_name
                    FROM attachments
                    WHERE file_type = \"avatar\"
                    "
                )
                    .fetch_all(executor)
                    .await {
                        Ok(rows) => {
                            let all_avatar_ids = rows.iter().map(|a| UserAvatar { 
                                user_id: a.user_id.clone(), 
                                attachment_id: a.attachment_id.clone(),
                                file_name: a.file_name.clone()
                            }).collect::<Vec<UserAvatar>>();

                            Ok(all_avatar_ids)
                        },
                        Err(e) => {
                            //log::error!("Failed to get user (ID: {id}): {e}");
                            Err(e)?
                        }
                    }
            }

            pub async fn get_ldap(identifier: &UserIdentifier, executor: impl MySqlExecutor<'_>) -> Result<Option<Self>, sqlx::Error> {
                match identifier {
                    UserIdentifier::Id(id) => {
                        match sqlx::query_as!(
                            Self,
                            "
                            SELECT id, username, email, pw_hash, created_at AS `created_at!: DateTime<Local>`, last_active_at AS `last_active_at!: DateTime<Local>`, role, points, `groups`, auth_type
                            FROM users 
                            WHERE id = ? AND auth_type = \"ldap\"
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
                            SELECT id, username, email, pw_hash, created_at AS `created_at!: DateTime<Local>`, last_active_at AS `last_active_at!: DateTime<Local>`, role, points, `groups`, auth_type
                            FROM users 
                            WHERE email = ? AND auth_type = \"ldap\"
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
                        //let pattern = format!("%{username}%");
                        match sqlx::query_as!(
                            Self,
                            "
                            SELECT id, username, email, pw_hash, created_at AS `created_at!: DateTime<Local>`, last_active_at AS `last_active_at!: DateTime<Local>`, role, points, `groups`, auth_type
                            FROM users 
                            WHERE username = ? AND auth_type = \"ldap\"
                            ", 
                            username
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
                    INSERT INTO users (id, username, email, pw_hash, created_at, last_active_at, role, points, `groups`, auth_type) 
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                    ", 
                    id.to_string(),
                    self.username,
                    self.email,
                    self.pw_hash,
                    self.created_at,
                    self.last_active_at,
                    self.role.to_string(),
                    self.points,
                    self.groups,
                    "normal"
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

            pub async fn add_ldap(&self, executor: impl MySqlExecutor<'_>) -> anyhow::Result<String> {
                let id = uuid::Uuid::new_v4();
                match sqlx::query!(
                    "
                    INSERT INTO users (id, username, email, pw_hash, created_at, last_active_at, role, points, `groups`, auth_type) 
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                    ", 
                    id.to_string(),
                    self.username,
                    self.email,
                    self.pw_hash,
                    self.created_at,
                    self.last_active_at,
                    self.role.to_string(),
                    self.points,
                    self.groups,
                    "ldap"
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

            pub async fn add_points(id: &String, points: &u32, executor: impl MySqlExecutor<'_>) -> Result<(), sqlx::Error> {
                match sqlx::query!(
                    "
                    UPDATE users
                    SET points = points + ?
                    WHERE id = ?
                    ",
                    points,
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

        impl DbUserWithoutPII {
            pub async fn get(identifier: &UserIdentifier, executor: impl MySqlExecutor<'_>) -> Result<Option<Self>, sqlx::Error> {
                match identifier {
                    UserIdentifier::Id(id) => {
                        match sqlx::query_as!(
                            Self,
                            "
                            SELECT username, created_at AS `created_at!: DateTime<Local>`, last_active_at AS `last_active_at!: DateTime<Local>`, role, points, `groups`
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
                            SELECT username, created_at AS `created_at!: DateTime<Local>`, last_active_at AS `last_active_at!: DateTime<Local>`, role, points, `groups`
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
                        //let pattern = format!("%{username}%");
                        match sqlx::query_as!(
                            Self,
                            "
                            SELECT username, created_at AS `created_at!: DateTime<Local>`, last_active_at AS `last_active_at!: DateTime<Local>`, role, points, `groups`
                            FROM users 
                            WHERE username = ?
                            ", 
                            username
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
                visible_to_groups: &String, 
                vm_ids: &Option<String>, 
                executor: impl MySqlExecutor<'_>
            ) -> Result<String, sqlx::Error> {
                let id = uuid::Uuid::new_v4();
                match sqlx::query!(
                    "
                    INSERT INTO challenges
                    (id, event_id, name, description, category, difficulty, points, flag_hash, visible_to_groups, vm_ids)
                    VALUES 
                    (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                    ", 
                    id.to_string(),
                    event_id,
                    name,
                    description,
                    category,
                    difficulty,
                    points,
                    flag_hash,
                    visible_to_groups,
                    vm_ids
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
                visible_to_groups: &String, 
                vm_ids: &Option<String>, 
                executor: impl MySqlExecutor<'_>
            ) -> Result<(), sqlx::Error> {
                if flag_hash.is_empty() {
                    match sqlx::query!(
                        "
                        UPDATE challenges
                        SET
                        event_id = ?, name = ?, description = ?, category = ?, difficulty = ?, points = ?, visible_to_groups = ?, vm_ids = ?
                        WHERE id = ?
                        ", 
                        event_id,
                        name,
                        description,
                        category,
                        difficulty,
                        points,
                        visible_to_groups,
                        vm_ids,
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
                } else {
                    match sqlx::query!(
                        "
                        UPDATE challenges
                        SET
                        event_id = ?, name = ?, description = ?, category = ?, difficulty = ?, points = ?, flag_hash = ?, visible_to_groups = ?, vm_ids = ?
                        WHERE id = ?
                        ", 
                        event_id,
                        name,
                        description,
                        category,
                        difficulty,
                        points,
                        flag_hash,
                        visible_to_groups,
                        vm_ids,
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

            pub async fn get(id: &String, executor: impl MySqlExecutor<'_>) -> anyhow::Result<Self> {
                match sqlx::query_as!(
                    Self,
                    "
                    SELECT id, event_id, name, description, category, difficulty, points, visible_to_groups, vm_ids
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
                    SELECT id, event_id, name, description, category, difficulty, points, visible_to_groups, vm_ids
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
                user_id: &Option<String>, 
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
                    (id, challenge_id, event_id, user_id, file_name, file_blob, file_type, mime_type)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                    ", 
                    id.to_string(), 
                    challenge_id,
                    event_id,
                    user_id,
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

            pub async fn edit_file_name(
                id: &String,
                file_name: &String, 
                executor: impl MySqlExecutor<'_>
            ) -> Result<(), sqlx::Error> {
                match sqlx::query!(
                    "
                    UPDATE attachments
                    SET file_name = ?
                    WHERE id = ?
                    ", 
                    file_name,
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

            pub async fn edit_illustration(
                id: &String, 
                target_identifier: &AttachmentIdentifier, 
                executor: impl MySqlExecutor<'_>
            ) -> Result<Option<()>, sqlx::Error> {
                match target_identifier {
                    AttachmentIdentifier::Id(target_id) => {
                        match sqlx::query!(
                            "
                            UPDATE attachments 
                            SET id = ?
                            WHERE id = ? AND file_type = \"illustration\"
                            ", 
                            target_id,
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
                            UPDATE attachments 
                            SET challenge_id = ?
                            WHERE id = ? AND file_type = \"illustration\"
                            ", 
                            challenge_id,
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
                    AttachmentIdentifier::EventId(event_id) => {
                        match sqlx::query!(
                            "
                            UPDATE attachments 
                            SET event_id = ?
                            WHERE id = ? AND file_type = \"illustration\"
                            ", 
                            event_id,
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
                    AttachmentIdentifier::FileName(file_name) => {
                        match sqlx::query!(
                            "
                            UPDATE attachments 
                            SET file_name = ?
                            WHERE id = ? AND file_type = \"illustration\"
                            ", 
                            file_name,
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
                    AttachmentIdentifier::IdFileName((target_id, file_name)) => {
                        match sqlx::query!(
                            "
                            UPDATE attachments 
                            SET id = ?, file_name = ?
                            WHERE id = ? AND file_type = \"illustration\"
                            ", 
                            target_id,
                            file_name,
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

            pub async fn get_certificate(executor: impl MySqlExecutor<'_>) -> Result<Option<Self>, sqlx::Error> {
                match sqlx::query_as!(
                    Self,
                    "
                    SELECT id, challenge_id, event_id, user_id, file_name, file_blob, file_type, mime_type, file_size
                    FROM attachments 
                    WHERE file_type = \"certificate\"
                    "
                )
                    .fetch_optional(executor)
                    .await {
                        Ok(certificate) => Ok(certificate),
                        Err(e) => {
                            //log::error!("Failed to get user (ID: {id}): {e}");
                            Err(e)?
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
                user_id: &Option<String>, 
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
                    (id, challenge_id, event_id, user_id, file_name, file_blob, file_type, mime_type)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                    ", 
                    id.to_string(), 
                    challenge_id,
                    event_id,
                    user_id,
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

            pub async fn edit_avatar(id: &String, user_id: &String, executor: impl MySqlExecutor<'_>) -> Result<(), sqlx::Error> {
                match sqlx::query!(
                    "
                    UPDATE attachments
                    SET user_id = ?
                    WHERE id = ?
                    ",
                    user_id,
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

            pub async fn edit_file_name(
                id: &String,
                file_name: &String, 
                executor: impl MySqlExecutor<'_>
            ) -> Result<(), sqlx::Error> {
                match sqlx::query!(
                    "
                    UPDATE attachments
                    SET file_name = ?
                    WHERE id = ?
                    ", 
                    file_name,
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

            pub async fn get_all_illustrations(executor: impl MySqlExecutor<'_>) -> Result<Vec<Self>, sqlx::Error> {
                match sqlx::query_as!(
                    Self,
                    "
                    SELECT id, challenge_id, event_id, user_id, file_name, file_type, mime_type, file_size
                    FROM attachments
                    WHERE file_type = \"illustration\"
                    "
                )
                    .fetch_all(executor)
                    .await {
                        Ok(illustrations) => Ok(illustrations),
                        Err(e) => {
                            //log::error!("Failed to get user (ID: {id}): {e}");
                            Err(e)?
                        }
                    }
            }

            pub async fn get_certificate(executor: impl MySqlExecutor<'_>) -> Result<Option<Self>, sqlx::Error> {
                match sqlx::query_as!(
                    Self,
                    "
                    SELECT id, challenge_id, event_id, user_id, file_name, file_type, mime_type, file_size
                    FROM attachments
                    WHERE file_type = \"certificate\"
                    "
                )
                    .fetch_optional(executor)
                    .await {
                        Ok(certificate) => Ok(certificate),
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

            pub async fn get_id(identifier: &AttachmentIdentifier, executor: impl MySqlExecutor<'_>) -> Result<Option<String>, sqlx::Error> {
                match identifier {
                    AttachmentIdentifier::Id(id) => {
                        match sqlx::query!(
                            "
                            SELECT id
                            FROM attachments 
                            WHERE id = ?
                            ",
                            id
                        )
                            .fetch_optional(executor)
                            .await {
                                Ok(row) => {
                                    match row {
                                        Some(row) => Ok(Some(row.id)),
                                        None => Ok(None)
                                    }
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
                            SELECT id
                            FROM attachments 
                            WHERE challenge_id = ?
                            ", 
                            challenge_id
                        )
                            .fetch_optional(executor)
                            .await {
                                Ok(row) => {
                                    match row {
                                        Some(row) => Ok(Some(row.id)),
                                        None => Ok(None)
                                    }
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
                            SELECT id
                            FROM attachments 
                            WHERE event_id = ?
                            ", 
                            event_id
                        )
                            .fetch_optional(executor)
                            .await {
                                Ok(row) => {
                                    match row {
                                        Some(row) => Ok(Some(row.id)),
                                        None => Ok(None)
                                    }
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
                            SELECT id
                            FROM attachments 
                            WHERE file_name = ?
                            ", 
                            file_name
                        )
                            .fetch_optional(executor)
                            .await {
                                Ok(row) => {
                                    match row {
                                        Some(row) => Ok(Some(row.id)),
                                        None => Ok(None)
                                    }
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
                            SELECT id
                            FROM attachments 
                            WHERE id = ? AND file_name = ?
                            ", 
                            id,
                            file_name
                        )
                            .fetch_optional(executor)
                            .await {
                                Ok(row) => {
                                    match row {
                                        Some(row) => Ok(Some(row.id)),
                                        None => Ok(None)
                                    }
                                },
                                Err(e) => {
                                    //log::error!("Failed to get user (ID: {id}): {e}");
                                    Err(e)?
                                }
                            }
                    }
                }
            }

            pub async fn get_illustration(identifier: &AttachmentIdentifier, executor: impl MySqlExecutor<'_>) -> Result<Option<Self>, sqlx::Error> {
                match identifier {
                    AttachmentIdentifier::Id(id) => {
                        match sqlx::query_as!(
                            Self,
                            "
                            SELECT id, challenge_id, event_id, user_id, file_name, file_type, mime_type, file_size
                            FROM attachments 
                            WHERE id = ? AND file_type = \"illustration\"
                            ",
                            id
                        )
                            .fetch_optional(executor)
                            .await {
                                Ok(illustration) => Ok(illustration),
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
                            WHERE challenge_id = ? AND file_type = \"illustration\"
                            ", 
                            challenge_id
                        )
                            .fetch_optional(executor)
                            .await {
                                Ok(illustration) => Ok(illustration),
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
                            WHERE event_id = ? AND file_type = \"illustration\"
                            ", 
                            event_id
                        )
                            .fetch_optional(executor)
                            .await {
                                Ok(illustration) => Ok(illustration),
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
                            WHERE file_name = ? AND file_type = \"illustration\"
                            ", 
                            file_name
                        )
                            .fetch_optional(executor)
                            .await {
                                Ok(illustration) => Ok(illustration),
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
                            WHERE id = ? AND file_name = ? AND file_type = \"illustration\"
                            ", 
                            id,
                            file_name
                        )
                            .fetch_optional(executor)
                            .await {
                                Ok(illustration) => Ok(illustration),
                                Err(e) => {
                                    //log::error!("Failed to get user (ID: {id}): {e}");
                                    Err(e)?
                                }
                            }
                    }
                }
            }

            pub async fn get_illustration_id(identifier: &AttachmentIdentifier, executor: impl MySqlExecutor<'_>) -> Result<Option<String>, sqlx::Error> {
                match identifier {
                    AttachmentIdentifier::Id(id) => {
                        match sqlx::query!(
                            "
                            SELECT id
                            FROM attachments 
                            WHERE id = ? AND file_type = \"illustration\"
                            ",
                            id
                        )
                            .fetch_optional(executor)
                            .await {
                                Ok(row) => {
                                    match row {
                                        Some(row) => Ok(Some(row.id)),
                                        None => Ok(None)
                                    }
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
                            SELECT id
                            FROM attachments 
                            WHERE challenge_id = ? AND file_type = \"illustration\"
                            ", 
                            challenge_id
                        )
                            .fetch_optional(executor)
                            .await {
                                Ok(row) => {
                                    match row {
                                        Some(row) => Ok(Some(row.id)),
                                        None => Ok(None)
                                    }
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
                            SELECT id
                            FROM attachments 
                            WHERE event_id = ? AND file_type = \"illustration\"
                            ", 
                            event_id
                        )
                            .fetch_optional(executor)
                            .await {
                                Ok(row) => {
                                    match row {
                                        Some(row) => Ok(Some(row.id)),
                                        None => Ok(None)
                                    }
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
                            SELECT id
                            FROM attachments 
                            WHERE file_name = ? AND file_type = \"illustration\"
                            ", 
                            file_name
                        )
                            .fetch_optional(executor)
                            .await {
                                Ok(row) => {
                                    match row {
                                        Some(row) => Ok(Some(row.id)),
                                        None => Ok(None)
                                    }
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
                            SELECT id
                            FROM attachments 
                            WHERE id = ? AND file_name = ? AND file_type = \"illustration\"
                            ", 
                            id,
                            file_name
                        )
                            .fetch_optional(executor)
                            .await {
                                Ok(row) => {
                                    match row {
                                        Some(row) => Ok(Some(row.id)),
                                        None => Ok(None)
                                    }
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

        impl Event {
            pub async fn add(
                name: &String, 
                description: &String, 
                start_at: &DateTime<Local>, 
                end_at: &DateTime<Local>, 
                visible_to_groups: &String, 
                executor: impl MySqlExecutor<'_>
            ) -> Result<String, sqlx::Error> {
                let id = uuid::Uuid::new_v4();
                match sqlx::query!(
                    "
                    INSERT INTO events
                    (id, name, description, start_at, end_at, visible_to_groups)
                    VALUES 
                    (?, ?, ?, ?, ?, ?)
                    ", 
                    id.to_string(), 
                    name,
                    description,
                    start_at,
                    end_at,
                    visible_to_groups
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
                visible_to_groups: &String, 
                executor: impl MySqlExecutor<'_>
            ) -> Result<(), sqlx::Error> {
                match sqlx::query!(
                    "
                    UPDATE events
                    SET name = ?, description = ?, start_at = ?, end_at = ?, visible_to_groups = ?
                    WHERE id = ?
                    ", 
                    name,
                    description,
                    start_at,
                    end_at,
                    visible_to_groups,
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
                    SELECT id, name, description, start_at AS `start_at!: DateTime<Local>`, end_at AS `end_at!: DateTime<Local>`, visible_to_groups
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
                    SELECT id, name, description, start_at AS `start_at!: DateTime<Local>`, end_at AS `end_at!: DateTime<Local>`, visible_to_groups
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
    
        impl LdapArgs {
            pub async fn insert(
                url: &String, 
                bind_dn: &String, 
                bind_pw: &String, 
                base_dn: &String, 
                executor: impl MySqlExecutor<'_>
            ) -> Result<(), sqlx::Error> {
                match sqlx::query!(
                    "
                    INSERT INTO ldap (url, bind_dn, bind_pw, base_dn, enabled)
                    VALUES (?, ?, ?, ?, ?)
                    ",
                    url,
                    bind_dn,
                    bind_pw,
                    base_dn,
                    0
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

            pub async fn update(
                url: &String, 
                bind_dn: &String, 
                bind_pw: &String, 
                base_dn: &String, 
                enabled: &bool, 
                executor: impl MySqlExecutor<'_>
            ) -> Result<(), sqlx::Error> {
                match sqlx::query!(
                    "
                    UPDATE ldap
                    SET url = ?, bind_dn = ?, bind_pw = ?, base_dn = ?, enabled = ?
                    ",
                    url,
                    bind_dn,
                    bind_pw,
                    base_dn,
                    enabled
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

            pub async fn update_certificate(
                file_blob: &Option<Vec<u8>>,
                file_name: &Option<String>,
                mime_type: &Option<String>,
                executor: impl MySqlExecutor<'_>
            ) -> Result<(), sqlx::Error> {
                match sqlx::query!(
                    "
                    UPDATE attachments
                    SET file_blob = ?, file_name = ?, mime_type = ?
                    WHERE file_type = \"certificate\"
                    ",
                    file_blob,
                    file_name,
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

            pub async fn enable(executor: impl MySqlExecutor<'_>) -> Result<(), sqlx::Error> {
                match sqlx::query!(
                    "
                    UPDATE ldap
                    SET enabled = 1
                    "
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

            pub async fn delete(executor: impl MySqlExecutor<'_>) -> Result<(), sqlx::Error> {
                match sqlx::query!(
                    "
                    DELETE FROM ldap
                    "
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

            pub async fn disable(executor: impl MySqlExecutor<'_>) -> Result<(), sqlx::Error> {
                match sqlx::query!(
                    "
                    UPDATE ldap
                    SET enabled = 0
                    "
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

            pub async fn get(executor: impl MySqlExecutor<'_>) -> Result<Option<Self>, sqlx::Error> {
                match sqlx::query_as!(
                    Self,
                    "
                    SELECT url, bind_dn, bind_pw, base_dn, enabled
                    FROM ldap
                    "
                )
                    .fetch_optional(executor)
                    .await {
                        Ok(ldap_args) => Ok(ldap_args),
                        Err(e) => {
                            //log::error!("Failed to get user (ID: {id}): {e}");
                            Err(e)?
                        }
                    }
            }

            pub async fn get_optional(executor: impl MySqlExecutor<'_>) -> Result<Option<Self>, sqlx::Error> {
                match sqlx::query_as!(
                    Self,
                    "
                    SELECT url, bind_dn, bind_pw, base_dn, enabled
                    FROM ldap
                    "
                )
                    .fetch_optional(executor)
                    .await {
                        Ok(ldap_args) => Ok(ldap_args),
                        Err(e) => {
                            //log::error!("Failed to get user (ID: {id}): {e}");
                            Err(e)?
                        }
                    }
            }

            pub async fn get_status(executor: impl MySqlExecutor<'_>) -> Result<bool, sqlx::Error> {
                match sqlx::query!(
                    "
                    SELECT enabled
                    FROM ldap
                    "
                )
                    .fetch_one(executor)
                    .await {
                        Ok(row) => {
                            match row.enabled {
                                0 => Ok(false),
                                1 => Ok(true),
                                _ => Ok(false)
                            }
                        },
                        Err(e) => {
                            //log::error!("Failed to get user (ID: {id}): {e}");
                            Err(e)?
                        }
                    }
            }
        }

        impl ProxmoxArgs {
            pub async fn insert(
                base_url: &String, 
                api_path: &String, 
                templates_pool_id: &String, 
                node: &String, 
                username: &Option<String>, 
                password: &Option<String>, 
                api_token: &Option<String>, 
                auth_type: &ProxmoxAuthType, 
                executor: impl MySqlExecutor<'_>
            ) -> Result<(), sqlx::Error> {
                match sqlx::query!(
                    "
                    INSERT INTO proxmox 
                    (base_url, api_path, templates_pool_id, node, username, password, api_token, auth_type)
                    VALUES 
                    (?, ?, ?, ?, ?, ?, ?, ?)
                    ",
                    base_url,
                    api_path,
                    templates_pool_id,
                    node,
                    username,
                    password,
                    api_token,
                    auth_type.to_string()
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

            pub async fn update(
                base_url: &String, 
                api_path: &String, 
                templates_pool_id: &String, 
                node: &String, 
                username: &Option<String>, 
                password: &Option<String>, 
                api_token: &Option<String>, 
                auth_type: &ProxmoxAuthType, 
                executor: impl MySqlExecutor<'_>
            ) -> Result<(), sqlx::Error> {
                match sqlx::query!(
                    "
                    UPDATE proxmox
                    SET base_url = ?, api_path = ?, templates_pool_id = ?, node = ?, username = ?, password = ?, api_token = ?, auth_type = ?
                    ",
                    base_url,
                    api_path,
                    templates_pool_id,
                    node,
                    username,
                    password,
                    api_token,
                    auth_type.to_string()
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

            pub async fn update_auth_type(
                auth_type: &ProxmoxAuthType, 
                executor: impl MySqlExecutor<'_>
            ) -> Result<(), sqlx::Error> {
                match sqlx::query!(
                    "
                    UPDATE proxmox
                    SET auth_type = ?
                    ",
                    auth_type.to_string()
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

            pub async fn delete(executor: impl MySqlExecutor<'_>) -> Result<(), sqlx::Error> {
                match sqlx::query!(
                    "
                    DELETE FROM proxmox
                    "
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

            pub async fn get(executor: impl MySqlExecutor<'_>) -> Result<Option<Self>, sqlx::Error> {
                match sqlx::query_as!(
                    Self,
                    "
                    SELECT base_url, api_path, templates_pool_id, node, username, password, api_token, auth_type
                    FROM proxmox
                    "
                )
                    .fetch_optional(executor)
                    .await {
                        Ok(ldap_args) => Ok(ldap_args),
                        Err(e) => {
                            //log::error!("Failed to get user (ID: {id}): {e}");
                            Err(e)?
                        }
                    }
            }

            pub async fn get_auth_type(executor: impl MySqlExecutor<'_>) -> Result<ProxmoxAuthType, sqlx::Error> {
                match sqlx::query!(
                    "
                    SELECT auth_type
                    FROM proxmox
                    "
                )
                    .fetch_one(executor)
                    .await {
                        Ok(row) => Ok(row.auth_type.into()),
                        Err(e) => {
                            //log::error!("Failed to get user (ID: {id}): {e}");
                            Err(e)?
                        }
                    }
            }
        }

        impl DbHint {
            pub async fn add(hint: &String, challenge_id: &String, points_penalty: &u32, executor: impl MySqlExecutor<'_>) -> Result<(), sqlx::Error> {
                let id = uuid::Uuid::new_v4();
                match sqlx::query!(
                    "
                    INSERT INTO hints (id, hint, challenge_id, points_penalty)
                    VALUES (?, ?, ?, ?)
                    ",
                    id.to_string(),
                    hint,
                    challenge_id,
                    points_penalty
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

            pub async fn delete(id: &String, executor: impl MySqlExecutor<'_>) -> Result<(), sqlx::Error> {
                match sqlx::query!(
                    "
                    DELETE FROM hints
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

            pub async fn delete_all_from_challenge(challenge_id: &String, executor: impl MySqlExecutor<'_>) -> Result<(), sqlx::Error> {
                match sqlx::query!(
                    "
                    DELETE FROM hints
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

            pub async fn edit(id: &String, hint: &String, challenge_id: &String, points_penalty: &u32, executor: impl MySqlExecutor<'_>) -> Result<(), sqlx::Error> {
                match sqlx::query!(
                    "
                    UPDATE hints
                    SET hint = ?, challenge_id = ?, points_penalty = ?
                    WHERE id = ?
                    ",
                    hint,
                    challenge_id,
                    points_penalty,
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

            pub async fn get(id: &String, executor: impl MySqlExecutor<'_>) -> Result<Self, sqlx::Error> {
                match sqlx::query_as!(
                    Self,
                    "
                    SELECT id, hint, challenge_id, points_penalty
                    FROM hints
                    WHERE id = ?
                    ",
                    id
                )
                    .fetch_one(executor)
                    .await {
                        Ok(hint) => Ok(hint),
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
                    SELECT id, hint, challenge_id, points_penalty
                    FROM hints
                    "
                )
                    .fetch_all(executor)
                    .await {
                        Ok(hints) => Ok(hints),
                        Err(e) => {
                            //log::error!("Failed to get user (ID: {id}): {e}");
                            Err(e)?
                        }
                    }
            }

            pub async fn get_all_from_challenge(challenge_id: &String, executor: impl MySqlExecutor<'_>) -> Result<Vec<Self>, sqlx::Error> {
                match sqlx::query_as!(
                    Self,
                    "
                    SELECT id, hint, challenge_id, points_penalty
                    FROM hints
                    WHERE challenge_id = ?
                    ",
                    challenge_id
                )
                    .fetch_all(executor)
                    .await {
                        Ok(hints) => Ok(hints),
                        Err(e) => {
                            //log::error!("Failed to get user (ID: {id}): {e}");
                            Err(e)?
                        }
                    }
            }
        }

        impl DbHintWithoutHint {
            pub async fn get_all_hints(executor: impl MySqlExecutor<'_>) -> Result<Vec<Self>, sqlx::Error> {
                match sqlx::query_as!(
                    Self,
                    "
                    SELECT id, challenge_id, points_penalty
                    FROM hints
                    "
                )
                    .fetch_all(executor)
                    .await {
                        Ok(hints) => Ok(hints),
                        Err(e) => {
                            //log::error!("Failed to get user (ID: {id}): {e}");
                            Err(e)?
                        }
                    }
            }

            pub async fn get_challenge_hints(challenge_id: &String, executor: impl MySqlExecutor<'_>) -> Result<Vec<Self>, sqlx::Error> {
                match sqlx::query_as!(
                    Self,
                    "
                    SELECT id, challenge_id, points_penalty
                    FROM hints
                    WHERE challenge_id = ?
                    ",
                    challenge_id
                )
                    .fetch_all(executor)
                    .await {
                        Ok(hints) => Ok(hints),
                        Err(e) => {
                            //log::error!("Failed to get user (ID: {id}): {e}");
                            Err(e)?
                        }
                    }
            }
        }

        impl HintsUsed {
            pub async fn add(challenge_id: &String, user_id: &String, hint_id: &String, executor: impl MySqlExecutor<'_>) -> Result<(), sqlx::Error> {
                match sqlx::query!(
                    "
                    INSERT INTO hints_used (challenge_id, user_id, hint_id)
                    VALUES (?, ?, ?)
                    ",
                    challenge_id,
                    user_id,
                    hint_id
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

            pub async fn get(user: &DbUser, executor: impl MySqlExecutor<'_>) -> Result<Vec<Self>, sqlx::Error> {
                match sqlx::query_as!(
                    Self,
                    "
                    SELECT id, challenge_id, user_id, hint_id
                    FROM hints_used
                    WHERE user_id = ?
                    ",
                    user.id
                )
                    .fetch_all(executor)
                    .await {
                        Ok(hints_used) => Ok(hints_used),
                        Err(e) => {
                            //log::error!("Failed to get user (ID: {id}): {e}");
                            Err(e)?
                        }
                    }
            }

            pub async fn delete_all_from_challenge(challenge_id: &String, executor: impl MySqlExecutor<'_>) -> Result<(), sqlx::Error> {
                match sqlx::query!(
                    "
                    DELETE FROM hints_used
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
        }
    }
}
