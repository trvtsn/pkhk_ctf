#[cfg(feature = "ssr")]
use crate::server::{AuthSession, hash_string};
use crate::{error_template::AppError, server::{UserRole, db::{self, structs::{AttachmentWithoutBlob, DbUser, Event}}, enums::ResultStatus, structs::ApiResult}};
use cfg_if::cfg_if;
use chrono::NaiveDateTime;
use leptos::{prelude::*, server_fn::codec::{MultipartData, MultipartFormData}};
// #[cfg(feature = "ssr")]
// use leptos_use::{UseEventSourceMessage, UseEventSourceOptions, UseEventSourceReturn, use_event_source_with_options};
use serde::{Deserialize, Serialize};
use tracing::instrument;

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
        flag: String,
        attachment: Option<AttachmentWithoutBlob>
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
        flag: String,
        attachment: Option<AttachmentWithoutBlob>
    }
}

#[server(name=AdminChallengeApi, prefix="/api/admin", endpoint="challenge")]
#[instrument]
pub async fn challenge(action: ChallengeAction) -> Result<ApiResult<String>, AppError> {
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
                ChallengeAction::Create { event_id, name, description, category, difficulty, points, flag, attachment } => {
                    let flag_hash = hash_string(flag.clone())?;
                    let mut tx = auth.backend.pool.begin().await?;
                    let new_challenge_id = match db::structs::Challenge::add(&event_id, &name, &description, &category, &difficulty, &points, &flag_hash, &mut *tx).await {
                        Ok(result) => result,
                        Err(e) => {
                            tx.rollback().await?;
                            tracing::error!(error = ?e);
                            return Err(AppError::InternalError(e.to_string()))
                        }
                    };

                    match attachment {
                        Some(attachment) => {
                            match db::structs::Attachment::edit_challenge(&attachment.id, &new_challenge_id, &mut *tx).await {
                                Ok(_) => {
                                    tx.commit().await?;
                                    Ok(ApiResult { result: ResultStatus::Success, details: "created challenge".to_string() })
                                },
                                Err(e) => {
                                    tx.rollback().await?;
                                    tracing::error!(error = ?e);
                                    Err(AppError::InternalError(e.to_string()))
                                }
                            }
                        }
                        None => {
                            tx.commit().await?;
                            Ok(ApiResult { result: ResultStatus::Success, details: "created challenge".to_string() })
                        }
                    }
                }
                ChallengeAction::Delete { id } => {
                    let mut tx = auth.backend.pool.begin().await?;

                    if let Err(e) = db::structs::Submission::delete(&db::enums::SubmissionIdentifier::ChallengeId(id), &mut *tx).await {
                        tx.rollback().await?;
                        tracing::error!(error = ?e);
                        return Ok(ApiResult { result: ResultStatus::Fail, details: "internal error".to_string() });
                    }

                    if let Err(e) = db::structs::Attachment::delete(&db::enums::AttachmentIdentifier::ChallengeId(id), &mut *tx).await {
                        tx.rollback().await?;
                        tracing::error!(error = ?e);
                        return Ok(ApiResult { result: ResultStatus::Fail, details: "internal error".to_string() });
                    }

                    match db::structs::Challenge::delete(&id, &mut *tx).await {
                        Ok(_) => {
                            tx.commit().await?;
                            Ok(ApiResult { result: ResultStatus::Success, details: "deleted challenge".to_string() })
                        },
                        Err(e) => {
                            tx.rollback().await?;
                            tracing::error!(error = ?e);
                            return Ok(ApiResult { result: ResultStatus::Fail, details: "internal error".to_string() });
                        }
                    }
                }
                ChallengeAction::Edit { id, event_id, name, description, category, difficulty, points, flag, attachment } => {
                    let flag_hash = hash_string(flag.clone())?;
                    let mut tx = auth.backend.pool.begin().await?;
                    match db::structs::Challenge::edit(&id, &event_id, &name, &description, &category, &difficulty, &points, &flag_hash, &mut *tx).await {
                        Ok(_) => {},
                        Err(e) => {
                            tx.rollback().await?;
                            tracing::error!(error = ?e);
                            return Ok(ApiResult { result: ResultStatus::Fail, details: "internal error".to_string() });
                        }
                    }

                    match attachment {
                        Some(attachment) => {
                            match db::structs::Attachment::edit_challenge(&attachment.id, &id, &mut *tx).await {
                                Ok(_) => {
                                    tx.commit().await?;
                                    Ok(ApiResult { result: ResultStatus::Success, details: "edited challenge".to_string() })
                                },
                                Err(e) => {
                                    tx.rollback().await?;
                                    Err(AppError::InternalError(e.to_string()))
                                }
                            }
                        }
                        None => {
                            tx.commit().await?;
                            Ok(ApiResult { result: ResultStatus::Success, details: "edited challenge".to_string() })
                        }
                    }
                }
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

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
#[instrument]
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
                        Err(e) => {
                            tracing::error!(error = ?e);
                            Ok(ApiResult { result: ResultStatus::Fail, details: Some("internal error".to_string()) })
                        }
                    }
                }
                EventAction::Delete { id } => {
                    match db::structs::Event::delete(&id, &auth.backend.pool).await {
                        Ok(_) => Ok(ApiResult { result: ResultStatus::Success, details: Some("deleted event".to_string()) }),
                        Err(e) => {
                            tracing::error!(error = ?e);
                            Ok(ApiResult { result: ResultStatus::Fail, details: Some("internal error".to_string()) })
                        }
                    }
                }
                EventAction::Edit { id, name, description, start_date, end_date } => {
                    match db::structs::Event::edit(&id, &name, &description, &start_date, &end_date, &auth.backend.pool).await {
                        Ok(_) => Ok(ApiResult { result: ResultStatus::Success, details: Some("edited event".to_string()) }),
                        Err(e) => {
                            tracing::error!(error = ?e);
                            Ok(ApiResult { result: ResultStatus::Fail, details: Some("internal error".to_string()) })
                        }
                    }
                }
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[server(name=AdminUsersGetAll, prefix="/api/admin", endpoint="users")]
#[instrument]
pub async fn get_all_users() -> Result<Vec<DbUser>, AppError> {
    cfg_if! {
        if #[cfg(feature = "ssr")] {
            let auth = use_context::<AuthSession>().unwrap();
            let user = auth.user.unwrap_or_default();

            if user.role != UserRole::Admin {
                return Err(AppError::Unauthorized);
            }

            match DbUser::get_all(&auth.backend.pool).await {
                Ok(users) => Ok(users),
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

#[server(name=AdminEventsGetAll, prefix="/api/admin", endpoint="events")]
#[instrument]
pub async fn get_all_events() -> Result<Vec<Event>, AppError> {
    cfg_if! {
        if #[cfg(feature = "ssr")] {
            let auth = use_context::<AuthSession>().unwrap();
            let user = auth.user.unwrap_or_default();

            if user.role != UserRole::Admin {
                return Err(AppError::Unauthorized);
            }

            match db::structs::Event::get_all(&auth.backend.pool).await {
                Ok(events) => Ok(events),
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

#[server(input=MultipartFormData, name=AdminUploadFile, prefix="/api/admin/file", endpoint="upload")]
#[instrument(skip(file))]
pub async fn upload_file(file: MultipartData) -> Result<ApiResult<AttachmentWithoutBlob>, AppError> {
    cfg_if! {
        if #[cfg(feature = "ssr")] {
            let auth = use_context::<AuthSession>().unwrap();
            let user = auth.user.unwrap_or_default();

            if user.role != UserRole::Admin {
                return Err(AppError::Unauthorized);
            }

            let mut file_name = String::new();
            let mut file_blob = Vec::<u8>::new();
            let mut mime_type = String::new();

            let mut data = file.into_inner().unwrap();
            while let Ok(Some(mut field)) = data.next_field().await {
                file_name = field.file_name().unwrap_or_default().to_string();
                mime_type = field.content_type().unwrap().to_string();

                while let Ok(Some(chunk)) = field.chunk().await {
                    file_blob.append(&mut chunk.to_vec());
                }
            }

            let insert_id = match db::structs::Attachment::add(&None, &None, &file_name, &file_blob, &db::enums::FileType::Attachment, &Some(mime_type), &auth.backend.pool).await {
                Ok(insert_id) => insert_id,
                Err(e) => {
                    tracing::error!(error = ?e);
                    return Err(AppError::InternalError("internal error".to_string()));
                }
            };

            match db::structs::AttachmentWithoutBlob::get(&db::enums::AttachmentIdentifier::Id(insert_id), &auth.backend.pool).await {
                Ok(attachment) => Ok(ApiResult { result: ResultStatus::Success, details: attachment.unwrap_or_default() }),
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

#[server(name=AdminGetAllCategories, prefix="/api/admin/challenges", endpoint="categories")]
#[instrument]
pub async fn get_all_challenge_categories() -> Result<Vec<String>, AppError> {
    cfg_if! {
        if #[cfg(feature = "ssr")] {
            let auth = use_context::<AuthSession>().unwrap();
            let user = auth.user.unwrap_or_default();

            if user.role != UserRole::Admin {
                return Err(AppError::Unauthorized);
            }

            match db::structs::Challenge::get_all_categories(&auth.backend.pool).await {
                Ok(categories) => Ok(categories),
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

#[server(name=AdminGetAllFiles, prefix="/api/admin/files", endpoint="all")]
#[instrument]
pub async fn get_all_files() -> Result<Vec<AttachmentWithoutBlob>, AppError> {
    cfg_if! {
        if #[cfg(feature = "ssr")] {
            let auth = use_context::<AuthSession>().unwrap();
            let user = auth.user.unwrap_or_default();

            if user.role != UserRole::Admin {
                return Err(AppError::Unauthorized);
            }

            match db::structs::AttachmentWithoutBlob::get_all(&None, &auth.backend.pool).await {
                Ok(attachments) => Ok(attachments),
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
