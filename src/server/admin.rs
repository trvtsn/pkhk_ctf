#[cfg(feature = "ssr")]
use crate::server::{AuthSession, hash_string, build_and_broadcast};
use crate::{error_template::AppError, server::{AdminEventPayloadKind, UserRole, db::{self, enums::UserIdentifier, structs::{AttachmentWithoutBlob, DbUser, Event, LdapArgs}}, enums::ResultStatus, structs::ApiResult}};
use cfg_if::cfg_if;
use chrono::{DateTime, Local};
#[cfg(feature = "ssr")]
use ldap3::LdapConn;
use leptos::{prelude::*, server_fn::codec::{MultipartData, MultipartFormData}};
#[cfg(feature = "ssr")]
use leptos_axum::ResponseOptions;
#[cfg(feature = "ssr")]
use http::StatusCode;
use serde::{Deserialize, Serialize};
use tracing::instrument;

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
        attachments: Option<Vec<AttachmentWithoutBlob>>,
        illustration: Option<AttachmentWithoutBlob>
    }
}

#[server(name=AdminChallengeApi, prefix="/api/admin", endpoint="challenge")]
#[instrument]
pub async fn challenge(action: ChallengeAction) -> Result<ApiResult<String>, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let auth = use_context::<AuthSession>().unwrap();
            let user = auth.user.unwrap_or_default();
            let response = expect_context::<ResponseOptions>();
            let db_user = match DbUser::get(&UserIdentifier::Id(user.id.clone()), &auth.backend.pool).await {
                Ok(Some(user)) => user,
                Ok(None) => {
                    return Err(AppError::InternalError("internal error".to_string()));
                }
                Err(e) => {
                    tracing::error!(error = ?e);
                    return Err(AppError::InternalError("internal error".to_string()));
                }
            };

            if db_user.role != UserRole::Admin {
                response.set_status(StatusCode::FORBIDDEN);
                return Err(AppError::Forbidden);
            }

            match action {
                ChallengeAction::Create { event_id, name, description, category, difficulty, points, flag, visible_to_groups, attachments, illustration } => {
                    let flag_hash = hash_string(flag.clone())?;
                    let mut tx = auth.backend.pool.begin().await?;
                    let new_challenge_id = match db::structs::Challenge::add(&event_id, &name, &description, &category, &difficulty, &points, &flag_hash, &visible_to_groups, &mut *tx).await {
                        Ok(result) => result,
                        Err(e) => {
                            tx.rollback().await?;
                            tracing::error!(error = ?e);
                            return Err(AppError::InternalError(e.to_string()))
                        }
                    };

                    if let Some(attachments) = attachments {
                        for attachment in attachments {
                            match db::structs::Attachment::edit_challenge(&attachment.id, &new_challenge_id, &mut *tx).await {
                                Ok(_) => {},
                                Err(e) => {
                                    tx.rollback().await?;
                                    tracing::error!(error = ?e);
                                    return Err(AppError::InternalError(e.to_string()));
                                }
                            }
                        }
                    }

                    match illustration {
                        Some(illustration) => {
                                match db::structs::Attachment::edit_illustration(&illustration.id, &db::enums::AttachmentIdentifier::ChallengeId(new_challenge_id), &mut *tx).await {
                                    Ok(_) => {
                                        tx.commit().await?;
                                        _ = build_and_broadcast(AdminEventPayloadKind::NewChallengeCreated).await;
                                        Ok(ApiResult { result: ResultStatus::Success, details: "created challenge".to_string() })
                                    },
                                    Err(e) => {
                                        tx.rollback().await?;
                                        tracing::error!(error = ?e);
                                        return Err(AppError::InternalError(e.to_string()));
                                    }
                                }
                        }
                        None => {
                            tx.commit().await?;
                            _ = build_and_broadcast(AdminEventPayloadKind::NewChallengeCreated).await;
                            Ok(ApiResult { result: ResultStatus::Success, details: "created challenge".to_string() })
                        }
                    }
                }
                ChallengeAction::Delete { id } => {
                    let mut tx = auth.backend.pool.begin().await?;

                    if let Err(e) = db::structs::Submission::delete(&db::enums::SubmissionIdentifier::ChallengeId(id.clone()), &mut *tx).await {
                        tx.rollback().await?;
                        tracing::error!(error = ?e);
                        return Ok(ApiResult { result: ResultStatus::Fail, details: "internal error".to_string() });
                    }

                    if let Err(e) = db::structs::Attachment::delete(&db::enums::AttachmentIdentifier::ChallengeId(id.clone()), &mut *tx).await {
                        tx.rollback().await?;
                        tracing::error!(error = ?e);
                        return Ok(ApiResult { result: ResultStatus::Fail, details: "internal error".to_string() });
                    }

                    match db::structs::Challenge::delete(&id, &mut *tx).await {
                        Ok(_) => {
                            tx.commit().await?;
                            _ = build_and_broadcast(AdminEventPayloadKind::ChallengeDeleted).await;
                            Ok(ApiResult { result: ResultStatus::Success, details: "deleted challenge".to_string() })
                        },
                        Err(e) => {
                            tx.rollback().await?;
                            tracing::error!(error = ?e);
                            return Ok(ApiResult { result: ResultStatus::Fail, details: "internal error".to_string() });
                        }
                    }
                }
                ChallengeAction::Edit { id, event_id, name, description, category, difficulty, points, flag, visible_to_groups, attachments, illustration } => {
                    let flag_hash = hash_string(flag.clone())?;
                    let mut tx = auth.backend.pool.begin().await?;
                    match db::structs::Challenge::edit(&id, &event_id, &name, &description, &category, &difficulty, &points, &flag_hash, &visible_to_groups, &mut *tx).await {
                        Ok(_) => {},
                        Err(e) => {
                            tx.rollback().await?;
                            tracing::error!(error = ?e);
                            return Ok(ApiResult { result: ResultStatus::Fail, details: "internal error".to_string() });
                        }
                    }

                    if let Some(attachments) = attachments {
                        for attachment in attachments {
                            match db::structs::Attachment::edit_challenge(&attachment.id, &id, &mut *tx).await {
                                Ok(_) => {},
                                Err(e) => {
                                    tx.rollback().await?;
                                    tracing::error!(error = ?e);
                                    return Err(AppError::InternalError(e.to_string()));
                                }
                            }
                        }
                    }

                    match illustration {
                        Some(illustration) => {
                                match db::structs::Attachment::edit_illustration(&illustration.id, &db::enums::AttachmentIdentifier::ChallengeId(id), &mut *tx).await {
                                    Ok(_) => {
                                        tx.commit().await?;
                                        _ = build_and_broadcast(AdminEventPayloadKind::ChallengeEdited).await;
                                        Ok(ApiResult { result: ResultStatus::Success, details: "edited challenge".to_string() })
                                    },
                                    Err(e) => {
                                        tx.rollback().await?;
                                        tracing::error!(error = ?e);
                                        return Err(AppError::InternalError(e.to_string()));
                                    }
                                }
                        }
                        None => {
                            tx.commit().await?;
                            _ = build_and_broadcast(AdminEventPayloadKind::ChallengeEdited).await;
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

#[server(name=AdminEventApi, prefix="/api/admin", endpoint="event")]
#[instrument]
pub async fn event(action: EventAction) -> Result<ApiResult<Option<String>>, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let auth = use_context::<AuthSession>().unwrap();
            let user = auth.user.unwrap_or_default();
            let response = expect_context::<ResponseOptions>();
            let db_user = match DbUser::get(&UserIdentifier::Id(user.id.clone()), &auth.backend.pool).await {
                Ok(Some(user)) => user,
                Ok(None) => {
                    return Err(AppError::InternalError("internal error".to_string()));
                }
                Err(e) => {
                    tracing::error!(error = ?e);
                    return Err(AppError::InternalError("internal error".to_string()));
                }
            };

            if db_user.role != UserRole::Admin {
                response.set_status(StatusCode::FORBIDDEN);
                return Err(AppError::Forbidden);
            }

            match action {
                EventAction::Create { name, description, start_at, end_at, visible_to_groups, attachments, illustration } => {
                    let mut tx = auth.backend.pool.begin().await?;
                    let new_event_id = match db::structs::Event::add(&name, &description, &start_at, &end_at, &visible_to_groups, &mut *tx).await {
                        Ok(result) => result,
                        Err(e) => {
                            tracing::error!(error = ?e);
                            tx.rollback().await?;
                            return Ok(ApiResult { result: ResultStatus::Fail, details: Some("internal error".to_string()) });
                        }
                    };

                    if let Some(attachments) = attachments {
                        for attachment in attachments {
                            match db::structs::Attachment::edit_event(&attachment.id, &new_event_id, &mut *tx).await {
                                Ok(_) => {},
                                Err(e) => {
                                    tx.rollback().await?;
                                    tracing::error!(error = ?e);
                                    return Err(AppError::InternalError(e.to_string()));
                                }
                            }
                        }
                    }

                    match illustration {
                        Some(illustration) => {
                                match db::structs::Attachment::edit_illustration(&illustration.id, &db::enums::AttachmentIdentifier::EventId(new_event_id), &mut *tx).await {
                                    Ok(_) => {
                                        tx.commit().await?;
                                        _ = build_and_broadcast(AdminEventPayloadKind::NewEventCreated).await;
                                        Ok(ApiResult { result: ResultStatus::Success, details: Some("created event".to_string()) })
                                    },
                                    Err(e) => {
                                        tx.rollback().await?;
                                        tracing::error!(error = ?e);
                                        return Err(AppError::InternalError(e.to_string()));
                                    }
                                }
                        }
                        None => {
                            tx.commit().await?;
                            _ = build_and_broadcast(AdminEventPayloadKind::NewEventCreated).await;
                            Ok(ApiResult { result: ResultStatus::Success, details: Some("created event".to_string()) })
                        }
                    }
                }
                EventAction::Delete { id } => {
                    match db::structs::Event::delete(&id, &auth.backend.pool).await {
                        Ok(_) => {
                            _ = build_and_broadcast(AdminEventPayloadKind::EventDeleted).await;
                            Ok(ApiResult { result: ResultStatus::Success, details: Some("deleted event".to_string()) })
                        },
                        Err(e) => {
                            tracing::error!(error = ?e);
                            Ok(ApiResult { result: ResultStatus::Fail, details: Some("internal error".to_string()) })
                        }
                    }
                }
                EventAction::Edit { id, name, description, start_at, end_at, visible_to_groups, attachments, illustration } => {
                    let mut tx = auth.backend.pool.begin().await?;
                    match db::structs::Event::edit(&id, &name, &description, &start_at, &end_at, &visible_to_groups, &mut *tx).await {
                        Ok(_) => {},
                        Err(e) => {
                            tracing::error!(error = ?e);
                            tx.rollback().await?;
                            return Ok(ApiResult { result: ResultStatus::Fail, details: Some("internal error".to_string()) });
                        }
                    }

                    if let Some(attachments) = attachments {
                        for attachment in attachments {
                            match db::structs::Attachment::edit_event(&attachment.id, &id, &mut *tx).await {
                                Ok(_) => {},
                                Err(e) => {
                                    tx.rollback().await?;
                                    tracing::error!(error = ?e);
                                    return Err(AppError::InternalError(e.to_string()));
                                }
                            }
                        }
                    }

                    match illustration {
                        Some(illustration) => {
                                match db::structs::Attachment::edit_illustration(&illustration.id, &db::enums::AttachmentIdentifier::EventId(id), &mut *tx).await {
                                    Ok(_) => {
                                        tx.commit().await?;
                                        _ = build_and_broadcast(AdminEventPayloadKind::NewEventCreated).await;
                                        Ok(ApiResult { result: ResultStatus::Success, details: Some("edited event".to_string() )})
                                    },
                                    Err(e) => {
                                        tx.rollback().await?;
                                        tracing::error!(error = ?e);
                                        return Err(AppError::InternalError(e.to_string()));
                                    }
                                }
                        }
                        None => {
                            tx.commit().await?;
                            _ = build_and_broadcast(AdminEventPayloadKind::NewEventCreated).await;
                            Ok(ApiResult { result: ResultStatus::Success, details: Some("edited event".to_string() )})
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
            let response = expect_context::<ResponseOptions>();
            let db_user = match DbUser::get(&UserIdentifier::Id(user.id.clone()), &auth.backend.pool).await {
                Ok(Some(user)) => user,
                Ok(None) => {
                    return Err(AppError::InternalError("internal error".to_string()));
                }
                Err(e) => {
                    tracing::error!(error = ?e);
                    return Err(AppError::InternalError("internal error".to_string()));
                }
            };

            if db_user.role != UserRole::Admin {
                response.set_status(StatusCode::FORBIDDEN);
                return Err(AppError::Forbidden);
            }

            match DbUser::get_all(&auth.backend.pool).await {
                Ok(users) => Ok(users),
                Err(e) => {
                    tracing::error!(error = ?e);
                    response.set_status(StatusCode::INTERNAL_SERVER_ERROR);
                    Err(AppError::InternalError("internal error".to_string()))
                }
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[server(name=AdminUsersGetAllGroups, prefix="/api/admin/users", endpoint="groups")]
#[instrument]
pub async fn get_all_user_groups() -> Result<Vec<String>, AppError> {
    cfg_if! {
        if #[cfg(feature = "ssr")] {
            let auth = use_context::<AuthSession>().unwrap();
            let user = auth.user.unwrap_or_default();
            let response = expect_context::<ResponseOptions>();
            let db_user = match DbUser::get(&UserIdentifier::Id(user.id.clone()), &auth.backend.pool).await {
                Ok(Some(user)) => user,
                Ok(None) => {
                    return Err(AppError::InternalError("internal error".to_string()));
                }
                Err(e) => {
                    tracing::error!(error = ?e);
                    return Err(AppError::InternalError("internal error".to_string()));
                }
            };

            if db_user.role != UserRole::Admin {
                response.set_status(StatusCode::FORBIDDEN);
                return Err(AppError::Forbidden);
            }

            match DbUser::get_all_groups(&auth.backend.pool).await {
                Ok(groups) => Ok(groups),
                Err(e) => {
                    tracing::error!(error = ?e);
                    response.set_status(StatusCode::INTERNAL_SERVER_ERROR);
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
            let response = expect_context::<ResponseOptions>();
            let db_user = match DbUser::get(&UserIdentifier::Id(user.id.clone()), &auth.backend.pool).await {
                Ok(Some(user)) => user,
                Ok(None) => {
                    return Err(AppError::InternalError("internal error".to_string()));
                }
                Err(e) => {
                    tracing::error!(error = ?e);
                    return Err(AppError::InternalError("internal error".to_string()));
                }
            };

            if db_user.role != UserRole::Admin {
                response.set_status(StatusCode::FORBIDDEN);
                return Err(AppError::Forbidden);
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
#[instrument(skip(files))]
pub async fn upload_files(files: MultipartData) -> Result<ApiResult<Vec<AttachmentWithoutBlob>>, AppError> {
    cfg_if! {
        if #[cfg(feature = "ssr")] {
            let auth = use_context::<AuthSession>().unwrap();
            let user = auth.user.unwrap_or_default();
            let response = expect_context::<ResponseOptions>();
            let db_user = match DbUser::get(&UserIdentifier::Id(user.id.clone()), &auth.backend.pool).await {
                Ok(Some(user)) => user,
                Ok(None) => {
                    return Err(AppError::InternalError("internal error".to_string()));
                }
                Err(e) => {
                    tracing::error!(error = ?e);
                    return Err(AppError::InternalError("internal error".to_string()));
                }
            };

            if db_user.role != UserRole::Admin {
                response.set_status(StatusCode::FORBIDDEN);
                return Err(AppError::Forbidden);
            }

            let mut attachments: Vec<AttachmentWithoutBlob> = Vec::new();

            let mut data = files.into_inner().unwrap();
            while let Ok(Some(mut field)) = data.next_field().await {
                let file_name = match field.file_name() {
                    Some(n) => n.to_string(),
                    None => continue,
                };

                let mime_type = field.content_type().map(|ct| ct.to_string()).unwrap_or_default();

                let mut file_blob = Vec::<u8>::new();
                while let Ok(Some(chunk)) = field.chunk().await {
                    file_blob.extend_from_slice(&chunk);
                }

                let insert_id = match db::structs::Attachment::add(&None, &None, &None, &file_name, &file_blob, &db::enums::FileType::Attachment, &Some(mime_type), &auth.backend.pool).await {
                    Ok(insert_id) => insert_id,
                    Err(e) => {
                        tracing::error!(error = ?e);
                        return Err(AppError::InternalError("internal error".to_string()));
                    }
                };

                match db::structs::AttachmentWithoutBlob::get(&db::enums::AttachmentIdentifier::Id(insert_id.clone()), &auth.backend.pool).await {
                    Ok(Some(attachment)) => attachments.push(attachment),
                    Ok(None) => tracing::error!("file upload with insert id {} but could not fetch it from db", insert_id),
                    Err(e) => {
                        tracing::error!(error = ?e, "failed to fetch upload file from db");
                        return Err(AppError::InternalError("internal error".to_string()));
                    }
                }
            }

            Ok(ApiResult { result: ResultStatus::Success, details: attachments })
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[server(input=MultipartFormData, name=AdminUploadIllustration, prefix="/api/admin/illustration", endpoint="upload")]
#[instrument(skip(file))]
pub async fn upload_illustration(file: MultipartData) -> Result<ApiResult<AttachmentWithoutBlob>, AppError> {
    cfg_if! {
        if #[cfg(feature = "ssr")] {
            let auth = use_context::<AuthSession>().unwrap();
            let user = auth.user.unwrap_or_default();
            let response = expect_context::<ResponseOptions>();
            let db_user = match DbUser::get(&UserIdentifier::Id(user.id.clone()), &auth.backend.pool).await {
                Ok(Some(user)) => user,
                Ok(None) => {
                    return Err(AppError::InternalError("internal error".to_string()));
                }
                Err(e) => {
                    tracing::error!(error = ?e);
                    return Err(AppError::InternalError("internal error".to_string()));
                }
            };

            if db_user.role != UserRole::Admin {
                response.set_status(StatusCode::FORBIDDEN);
                return Err(AppError::Forbidden);
            }

            let mut attachment = AttachmentWithoutBlob::default();

            let mut data = file.into_inner().unwrap();
            while let Ok(Some(mut field)) = data.next_field().await {
                let file_name = match field.file_name() {
                    Some(n) => n.to_string(),
                    None => continue,
                };

                let mime_type = field.content_type().map(|ct| ct.to_string()).unwrap_or_default();

                let mut file_blob = Vec::<u8>::new();
                while let Ok(Some(chunk)) = field.chunk().await {
                    file_blob.extend_from_slice(&chunk);
                }

                let insert_id = match db::structs::Attachment::add(&None, &None, &None, &file_name, &file_blob, &db::enums::FileType::Illustration, &Some(mime_type), &auth.backend.pool).await {
                    Ok(insert_id) => insert_id,
                    Err(e) => {
                        tracing::error!(error = ?e);
                        return Err(AppError::InternalError("internal error".to_string()));
                    }
                };

                match db::structs::AttachmentWithoutBlob::get(&db::enums::AttachmentIdentifier::Id(insert_id.clone()), &auth.backend.pool).await {
                    Ok(Some(attachment_result)) => attachment = attachment_result,
                    Ok(None) => {
                        tracing::error!("file upload with insert id {} but could not fetch it from db", insert_id);
                        return Err(AppError::InternalError("internal error".to_string()));
                    },
                    Err(e) => {
                        tracing::error!(error = ?e, "failed to fetch upload file from db");
                        return Err(AppError::InternalError("internal error".to_string()));
                    }
                }
            }

            Ok(ApiResult { result: ResultStatus::Success, details: attachment })
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[server(input=MultipartFormData, name=AdminUploadAvatar, prefix="/api/admin/avatar", endpoint="upload")]
#[instrument(skip(file))]
pub async fn upload_avatar(file: MultipartData) -> Result<ApiResult<AttachmentWithoutBlob>, AppError> {
    cfg_if! {
        if #[cfg(feature = "ssr")] {
            let auth = use_context::<AuthSession>().unwrap();
            let user = auth.user.unwrap_or_default();
            let response = expect_context::<ResponseOptions>();
            let db_user = match DbUser::get(&UserIdentifier::Id(user.id.clone()), &auth.backend.pool).await {
                Ok(Some(user)) => user,
                Ok(None) => {
                    return Err(AppError::InternalError("internal error".to_string()));
                }
                Err(e) => {
                    tracing::error!(error = ?e);
                    return Err(AppError::InternalError("internal error".to_string()));
                }
            };

            if db_user.role != UserRole::Admin {
                response.set_status(StatusCode::FORBIDDEN);
                return Err(AppError::Forbidden);
            }

            let mut attachment = AttachmentWithoutBlob::default();

            let mut data = file.into_inner().unwrap();
            while let Ok(Some(mut field)) = data.next_field().await {
                let file_name = match field.file_name() {
                    Some(n) => n.to_string(),
                    None => continue,
                };

                let mime_type = field.content_type().map(|ct| ct.to_string()).unwrap_or_default();

                let mut file_blob = Vec::<u8>::new();
                while let Ok(Some(chunk)) = field.chunk().await {
                    file_blob.extend_from_slice(&chunk);
                }

                let insert_id = match db::structs::Attachment::add(&None, &None, &None, &file_name, &file_blob, &db::enums::FileType::Avatar, &Some(mime_type), &auth.backend.pool).await {
                    Ok(insert_id) => insert_id,
                    Err(e) => {
                        tracing::error!(error = ?e);
                        return Err(AppError::InternalError("internal error".to_string()));
                    }
                };

                match db::structs::AttachmentWithoutBlob::get(&db::enums::AttachmentIdentifier::Id(insert_id.clone()), &auth.backend.pool).await {
                    Ok(Some(attachment_result)) => attachment = attachment_result,
                    Ok(None) => tracing::error!("file upload with insert id {} but could not fetch it from db", insert_id),
                    Err(e) => {
                        tracing::error!(error = ?e, "failed to fetch upload file from db");
                        return Err(AppError::InternalError("internal error".to_string()));
                    }
                }
            }

            Ok(ApiResult { result: ResultStatus::Success, details: attachment })
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
            let response = expect_context::<ResponseOptions>();
            let db_user = match DbUser::get(&UserIdentifier::Id(user.id.clone()), &auth.backend.pool).await {
                Ok(Some(user)) => user,
                Ok(None) => {
                    return Err(AppError::InternalError("internal error".to_string()));
                }
                Err(e) => {
                    tracing::error!(error = ?e);
                    return Err(AppError::InternalError("internal error".to_string()));
                }
            };

            if db_user.role != UserRole::Admin {
                response.set_status(StatusCode::FORBIDDEN);
                return Err(AppError::Forbidden);
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
            let response = expect_context::<ResponseOptions>();
            let db_user = match DbUser::get(&UserIdentifier::Id(user.id.clone()), &auth.backend.pool).await {
                Ok(Some(user)) => user,
                Ok(None) => {
                    return Err(AppError::InternalError("internal error".to_string()));
                }
                Err(e) => {
                    tracing::error!(error = ?e);
                    return Err(AppError::InternalError("internal error".to_string()));
                }
            };

            if db_user.role != UserRole::Admin {
                response.set_status(StatusCode::FORBIDDEN);
                return Err(AppError::Forbidden);
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

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum UserAction {
    Create {
        username: String,  
        email: String, 
        password: String, 
        confirm_password: String,
        role: UserRole,
        avatar: Option<AttachmentWithoutBlob>,
        group: String
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
        avatar: Option<AttachmentWithoutBlob>,
        group: String
    },
    EditPassword {
        id: String,
        password: String, 
        confirm_password: String
    }
}

#[server(name=AdminUserApi, prefix="/api/admin", endpoint="user")]
#[instrument]
pub async fn user(action: UserAction) -> Result<ApiResult<Option<String>>, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let auth = use_context::<AuthSession>().unwrap();
            let user = auth.user.unwrap_or_default();
            let response = expect_context::<ResponseOptions>();
            let db_user = match DbUser::get(&UserIdentifier::Id(user.id.clone()), &auth.backend.pool).await {
                Ok(Some(user)) => user,
                Ok(None) => {
                    return Err(AppError::InternalError("internal error".to_string()));
                }
                Err(e) => {
                    tracing::error!(error = ?e);
                    return Err(AppError::InternalError("internal error".to_string()));
                }
            };

            if db_user.role != UserRole::Admin {
                response.set_status(StatusCode::FORBIDDEN);
                return Err(AppError::Forbidden);
            }

            match action {
                UserAction::Create { username, email, password, confirm_password, role, avatar, group } => {
                    if password != confirm_password {
                        return Err(AppError::BadRequest("password and confirm password must match".to_string()));
                    }

                    if username.is_empty() {
                        return Err(AppError::BadRequest("username must not be empty".to_string()));
                    }

                    let hashed_pw = hash_string(password.clone())?;
                    let new_user = DbUser { 
                        id: "".to_string(), 
                        username: username.clone(), 
                        email: email.clone(), 
                        pw_hash: hashed_pw, 
                        created_at: chrono::Local::now(), 
                        last_active_at: chrono::Local::now(), 
                        role,
                        points: 0,
                        group
                    };
                    let mut tx = auth.backend.pool.begin().await?;
                    let new_user_id = match DbUser::add(&new_user, &mut *tx).await {
                        Ok(result) => result,
                        Err(e) => {
                            tracing::error!(error = ?e);
                            tx.rollback().await?;
                            return Ok(ApiResult { result: ResultStatus::Fail, details: Some("internal error".to_string()) });
                        }
                    };

                    if avatar.is_some() {
                        match AttachmentWithoutBlob::edit_avatar(&avatar.unwrap_or_default().id, &new_user_id, &mut *tx).await {
                            Ok(_) => {
                                tx.commit().await?;
                                Ok(ApiResult { result: ResultStatus::Success, details: Some("created user".to_string()) })
                            },
                            Err(e) => {
                                tracing::error!(error = ?e);
                                tx.rollback().await?;
                                Ok(ApiResult { result: ResultStatus::Fail, details: Some("internal error".to_string()) })
                            }
                        }
                    } else {
                        tx.commit().await?;
                        Ok(ApiResult { result: ResultStatus::Success, details: Some("created user".to_string()) })
                    }
                }
                UserAction::Delete { id } => {
                    let user = match DbUser::get(&UserIdentifier::Id(id.clone()), &auth.backend.pool).await {
                        Ok(Some(user)) => {
                            if user.role == UserRole::Admin {
                                return Err(AppError::InternalError("cannot delete admin users".to_string()));
                            }
                            user
                        },
                        Ok(None) => {
                            return Err(AppError::InternalError("internal error".to_string()));
                        }
                        Err(e) => {
                            tracing::error!(error = ?e);
                            return Err(AppError::InternalError("internal error".to_string()));
                        }
                    };

                    match DbUser::delete(&user.id, &auth.backend.pool).await {
                        Ok(_) => Ok(ApiResult { result: ResultStatus::Success, details: Some("deleted event".to_string()) }),
                        Err(e) => {
                            tracing::error!(error = ?e);
                            Ok(ApiResult { result: ResultStatus::Fail, details: Some("internal error".to_string()) })
                        }
                    }
                }
                UserAction::Edit { id, username, email, password, confirm_password, points, role, avatar, group } => {
                    let user = match DbUser::get(&UserIdentifier::Id(id.clone()), &auth.backend.pool).await {
                        Ok(Some(user)) => {
                            if user.role == UserRole::Admin {
                                return Err(AppError::InternalError("cannot edit admin users".to_string()));
                            }
                            user
                        },
                        Ok(None) => {
                            return Err(AppError::InternalError("internal error".to_string()));
                        }
                        Err(e) => {
                            tracing::error!(error = ?e);
                            return Err(AppError::InternalError("internal error".to_string()));
                        }
                    };

                    if password != confirm_password {
                        return Err(AppError::BadRequest("password and confirm password must match".to_string()));
                    }

                    if username.is_empty() {
                        return Err(AppError::BadRequest("username must not be empty".to_string()));
                    }

                    let mut tx = auth.backend.pool.begin().await?;
                    match DbUser::edit_username(&user.id, &username, &mut *tx).await {
                        Ok(_) => {},
                        Err(e) => {
                            tracing::error!(error = ?e);
                            tx.rollback().await?;
                            return Ok(ApiResult { result: ResultStatus::Fail, details: Some("internal error".to_string()) });
                        }
                    }

                    match DbUser::edit_email(&user.id, &email, &mut *tx).await {
                        Ok(_) => {},
                        Err(e) => {
                            tracing::error!(error = ?e);
                            tx.rollback().await?;
                            return Ok(ApiResult { result: ResultStatus::Fail, details: Some("internal error".to_string()) });
                        }
                    }

                    match DbUser::edit_points(&user.id, &points, &mut *tx).await {
                        Ok(_) => {},
                        Err(e) => {
                            tracing::error!(error = ?e);
                            tx.rollback().await?;
                            return Ok(ApiResult { result: ResultStatus::Fail, details: Some("internal error".to_string()) });
                        }
                    }

                    match DbUser::edit_role(&id, &role, &mut *tx).await {
                        Ok(_) => {},
                        Err(e) => {
                            tracing::error!(error = ?e);
                            tx.rollback().await?;
                            return Ok(ApiResult { result: ResultStatus::Fail, details: Some("internal error".to_string()) });
                        }
                    }

                    match DbUser::edit_group(&id, &group, &mut *tx).await {
                        Ok(_) => {},
                        Err(e) => {
                            tracing::error!(error = ?e);
                            tx.rollback().await?;
                            return Ok(ApiResult { result: ResultStatus::Fail, details: Some("internal error".to_string()) });
                        }
                    }

                    if avatar.is_some() {
                        match AttachmentWithoutBlob::edit_avatar(&avatar.unwrap_or_default().id, &id, &mut *tx).await {
                            Ok(_) => {
                                tx.commit().await?;
                                Ok(ApiResult { result: ResultStatus::Success, details: Some("edited user".to_string()) })
                            },
                            Err(e) => {
                                tracing::error!(error = ?e);
                                tx.rollback().await?;
                                Ok(ApiResult { result: ResultStatus::Fail, details: Some("internal error".to_string()) })
                            }
                        }
                    } else {
                        tx.commit().await?;
                        Ok(ApiResult { result: ResultStatus::Success, details: Some("edited user".to_string()) })
                    }
                }
                UserAction::EditPassword { id, password, confirm_password } => {
                    let user = match DbUser::get(&UserIdentifier::Id(id.clone()), &auth.backend.pool).await {
                        Ok(Some(user)) => {
                            if user.role == UserRole::Admin {
                                return Err(AppError::InternalError("cannot edit password on admin users".to_string()));
                            }
                            user
                        },
                        Ok(None) => {
                            return Err(AppError::InternalError("internal error".to_string()));
                        }
                        Err(e) => {
                            tracing::error!(error = ?e);
                            return Err(AppError::InternalError("internal error".to_string()));
                        }
                    };

                    if password != confirm_password {
                        return Err(AppError::BadRequest("password and confirm password must match".to_string()));
                    }

                    let hashed_pw = hash_string(password.clone())?;

                    match DbUser::edit_password(&user.id, &hashed_pw, &auth.backend.pool).await {
                        Ok(_) => Ok(ApiResult { result: ResultStatus::Success, details: Some("edited user".to_string()) }),
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

#[server(name=AdminDeleteFile, prefix="/api/admin/file", endpoint="delete")]
#[instrument]
pub async fn delete_file(id: String) -> Result<ApiResult<Option<String>>, AppError> {
    cfg_if! {
        if #[cfg(feature = "ssr")] {
            let auth = use_context::<AuthSession>().unwrap();
            let user = auth.user.unwrap_or_default();
            let response = expect_context::<ResponseOptions>();
            let db_user = match DbUser::get(&UserIdentifier::Id(user.id.clone()), &auth.backend.pool).await {
                Ok(Some(user)) => user,
                Ok(None) => {
                    return Err(AppError::InternalError("internal error".to_string()));
                }
                Err(e) => {
                    tracing::error!(error = ?e);
                    return Err(AppError::InternalError("internal error".to_string()));
                }
            };

            if db_user.role != UserRole::Admin {
                response.set_status(StatusCode::FORBIDDEN);
                return Err(AppError::Forbidden);
            }

            match db::structs::Attachment::delete(&db::enums::AttachmentIdentifier::Id(id.clone()), &auth.backend.pool).await {
                Ok(_) => Ok(ApiResult { result: ResultStatus::Success, details: Some("deleted file".to_string()) }),
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

#[server(name=GetDbUser, prefix="/api/admin/user", endpoint="info")]
#[instrument]
pub async fn get_db_user(username: Option<String>) -> Result<Option<DbUser>, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let auth = use_context::<AuthSession>().unwrap();
            let user = auth.user.unwrap_or_default();
            let response = expect_context::<ResponseOptions>();
            let db_user = match DbUser::get(&UserIdentifier::Id(user.id.clone()), &auth.backend.pool).await {
                Ok(Some(user)) => user,
                Ok(None) => {
                    return Err(AppError::InternalError("internal error".to_string()));
                }
                Err(e) => {
                    tracing::error!(error = ?e);
                    return Err(AppError::InternalError("internal error".to_string()));
                }
            };

            if db_user.role != UserRole::Admin {
                response.set_status(StatusCode::FORBIDDEN);
                return Err(AppError::Forbidden);
            }

            if username.is_some() {
                match DbUser::get(&UserIdentifier::Username(username.unwrap_or_default().clone()), &auth.backend.pool).await {
                    Ok(user) => Ok(user),
                    Err(e) => {
                        tracing::error!(error = ?e);
                        Err(AppError::InternalError("internal error".to_string()))
                    }
                }    
            } else {
                match DbUser::get(&UserIdentifier::Id(user.id.clone()), &auth.backend.pool).await {
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

#[server(name=TestLdap, prefix="/api/admin", endpoint="test_ldap")]
#[instrument]
pub async fn test_ldap(args: LdapArgs) -> Result<ApiResult<Option<String>>, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let auth = use_context::<AuthSession>().unwrap();
            let user = auth.user.unwrap_or_default();
            let response = expect_context::<ResponseOptions>();
            let db_user = match DbUser::get(&UserIdentifier::Id(user.id.clone()), &auth.backend.pool).await {
                Ok(Some(user)) => user,
                Ok(None) => {
                    return Err(AppError::InternalError("internal error".to_string()));
                }
                Err(e) => {
                    tracing::error!(error = ?e);
                    return Err(AppError::InternalError("internal error".to_string()));
                }
            };

            if db_user.role != UserRole::Admin {
                response.set_status(StatusCode::FORBIDDEN);
                return Err(AppError::Forbidden);
            }
            
            let mut ldap = match LdapConn::new(args.url.as_str()) {
                Ok(conn) => conn,
                Err(e) => return Ok(ApiResult { result: ResultStatus::Fail, details: Some(e.to_string()) })
            };

            match ldap.simple_bind(args.bind_dn.as_str(), args.bind_pw.as_str()) {
                Ok(res) => {
                    match res.success() {
                        Ok(res) => Ok(ApiResult { result: ResultStatus::Success, details: Some(res.to_string()) }),
                        Err(e) => Ok(ApiResult { result: ResultStatus::Fail, details: Some(e.to_string()) })
                    }
                },
                Err(e) => Ok(ApiResult { result: ResultStatus::Fail, details: Some(e.to_string()) })
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[server(name=GetLdap, prefix="/api/admin/ldap", endpoint="get")]
#[instrument]
pub async fn get_ldap() -> Result<Option<LdapArgs>, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let auth = use_context::<AuthSession>().unwrap();
            let user = auth.user.unwrap_or_default();
            let response = expect_context::<ResponseOptions>();
            let db_user = match DbUser::get(&UserIdentifier::Id(user.id.clone()), &auth.backend.pool).await {
                Ok(Some(user)) => user,
                Ok(None) => {
                    return Err(AppError::InternalError("internal error".to_string()));
                }
                Err(e) => {
                    tracing::error!(error = ?e);
                    return Err(AppError::InternalError("internal error".to_string()));
                }
            };

            if db_user.role != UserRole::Admin {
                response.set_status(StatusCode::FORBIDDEN);
                return Err(AppError::Forbidden);
            }
            
            match LdapArgs::get(&auth.backend.pool).await {
                Ok(args) => Ok(args),
                Err(e) => Err(e.into())
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[server(name=UpdateLdap, prefix="/api/admin", endpoint="ldap")]
#[instrument]
pub async fn update_ldap(args: LdapArgs) -> Result<ApiResult<Option<String>>, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let auth = use_context::<AuthSession>().unwrap();
            let user = auth.user.unwrap_or_default();
            let response = expect_context::<ResponseOptions>();
            let db_user = match DbUser::get(&UserIdentifier::Id(user.id.clone()), &auth.backend.pool).await {
                Ok(Some(user)) => user,
                Ok(None) => {
                    return Err(AppError::InternalError("internal error".to_string()));
                }
                Err(e) => {
                    tracing::error!(error = ?e);
                    return Err(AppError::InternalError("internal error".to_string()));
                }
            };

            if db_user.role != UserRole::Admin {
                response.set_status(StatusCode::FORBIDDEN);
                return Err(AppError::Forbidden);
            }
            
            let mut ldap = match LdapConn::new(args.url.as_str()) {
                Ok(conn) => conn,
                Err(e) => return Ok(ApiResult { result: ResultStatus::Fail, details: Some(e.to_string()) })
            };

            match ldap.simple_bind(args.bind_dn.as_str(), args.bind_pw.as_str()) {
                Ok(res) => {
                    match res.success() {
                        Ok(_) => {},
                        Err(e) => return Ok(ApiResult { result: ResultStatus::Fail, details: Some(e.to_string()) })
                    }
                },
                Err(e) => return Ok(ApiResult { result: ResultStatus::Fail, details: Some(e.to_string()) })
            }

            // bind_pw should be hashed, but how to connect with a hashed password?
            match LdapArgs::update(&args.url, &args.bind_dn, &args.bind_pw, &args.base_dn, &auth.backend.pool).await {
                Ok(_) => Ok(ApiResult { result: ResultStatus::Success, details: Some("successfully updated LDAP configuration".to_string()) }),
                Err(e) => {
                    Ok(ApiResult { result: ResultStatus::Fail, details: Some(format!("bind succeeded but failed to update DB row: {e}")) })
                }
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[server(name=EnableLdap, prefix="/api/admin/ldap", endpoint="enable")]
#[instrument]
pub async fn enable_ldap() -> Result<ApiResult<Option<String>>, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let auth = use_context::<AuthSession>().unwrap();
            let user = auth.user.unwrap_or_default();
            let response = expect_context::<ResponseOptions>();
            let db_user = match DbUser::get(&UserIdentifier::Id(user.id.clone()), &auth.backend.pool).await {
                Ok(Some(user)) => user,
                Ok(None) => {
                    return Err(AppError::InternalError("internal error".to_string()));
                }
                Err(e) => {
                    tracing::error!(error = ?e);
                    return Err(AppError::InternalError("internal error".to_string()));
                }
            };

            if db_user.role != UserRole::Admin {
                response.set_status(StatusCode::FORBIDDEN);
                return Err(AppError::Forbidden);
            }

            match LdapArgs::enable(&auth.backend.pool).await {
                Ok(_) => Ok(ApiResult { result: ResultStatus::Success, details: Some("successfully enabled LDAP authentication".to_string()) }),
                Err(e) => {
                    Ok(ApiResult { result: ResultStatus::Fail, details: Some(format!("Failed to update DB row: {e}")) })
                }
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[server(name=DisableLdap, prefix="/api/admin/ldap", endpoint="disable")]
#[instrument]
pub async fn disable_ldap() -> Result<ApiResult<Option<String>>, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let auth = use_context::<AuthSession>().unwrap();
            let user = auth.user.unwrap_or_default();
            let response = expect_context::<ResponseOptions>();
            let db_user = match DbUser::get(&UserIdentifier::Id(user.id.clone()), &auth.backend.pool).await {
                Ok(Some(user)) => user,
                Ok(None) => {
                    return Err(AppError::InternalError("internal error".to_string()));
                }
                Err(e) => {
                    tracing::error!(error = ?e);
                    return Err(AppError::InternalError("internal error".to_string()));
                }
            };

            if db_user.role != UserRole::Admin {
                response.set_status(StatusCode::FORBIDDEN);
                return Err(AppError::Forbidden);
            }
            
            match LdapArgs::disable(&auth.backend.pool).await {
                Ok(_) => Ok(ApiResult { result: ResultStatus::Success, details: Some("successfully disabled LDAP authentication".to_string()) }),
                Err(e) => {
                    Ok(ApiResult { result: ResultStatus::Fail, details: Some(format!("failed to update DB row: {e}")) })
                }
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}
