#[cfg(feature = "ssr")]
use crate::{logging::logs_sse, server::{AuthSession, admin_sse, status_data_sse, structs::AppState}};
use crate::{error_template::AppError, server::{UserRole, db::{enums::UserIdentifier, structs::{AttachmentWithoutBlob, DbHint, DbUser, UserAvatar}}, proxmox::ProxmoxVMInstance, structs::User}, utils::get_context};
#[cfg(feature = "ssr")]
use axum::{Router, routing::get};
use chrono::{DateTime, Local};
#[cfg(feature = "ssr")]
use leptos_axum::ResponseOptions;
#[cfg(feature = "ssr")]
use http::StatusCode;
use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use sqlx::MySqlPool;
use tracing::instrument;

pub mod api;

#[cfg(feature = "ssr")]
pub fn router() -> Router<AppState> {
    Router::<AppState>::new()
        .route("/admin/events", get(admin_sse))
        .route("/admin/logs", get(logs_sse))
        .route("/admin/status", get(status_data_sse))
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct ProxmoxUserInfo {
    pub user: DbUser,
    pub pve_user_id: Option<String>,
    pub pool: Option<String>,
    pub vms: Vec<ProxmoxVMInstance>
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ChallengeAction {
    Create {
        event_id: String, 
        name: String, 
        description: String, 
        category: String,
        difficulty: i8, 
        points: u32, 
        flag: String,
        visible_to_groups: String,
        vm_ids: Option<String>,
        hints: Option<Vec<DbHint>>,
        attachments: Option<Vec<AttachmentWithoutBlob>>,
        illustration: Option<AttachmentWithoutBlob>
    },
    Delete {
        id: String
    },
    Edit {
        id: String,
        event_id: String, 
        name: String, 
        description: String, 
        category: String,
        difficulty: i8, 
        points: u32, 
        flag: String,
        visible_to_groups: String,
        vm_ids: Option<String>,
        hints: Option<Vec<DbHint>>,
        attachments: Option<Vec<AttachmentWithoutBlob>>,
        illustration: Option<AttachmentWithoutBlob>
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum EventAction {
    Create {
        name: String,  
        description: String, 
        start_at: DateTime<Local>, 
        end_at: DateTime<Local>,
        visible_to_groups: String,
        attachments: Option<Vec<AttachmentWithoutBlob>>,
        illustration: Option<AttachmentWithoutBlob>
    },
    Delete {
        id: String
    },
    Edit {
        id: String,
        name: String,  
        description: String, 
        start_at: DateTime<Local>, 
        end_at: DateTime<Local>,
        visible_to_groups: String,
        attachments: Option<Vec<AttachmentWithoutBlob>>,
        illustration: Option<AttachmentWithoutBlob>
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum UserAction {
    Create {
        username: String,
        email: String,
        password: String,
        confirm_password: String,
        role: UserRole,
        avatar: Option<UserAvatar>,
        groups: String
    },
    Delete {
        id: String
    },
    Edit {
        id: String,
        username: String,
        email: String,
        password: String,
        confirm_password: String,
        points: u32,
        role: UserRole,
        avatar: Option<UserAvatar>,
        groups: String
    },
    EditPassword {
        id: String,
        password: String,
        confirm_password: String
    }
}

/// Used at the start of every admin API function. 
/// Checks if the user is logged in, and if they're of role `UserRole::Admin`.
#[cfg(feature = "ssr")]
#[instrument]
pub async fn authenticated_check() -> Result<(User, MySqlPool), AppError> {
    let auth = get_context::<AuthSession>()?;
    let response = get_context::<ResponseOptions>()?;
    let Some(user) = auth.user else {
        response.set_status(StatusCode::FORBIDDEN);
        return Err(AppError::Forbidden);
    };
    let Some(db_user) = DbUser::get(&UserIdentifier::Id(user.id.clone()), &auth.backend.pool).await? else { 
        return Err(AppError::BadRequest("Invalid session".to_string()));
    };

    if db_user.role != UserRole::Admin {
        response.set_status(StatusCode::FORBIDDEN);
        return Err(AppError::Forbidden);
    } else {
        Ok((user, auth.backend.pool))
    }
}

#[cfg(feature = "ssr")]
#[instrument]
async fn fetch_db_user(id: &str, pool: &MySqlPool) -> Result<DbUser, AppError> {
    match DbUser::get(&UserIdentifier::Id(id.to_owned()), pool).await? {
        Some(user) => Ok(user),
        None => Err(AppError::DatabaseError("Failed to fetch user".to_string())),
    }
}
