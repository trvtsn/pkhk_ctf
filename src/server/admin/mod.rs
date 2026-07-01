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
use zeroize::Zeroize;

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

impl Zeroize for UserAction {
    fn zeroize(&mut self) {
        match self {
            UserAction::Create { username, email, password, confirm_password, .. } => {
                username.zeroize();
                email.zeroize();
                password.zeroize();
                confirm_password.zeroize();
            },
            UserAction::Delete { id } => id.zeroize(),
            UserAction::Edit { id, username, email, password, confirm_password, .. } => {
                id.zeroize();
                username.zeroize();
                email.zeroize();
                password.zeroize();
                confirm_password.zeroize();
            },
            UserAction::EditPassword { id, password, confirm_password } => {
                id.zeroize();
                password.zeroize();
                confirm_password.zeroize();
            }
        }
    }
}

/// Used at the start of every admin API function. 
/// Checks if the user is logged in, and if they're of role `UserRole::Admin`.
#[cfg(feature = "ssr")]
#[instrument]
pub async fn authenticated_check() -> Result<(User, MySqlPool), AppError> {
    let auth = get_context::<AuthSession>()?;
    let response = get_context::<ResponseOptions>()?;
    let user = match auth.user {
        Some(user) => user,
        None => {
            response.set_status(StatusCode::FORBIDDEN);
            return Err(AppError::Forbidden);
        }
    };
    let db_user = match DbUser::get(&UserIdentifier::Id(user.id.clone()), &auth.backend.pool).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Err(AppError::InternalError("failed to fetch user from db".to_string()));
        }
        Err(e) => {
            tracing::error!(error = ?e);
            return Err(AppError::InternalError(e.to_string()));
        }
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
    match DbUser::get(&UserIdentifier::Id(id.to_owned()), pool).await {
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
