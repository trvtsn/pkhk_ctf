#[cfg(feature = "ssr")]
use crate::server::{AuthSession, hash_string, build_and_broadcast, is_host_reachable};
use crate::{error_template::AppError, server::{AdminEventPayloadKind, UserRole, db::{self, enums::{AttachmentIdentifier, FileType, UserIdentifier}, structs::{Attachment, AttachmentWithoutBlob, DbHint, DbUser, Event, EventWithAttachments, HintsUsed, LdapArgs, ProxmoxArgs, UserAvatar}}, enums::ResultStatus, proxmox::ProxmoxVMTemplate, structs::{ApiResult, User}}, utils::get_context};
use cfg_if::cfg_if;
use chrono::{DateTime, Local};
#[cfg(feature = "ssr")]
use ldap3::{LdapConnAsync, LdapConnSettings};
use leptos::{prelude::*, server_fn::codec::{MultipartData, MultipartFormData}};
#[cfg(feature = "ssr")]
use leptos_axum::ResponseOptions;
#[cfg(feature = "ssr")]
use http::StatusCode;
use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use sqlx::MySqlPool;
use std::collections::{HashMap, HashSet};
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

#[server(name=AdminChallengeApi, prefix="/api/admin", endpoint="challenge")]
#[instrument]
pub async fn challenge(action: ChallengeAction) -> Result<ApiResult<String>, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let (_, pool) = authenticated_check().await?;

            match action {
                ChallengeAction::Create { event_id, name, description, category, difficulty, points, flag, visible_to_groups, attachments, illustration, vm_ids, hints } => {
                    let flag_hash = hash_string(&flag).await?;
                    let mut tx = pool.begin().await?;
                    let new_challenge_id = match db::structs::Challenge::add(&event_id, &name, &description, &category, &difficulty, &points, &flag_hash, &visible_to_groups, &vm_ids, &mut *tx).await {
                        Ok(result) => result,
                        Err(e) => {
                            tx.rollback().await?;
                            tracing::error!(error = ?e);
                            return Err(AppError::InternalError(e.to_string()))
                        }
                    };

                    if let Some(attachments) = attachments {
                        for attachment in attachments {
                            if let Err(e) = db::structs::Attachment::edit_challenge(&attachment.id, &new_challenge_id, &mut *tx).await {
                                tx.rollback().await?;
                                tracing::error!(error = ?e);
                                return Err(AppError::InternalError(e.to_string()));
                            }
                        }
                    }

                    if let Some(hints) = hints {
                        for hint in hints {
                            if !hint.hint.is_empty() {
                                if let Err(e) = DbHint::add(&hint.hint, &new_challenge_id, &hint.points_penalty, &mut *tx).await {
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
                                        tokio::spawn(async {
                                            _ = build_and_broadcast(AdminEventPayloadKind::NewChallengeCreated).await;
                                        });
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
                            tokio::spawn(async {
                                _ = build_and_broadcast(AdminEventPayloadKind::NewChallengeCreated).await;
                            });
                            Ok(ApiResult { result: ResultStatus::Success, details: "created challenge".to_string() })
                        }
                    }
                }
                ChallengeAction::Delete { id } => {
                    let mut tx = pool.begin().await?;

                    if let Err(e) = db::structs::Submission::delete(&db::enums::SubmissionIdentifier::ChallengeId(id.clone()), &mut *tx).await {
                        tx.rollback().await?;
                        tracing::error!(error = ?e);
                        return Ok(ApiResult { result: ResultStatus::Fail, details: e.to_string() });
                    }

                    if let Err(e) = db::structs::Attachment::delete(&db::enums::AttachmentIdentifier::ChallengeId(id.clone()), &mut *tx).await {
                        tx.rollback().await?;
                        tracing::error!(error = ?e);
                        return Ok(ApiResult { result: ResultStatus::Fail, details: e.to_string() });
                    }

                    if let Err(e) = HintsUsed::delete_all_from_challenge(&id, &mut *tx).await {
                        tx.rollback().await?;
                        tracing::error!(error = ?e);
                        return Ok(ApiResult { result: ResultStatus::Fail, details: e.to_string() });
                    }

                    if let Err(e) = DbHint::delete_all_from_challenge(&id, &mut *tx).await {
                        tx.rollback().await?;
                        tracing::error!(error = ?e);
                        return Ok(ApiResult { result: ResultStatus::Fail, details: e.to_string() });
                    }

                    match db::structs::Challenge::delete(&id, &mut *tx).await {
                        Ok(_) => {
                            tx.commit().await?;
                            tokio::spawn(async {
                                _ = build_and_broadcast(AdminEventPayloadKind::ChallengeDeleted).await;
                            });
                            Ok(ApiResult { result: ResultStatus::Success, details: "deleted challenge".to_string() })
                        },
                        Err(e) => {
                            tx.rollback().await?;
                            tracing::error!(error = ?e);
                            return Ok(ApiResult { result: ResultStatus::Fail, details: e.to_string() });
                        }
                    }
                }
                ChallengeAction::Edit { id, event_id, name, description, category, difficulty, points, flag, visible_to_groups, attachments, illustration, vm_ids, hints } => {
                    let flag_hash = if flag.is_empty() { String::new() } else { hash_string(&flag).await? };

                    let mut tx = pool.begin().await?;
                    if let Err(e) = db::structs::Challenge::edit(&id, &event_id, &name, &description, &category, &difficulty, &points, &flag_hash, &visible_to_groups, &vm_ids, &mut *tx).await {
                        tx.rollback().await?;
                        tracing::error!(error = ?e);
                        return Ok(ApiResult { result: ResultStatus::Fail, details: e.to_string() });
                    }

                    let all_challenge_attachment_ids = match AttachmentWithoutBlob::get_all(&Some(db::enums::AttachmentIdentifier::ChallengeId(id.clone())), &mut *tx).await {
                        Ok(all_attachments) => all_attachments.into_iter().map(|a| a.id.clone()).collect::<Vec<String>>(),
                        Err(e) => {
                            tx.rollback().await?;
                            tracing::error!(error = ?e);
                            return Err(AppError::InternalError(e.to_string()));
                        }
                    };
                    
                    if let Some(attachments) = &attachments {
                        for attachment in attachments.iter() {
                            if let Err(e) = db::structs::Attachment::edit_challenge(&attachment.id, &id, &mut *tx).await {
                                tx.rollback().await?;
                                tracing::error!(error = ?e);
                                return Err(AppError::InternalError(e.to_string()));
                            }
                        }
                    }

                    let new_attachment_ids = attachments.unwrap_or_default().into_iter().map(|h| h.id).collect::<HashSet<String>>();
                    for existing_attachment_id in all_challenge_attachment_ids {
                        if !new_attachment_ids.contains(&existing_attachment_id) {
                            if let Err(e) = AttachmentWithoutBlob::delete(&AttachmentIdentifier::Id(existing_attachment_id), &mut *tx).await {
                                tx.rollback().await?;
                                tracing::error!(error = ?e);
                                return Err(AppError::InternalError(e.to_string()));
                            }
                        }
                    }

                    if let Some(hints) = hints {
                        let all_challenge_hint_ids = match DbHint::get_all_from_challenge(&id, &mut *tx).await {
                            Ok(all_hints) => all_hints.into_iter().map(|h| h.id).collect::<HashSet<String>>(),
                            Err(e) => {
                                tx.rollback().await?;
                                tracing::error!(error = ?e);
                                return Err(AppError::InternalError(e.to_string()));
                            }
                        };
                        
                        let new_hints_ids = hints.iter().map(|h| h.id.clone()).collect::<HashSet<String>>();
                        for hint in hints {
                            if !hint.hint.is_empty() && !all_challenge_hint_ids.contains(&hint.id) {
                                if let Err(e) = DbHint::add(&hint.hint, &id, &hint.points_penalty, &mut *tx).await {
                                    tx.rollback().await?;
                                    tracing::error!(error = ?e);
                                    return Err(AppError::InternalError(e.to_string()));
                                }
                            } else if !hint.hint.is_empty() && all_challenge_hint_ids.contains(&hint.id) {
                                if let Err(e) = DbHint::edit(&hint.id, &hint.hint, &id, &hint.points_penalty, &mut *tx).await {
                                    tx.rollback().await?;
                                    tracing::error!(error = ?e);
                                    return Err(AppError::InternalError(e.to_string()));
                                }
                            } else if hint.hint.is_empty() && hint.points_penalty == 0 && all_challenge_hint_ids.contains(&hint.id) {
                                if let Err(e) = DbHint::delete(&hint.id, &mut *tx).await {
                                    tx.rollback().await?;
                                    tracing::error!(error = ?e);
                                    return Err(AppError::InternalError(e.to_string()));
                                }
                            }
                        }

                        for existing_hint_id in all_challenge_hint_ids {
                            if !new_hints_ids.contains(&existing_hint_id) {
                                if let Err(e) = DbHint::delete(&existing_hint_id, &mut *tx).await {
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
                                    tokio::spawn(async {
                                        _ = build_and_broadcast(AdminEventPayloadKind::ChallengeEdited).await;
                                    });
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
                            let existing_illustration_id = match AttachmentWithoutBlob::get_illustration_id(
                                &db::enums::AttachmentIdentifier::ChallengeId(id), &mut *tx
                            ).await {
                                Ok(id) => id,
                                Err(e) => {
                                    tx.rollback().await?;
                                    tracing::error!(error = ?e);
                                    return Err(AppError::InternalError(e.to_string()));
                                }
                            };
                            
                            if let Some(id) = existing_illustration_id {
                                match db::structs::Attachment::delete(&db::enums::AttachmentIdentifier::ChallengeId(id), &mut *tx).await {
                                    Ok(_) => {
                                        tx.commit().await?;
                                        tokio::spawn(async {
                                            _ = build_and_broadcast(AdminEventPayloadKind::ChallengeEdited).await;
                                        });
                                        return Ok(ApiResult { result: ResultStatus::Success, details: "edited challenge".to_string() });
                                    },
                                    Err(e) => {
                                        tx.rollback().await?;
                                        tracing::error!(error = ?e);
                                        return Err(AppError::InternalError(e.to_string()));
                                    }
                                }
                            }

                            tx.commit().await?;
                            tokio::spawn(async {
                                _ = build_and_broadcast(AdminEventPayloadKind::ChallengeEdited).await;
                            });
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
pub async fn event(action: EventAction) -> Result<ApiResult<String>, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let (_, pool) = authenticated_check().await?;

            match action {
                EventAction::Create { name, description, start_at, end_at, visible_to_groups, attachments, illustration } => {
                    let mut tx = pool.begin().await?;
                    let new_event_id = match db::structs::Event::add(&name, &description, &start_at, &end_at, &visible_to_groups, &mut *tx).await {
                        Ok(result) => result,
                        Err(e) => {
                            tracing::error!(error = ?e);
                            tx.rollback().await?;
                            return Ok(ApiResult { result: ResultStatus::Fail, details: e.to_string() });
                        }
                    };

                    if let Some(attachments) = attachments {
                        for attachment in attachments {
                            if let Err(e) = db::structs::Attachment::edit_event(&attachment.id, &new_event_id, &mut *tx).await {
                                tx.rollback().await?;
                                tracing::error!(error = ?e);
                                return Err(AppError::InternalError(e.to_string()));
                            }
                        }
                    }

                    match illustration {
                        Some(illustration) => {
                                match db::structs::Attachment::edit_illustration(&illustration.id, &db::enums::AttachmentIdentifier::EventId(new_event_id), &mut *tx).await {
                                    Ok(_) => {
                                        tx.commit().await?;
                                        tokio::spawn(async {
                                            _ = build_and_broadcast(AdminEventPayloadKind::NewEventCreated).await;
                                        });
                                        Ok(ApiResult { result: ResultStatus::Success, details: "created event".to_string() })
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
                            tokio::spawn(async {
                                _ = build_and_broadcast(AdminEventPayloadKind::NewEventCreated).await;
                            });
                            Ok(ApiResult { result: ResultStatus::Success, details: "created event".to_string() })
                        }
                    }
                }
                EventAction::Delete { id } => {
                    match db::structs::Event::delete(&id, &pool).await {
                        Ok(_) => {
                            tokio::spawn(async {
                                _ = build_and_broadcast(AdminEventPayloadKind::EventDeleted).await;
                            });
                            Ok(ApiResult { result: ResultStatus::Success, details: "deleted event".to_string() })
                        },
                        Err(e) => {
                            tracing::error!(error = ?e);
                            Ok(ApiResult { result: ResultStatus::Fail, details: e.to_string() })
                        }
                    }
                }
                EventAction::Edit { id, name, description, start_at, end_at, visible_to_groups, attachments, illustration } => {
                    let mut tx = pool.begin().await?;
                    if let Err(e) = db::structs::Event::edit(&id, &name, &description, &start_at, &end_at, &visible_to_groups, &mut *tx).await {
                        tracing::error!(error = ?e);
                        tx.rollback().await?;
                        return Ok(ApiResult { result: ResultStatus::Fail, details: e.to_string() });
                    }

                    let all_event_attachment_ids = match AttachmentWithoutBlob::get_all(&Some(db::enums::AttachmentIdentifier::EventId(id.clone())), &mut *tx).await {
                        Ok(all_attachments) => all_attachments.into_iter().map(|a| a.id.clone()).collect::<Vec<String>>(),
                        Err(e) => {
                            tx.rollback().await?;
                            tracing::error!(error = ?e);
                            return Err(AppError::InternalError(e.to_string()));
                        }
                    };

                    if let Some(attachments) = &attachments {
                        for attachment in attachments {
                            if let Err(e) = db::structs::Attachment::edit_event(&attachment.id, &id, &mut *tx).await {
                                tx.rollback().await?;
                                tracing::error!(error = ?e);
                                return Err(AppError::InternalError(e.to_string()));
                            }
                        }
                    }

                    let new_attachment_ids = attachments.unwrap_or_default().into_iter().map(|h| h.id).collect::<HashSet<String>>();
                    for existing_attachment_id in all_event_attachment_ids {
                        if !new_attachment_ids.contains(&existing_attachment_id) {
                            if let Err(e) = AttachmentWithoutBlob::delete(&AttachmentIdentifier::Id(existing_attachment_id), &mut *tx).await {
                                tx.rollback().await?;
                                tracing::error!(error = ?e);
                                return Err(AppError::InternalError(e.to_string()));
                            }
                        }
                    }
                    match illustration {
                        Some(illustration) => {
                                match db::structs::Attachment::edit_illustration(&illustration.id, &db::enums::AttachmentIdentifier::EventId(id), &mut *tx).await {
                                    Ok(_) => {
                                        tx.commit().await?;
                                        tokio::spawn(async {
                                            _ = build_and_broadcast(AdminEventPayloadKind::NewEventCreated).await;
                                        });
                                        Ok(ApiResult { result: ResultStatus::Success, details: "edited event".to_string() })
                                    },
                                    Err(e) => {
                                        tx.rollback().await?;
                                        tracing::error!(error = ?e);
                                        return Err(AppError::InternalError(e.to_string()));
                                    }
                                }
                        }
                        None => {
                            let existing_illustration_id = match AttachmentWithoutBlob::get_illustration_id(
                                &db::enums::AttachmentIdentifier::EventId(id), &mut *tx
                            ).await {
                                Ok(id) => id,
                                Err(e) => {
                                    tx.rollback().await?;
                                    tracing::error!(error = ?e);
                                    return Err(AppError::InternalError(e.to_string()));
                                }
                            };
                            
                            if let Some(id) = existing_illustration_id {
                                match db::structs::Attachment::delete(&db::enums::AttachmentIdentifier::EventId(id), &mut *tx).await {
                                    Ok(_) => {
                                        tx.commit().await?;
                                        tokio::spawn(async {
                                            _ = build_and_broadcast(AdminEventPayloadKind::EventEdited).await;
                                        });
                                        return Ok(ApiResult { result: ResultStatus::Success, details: "edited challenge".to_string() });
                                    },
                                    Err(e) => {
                                        tx.rollback().await?;
                                        tracing::error!(error = ?e);
                                        return Err(AppError::InternalError(e.to_string()));
                                    }
                                }
                            }

                            tx.commit().await?;
                            tokio::spawn(async {
                                _ = build_and_broadcast(AdminEventPayloadKind::NewEventCreated).await;
                            });
                            Ok(ApiResult { result: ResultStatus::Success, details: "edited event".to_string() })
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
            let (_, pool) = authenticated_check().await?;

            match DbUser::get_all(&pool).await {
                Ok(users) => Ok(users),
                Err(e) => {
                    tracing::error!(error = ?e);
                    Err(AppError::InternalError(e.to_string()))
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
            let (_, pool) = authenticated_check().await?;

            match DbUser::get_all_groups(&pool).await {
                Ok(groups) => Ok(groups),
                Err(e) => {
                    tracing::error!(error = ?e);
                    Err(AppError::InternalError(e.to_string()))
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
            let (_, pool) = authenticated_check().await?;

            match db::structs::Event::get_all(&pool).await {
                Ok(events) => Ok(events),
                Err(e) => {
                    tracing::error!(error = ?e);
                    Err(AppError::InternalError(e.to_string()))
                }
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[server(name=AdminEventsGetAllWithAttachments, prefix="/api/admin", endpoint="ewa")]
#[instrument]
pub async fn get_all_events_with_attachments() -> Result<Vec<EventWithAttachments>, AppError> {
    cfg_if! {
        if #[cfg(feature = "ssr")] {
            let (_, pool) = authenticated_check().await?;

            let events = match db::structs::Event::get_all(&pool).await {
                Ok(events) => events,
                Err(e) => {
                    tracing::error!(error = ?e);
                    return Err(AppError::InternalError(e.to_string()));
                }
            };

            let all_attachments = db::structs::AttachmentWithoutBlob::get_all(&None, &pool).await?;

            let mut attachments_by_event = HashMap::<String, Vec<AttachmentWithoutBlob>>::new();
            let mut illustrations_by_event = HashMap::<String, AttachmentWithoutBlob>::new();

            for att in all_attachments {
                if let Some(event_id) = &att.event_id {
                    if att.file_type == FileType::Illustration {
                        illustrations_by_event.insert(event_id.clone(), att);
                    } else if att.file_type == FileType::Attachment {
                        attachments_by_event.entry(event_id.clone()).or_default().push(att);
                    }
                }
            }

            let ewa = events.into_iter().map(|event| {
                let attachments = attachments_by_event.remove(&event.id).unwrap_or_default();
                let illustration = illustrations_by_event.remove(&event.id);
                EventWithAttachments { event, attachments, illustration }
            }).collect();

            Ok(ewa)
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
            let (_, pool) = authenticated_check().await?;

            let mut attachments: Vec<AttachmentWithoutBlob> = Vec::new();

            let mut data = match files.into_inner() {
                Some(inner_data) => inner_data,
                None => {
                    return Err(AppError::InternalError("Failed to extract inner data from files".to_string()));
                }
            };
            let mut found_file = false;
            while let Ok(Some(mut field)) = data.next_field().await {
                let file_name = match field.file_name() {
                    Some(n) => {
                        found_file = true;
                        n.to_string()
                    },
                    None => continue,
                };

                let mime_type = field.content_type().map(|ct| ct.to_string()).unwrap_or_default();

                let mut file_blob = Vec::<u8>::new();
                while let Ok(Some(chunk)) = field.chunk().await {
                    file_blob.extend_from_slice(&chunk);
                }

                if file_blob.is_empty() {
                    return Err(AppError::BadRequest(format!(
                        "uploaded file \"{}\" is empty",
                        file_name
                    )));
                }

                let insert_id = match db::structs::Attachment::add(&None, &None, &None, &file_name, &file_blob, &db::enums::FileType::Attachment, &Some(mime_type), &pool).await {
                    Ok(insert_id) => insert_id,
                    Err(e) => {
                        tracing::error!(error = ?e);
                        return Err(AppError::InternalError(e.to_string()));
                    }
                };

                match db::structs::AttachmentWithoutBlob::get(&db::enums::AttachmentIdentifier::Id(insert_id.clone()), &pool).await {
                    Ok(Some(attachment)) => attachments.push(attachment),
                    Ok(None) => tracing::error!("file upload with insert id {} but could not fetch it from db", insert_id),
                    Err(e) => {
                        tracing::error!(error = ?e, "failed to fetch upload file from db");
                        return Err(AppError::InternalError(e.to_string()));
                    }
                }
            }

            if !found_file {
                Err(AppError::BadRequest("no files uploaded".to_string()))
            } else {
                Ok(ApiResult { result: ResultStatus::Success, details: attachments })
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[server(input=MultipartFormData, name=AdminUploadCertificate, prefix="/api/admin/certificate", endpoint="upload")]
#[instrument(skip(file))]
pub async fn upload_certificate(file: MultipartData) -> Result<ApiResult<AttachmentWithoutBlob>, AppError> {
    cfg_if! {
        if #[cfg(feature = "ssr")] {
            let (_, pool) = authenticated_check().await?;

            let existing_certificate = match AttachmentWithoutBlob::get_certificate(&pool).await {
                Ok(cert) => cert,
                Err(e) => return Err(e.into())
            };
            if let Some(existing_certificate) = existing_certificate {
                if let Err(e) = db::structs::Attachment::delete(&db::enums::AttachmentIdentifier::Id(existing_certificate.id), &pool).await {
                    tracing::error!(error = ?e);
                    return Err(AppError::InternalError(e.to_string()));
                }
            }

            let mut attachment = AttachmentWithoutBlob::default();

            let mut data = match file.into_inner() {
                Some(inner_data) => inner_data,
                None => {
                    return Err(AppError::InternalError("Failed to extract inner data from file".to_string()));
                }
            };
            let mut found_file = false;
            while let Ok(Some(mut field)) = data.next_field().await {
                let file_name = match field.file_name() {
                    Some(n) => {
                        found_file = true;
                        n.to_string()
                    },
                    None => continue,
                };

                let mime_type = field.content_type().map(|ct| ct.to_string()).unwrap_or_default();

                let mut file_blob = Vec::<u8>::new();
                while let Ok(Some(chunk)) = field.chunk().await {
                    file_blob.extend_from_slice(&chunk);
                }

                if file_blob.is_empty() {
                    return Err(AppError::BadRequest(format!(
                        "uploaded file \"{}\" is empty",
                        file_name
                    )));
                }

                let insert_id = match db::structs::Attachment::add(&None, &None, &None, &file_name, &file_blob, &db::enums::FileType::Certificate, &Some(mime_type), &pool).await {
                    Ok(id) => id,
                    Err(e) => {
                        tracing::error!(error = ?e);
                        return Err(AppError::InternalError(e.to_string()));
                    }
                };

                match db::structs::AttachmentWithoutBlob::get(&db::enums::AttachmentIdentifier::Id(insert_id.clone()), &pool).await {
                    Ok(Some(attachment_result)) => attachment = attachment_result,
                    Ok(None) => {
                        let error = format!("file uploaded with insert id {} but could not fetch it from db", insert_id);
                        tracing::error!(error);
                        return Err(AppError::InternalError(error));
                    },
                    Err(e) => {
                        tracing::error!(error = ?e, "failed to fetch uploaded file from db");
                        return Err(AppError::InternalError(e.to_string()));
                    }
                }
            }

            if !found_file {
                Err(AppError::BadRequest("no files uploaded".to_string()))
            } else {
                Ok(ApiResult { result: ResultStatus::Success, details: attachment })
            }
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
            let (_, pool) = authenticated_check().await?;

            let mut attachment = AttachmentWithoutBlob::default();

            let mut data = match file.into_inner() {
                Some(inner_data) => inner_data,
                None => {
                    return Err(AppError::InternalError("Failed to extract inner data from file".to_string()));
                }
            };
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

                let insert_id = match db::structs::Attachment::add(&None, &None, &None, &file_name, &file_blob, &db::enums::FileType::Illustration, &Some(mime_type), &pool).await {
                    Ok(insert_id) => insert_id,
                    Err(e) => {
                        tracing::error!(error = ?e);
                        return Err(AppError::InternalError(e.to_string()));
                    }
                };

                match db::structs::AttachmentWithoutBlob::get(&db::enums::AttachmentIdentifier::Id(insert_id.clone()), &pool).await {
                    Ok(Some(attachment_result)) => attachment = attachment_result,
                    Ok(None) => {
                        let error = format!("file uploaded with insert id {} but could not fetch it from db", insert_id);
                        tracing::error!(error);
                        return Err(AppError::InternalError(error));
                    },
                    Err(e) => {
                        tracing::error!(error = ?e, "failed to fetch upload file from db");
                        return Err(AppError::InternalError(e.to_string()));
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
pub async fn upload_avatar(file: MultipartData) -> Result<ApiResult<UserAvatar>, AppError> {
    cfg_if! {
        if #[cfg(feature = "ssr")] {
            let (_, pool) = authenticated_check().await?;

            let mut avatar = UserAvatar::default();

            let mut data = match file.into_inner() {
                Some(inner_data) => inner_data,
                None => {
                    return Err(AppError::InternalError("Failed to extract inner data from file".to_string()));
                }
            };
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

                let insert_id = match db::structs::Attachment::add(&None, &None, &None, &file_name, &file_blob, &db::enums::FileType::Avatar, &Some(mime_type), &pool).await {
                    Ok(insert_id) => insert_id,
                    Err(e) => {
                        tracing::error!(error = ?e);
                        return Err(AppError::InternalError(e.to_string()));
                    }
                };

                let result = UserAvatar {
                    attachment_id: insert_id,
                    user_id: None,
                    file_name
                };

                avatar = result;
            }

            Ok(ApiResult { result: ResultStatus::Success, details: avatar })
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
            let (_, pool) = authenticated_check().await?;

            match db::structs::Challenge::get_all_categories(&pool).await {
                Ok(categories) => Ok(categories),
                Err(e) => {
                    tracing::error!(error = ?e);
                    Err(AppError::InternalError(e.to_string()))
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
            let (_, pool) = authenticated_check().await?;

            match db::structs::AttachmentWithoutBlob::get_all(&None, &pool).await {
                Ok(attachments) => Ok(attachments),
                Err(e) => {
                    tracing::error!(error = ?e);
                    Err(AppError::InternalError(e.to_string()))
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

#[server(name=AdminUserApi, prefix="/api/admin", endpoint="user")]
#[instrument]
pub async fn user(action: UserAction) -> Result<ApiResult<String>, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let (_, pool) = authenticated_check().await?;

            match action {
                UserAction::Create { username, email, password, confirm_password, role, avatar, groups } => {
                    if password != confirm_password {
                        return Err(AppError::BadRequest("password and confirm password must match".to_string()));
                    }

                    if username.is_empty() {
                        return Err(AppError::BadRequest("username must not be empty".to_string()));
                    }

                    let hashed_pw = hash_string(&password).await?;
                    let new_user = DbUser {
                        id: "".to_string(), 
                        username, 
                        email, 
                        pw_hash: hashed_pw, 
                        created_at: chrono::Local::now(), 
                        last_active_at: chrono::Local::now(), 
                        role,
                        points: 0,
                        groups,
                        auth_type: "normal".to_string()
                    };
                    let mut tx = pool.begin().await?;
                    let new_user_id = match DbUser::add(&new_user, &mut *tx).await {
                        Ok(result) => result,
                        Err(e) => {
                            tracing::error!(error = ?e);
                            tx.rollback().await?;
                            return Ok(ApiResult { result: ResultStatus::Fail, details: e.to_string() });
                        }
                    };

                    if let Some(avatar) = avatar {
                        match AttachmentWithoutBlob::edit_avatar(&avatar.attachment_id, &new_user_id, &mut *tx).await {
                            Ok(_) => {
                                tx.commit().await?;
                                Ok(ApiResult { result: ResultStatus::Success, details: "created user".to_string() })
                            },
                            Err(e) => {
                                tracing::error!(error = ?e);
                                tx.rollback().await?;
                                Ok(ApiResult { result: ResultStatus::Fail, details: e.to_string() })
                            }
                        }
                    } else {
                        tx.commit().await?;
                        Ok(ApiResult { result: ResultStatus::Success, details: "created user".to_string() })
                    }
                }
                UserAction::Delete { id } => {
                    let user = match DbUser::get(&UserIdentifier::Id(id), &pool).await {
                        Ok(Some(user)) => {
                            if user.role == UserRole::Admin {
                                return Err(AppError::InternalError("cannot delete admin users".to_string()));
                            }
                            user
                        },
                        Ok(None) => {
                            return Err(AppError::InternalError("failed to fetch user from db".to_string()));
                        }
                        Err(e) => {
                            tracing::error!(error = ?e);
                            return Err(AppError::InternalError(e.to_string()));
                        }
                    };

                    match DbUser::delete(&user.id, &pool).await {
                        Ok(_) => Ok(ApiResult { result: ResultStatus::Success, details: "deleted event".to_string() }),
                        Err(e) => {
                            tracing::error!(error = ?e);
                            Ok(ApiResult { result: ResultStatus::Fail, details: e.to_string() })
                        }
                    }
                }
                UserAction::Edit { id, username, email, password, confirm_password, points, role, avatar, groups } => {
                    let user = match DbUser::get(&UserIdentifier::Id(id.clone()), &pool).await {
                        Ok(Some(user)) => {
                            if user.role == UserRole::Admin {
                                return Err(AppError::InternalError("cannot edit admin users".to_string()));
                            }
                            user
                        },
                        Ok(None) => {
                            return Err(AppError::InternalError("failed to fetch user from db".to_string()));
                        }
                        Err(e) => {
                            tracing::error!(error = ?e);
                            return Err(AppError::InternalError(e.to_string()));
                        }
                    };

                    if password != confirm_password {
                        return Err(AppError::BadRequest("password and confirm password must match".to_string()));
                    }

                    if username.is_empty() {
                        return Err(AppError::BadRequest("username must not be empty".to_string()));
                    }

                    let mut tx = pool.begin().await?;
                    if let Err(e) = DbUser::edit_username(&user.id, &username, &mut *tx).await {
                        tracing::error!(error = ?e);
                        tx.rollback().await?;
                        return Ok(ApiResult { result: ResultStatus::Fail, details: e.to_string() });
                    }

                    if let Err(e) = DbUser::edit_email(&user.id, &email, &mut *tx).await {
                        tracing::error!(error = ?e);
                        tx.rollback().await?;
                        return Ok(ApiResult { result: ResultStatus::Fail, details: e.to_string() });
                    }

                    if let Err(e) = DbUser::edit_points(&user.id, &points, &mut *tx).await {
                        tracing::error!(error = ?e);
                        tx.rollback().await?;
                        return Ok(ApiResult { result: ResultStatus::Fail, details: e.to_string() });
                    }

                    if let Err(e) = DbUser::edit_role(&id, &role, &mut *tx).await {
                        tracing::error!(error = ?e);
                        tx.rollback().await?;
                        return Ok(ApiResult { result: ResultStatus::Fail, details: e.to_string() });
                    }

                    if let Err(e) = DbUser::edit_groups(&id, &groups, &mut *tx).await {
                        tracing::error!(error = ?e);
                        tx.rollback().await?;
                        return Ok(ApiResult { result: ResultStatus::Fail, details: e.to_string() });
                    }

                    if let Some(avatar) = avatar {
                        if let Err(e) = AttachmentWithoutBlob::edit_avatar(&avatar.attachment_id, &id, &mut *tx).await {
                            tracing::error!(error = ?e);
                            tx.rollback().await?;
                            return Ok(ApiResult { result: ResultStatus::Fail, details: e.to_string() });
                        }
                    } else {
                        let existing_avatar = match DbUser::get_avatar(&UserIdentifier::Id(id), &mut *tx).await {
                            Ok(avatar) => avatar,
                            Err(e) => {
                                tracing::error!(error = ?e);
                                tx.rollback().await?;
                                return Ok(ApiResult { result: ResultStatus::Fail, details: e.to_string() });
                            }
                        };

                        if let Some(existing_avatar) = existing_avatar {
                            if let Err(e) = db::structs::Attachment::delete(&db::enums::AttachmentIdentifier::Id(existing_avatar.id), &mut *tx).await {
                                    tx.rollback().await?;
                                    tracing::error!(error = ?e);
                                    return Err(AppError::InternalError(e.to_string()));
                            }
                        }
                    }

                    tx.commit().await?;
                    return Ok(ApiResult { result: ResultStatus::Success, details: "edited user".to_string() });
                }
                UserAction::EditPassword { id, password, confirm_password } => {
                    let user = match DbUser::get(&UserIdentifier::Id(id), &pool).await {
                        Ok(Some(user)) => {
                            if user.role == UserRole::Admin {
                                return Err(AppError::InternalError("cannot edit password on admin users".to_string()));
                            }
                            user
                        },
                        Ok(None) => {
                            return Err(AppError::InternalError("failed to fetch user from db".to_string()));
                        }
                        Err(e) => {
                            tracing::error!(error = ?e);
                            return Err(AppError::InternalError(e.to_string()));
                        }
                    };

                    if password != confirm_password {
                        return Err(AppError::BadRequest("password and confirm password must match".to_string()));
                    }

                    let hashed_pw = hash_string(&password).await?;

                    match DbUser::edit_password(&user.id, &hashed_pw, &pool).await {
                        Ok(_) => Ok(ApiResult { result: ResultStatus::Success, details: "edited user".to_string() }),
                        Err(e) => {
                            tracing::error!(error = ?e);
                            Ok(ApiResult { result: ResultStatus::Fail, details: e.to_string() })
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
pub async fn delete_file(id: String) -> Result<ApiResult<String>, AppError> {
    cfg_if! {
        if #[cfg(feature = "ssr")] {
            let (_, pool) = authenticated_check().await?;

            match db::structs::Attachment::delete(&db::enums::AttachmentIdentifier::Id(id), &pool).await {
                Ok(_) => Ok(ApiResult { result: ResultStatus::Success, details: "deleted file".to_string() }),
                Err(e) => {
                    tracing::error!(error = ?e);
                    Err(AppError::InternalError(e.to_string()))
                }
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[server(name=AdminRenameFile, prefix="/api/admin/file", endpoint="rename")]
#[instrument]
pub async fn rename_file(id: String, file_name: String) -> Result<ApiResult<String>, AppError> {
    cfg_if! {
        if #[cfg(feature = "ssr")] {
            let (_, pool) = authenticated_check().await?;

            match db::structs::Attachment::edit_file_name(&id, &file_name, &pool).await {
                Ok(_) => Ok(ApiResult { result: ResultStatus::Success, details: "renamed file".to_string() }),
                Err(e) => {
                    tracing::error!(error = ?e);
                    Err(AppError::InternalError(e.to_string()))
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
            let (user, pool) = authenticated_check().await?;

            if let Some(username) = username {
                match DbUser::get(&UserIdentifier::Username(username), &pool).await {
                    Ok(user) => Ok(user),
                    Err(e) => {
                        tracing::error!(error = ?e);
                        Err(AppError::InternalError(e.to_string()))
                    }
                }    
            } else {
                match DbUser::get(&UserIdentifier::Id(user.id), &pool).await {
                    Ok(user) => Ok(user),
                    Err(e) => {
                        tracing::error!(error = ?e);
                        Err(AppError::InternalError(e.to_string()))
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
pub async fn test_ldap(args: LdapArgs) -> Result<ApiResult<String>, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let (_, pool) = authenticated_check().await?;
            
            let ldap_url = url::Url::parse(&args.url)?;
            is_host_reachable(&ldap_url.to_string()).await?;

            if !args.enabled.0 {
                return Err(AppError::InternalError("LDAP is disabled".to_string()));
            }

            let existing_certificate = match Attachment::get_certificate(&pool).await {
                Ok(cert) => cert,
                Err(e) => return Err(e.into())
            };

            #[allow(unused)]
            let mut settings = LdapConnSettings::default();
            if let Some(cert) = existing_certificate && ldap_url.scheme() == "ldaps" {
                let cert = native_tls::Certificate::from_pem(&cert.file_blob)?;
                let connector = native_tls::TlsConnector::builder().add_root_certificate(cert).build()?;
                settings = LdapConnSettings::new().set_connector(connector);
            } else {
                settings = LdapConnSettings::new().set_no_tls_verify(true).set_starttls(false);
            }
            
            let (conn, mut ldap) = match LdapConnAsync::with_settings(settings, ldap_url.as_str()).await {
                Ok(conn) => conn,
                Err(e) => return Ok(ApiResult { result: ResultStatus::Fail, details: e.to_string() })
            };
            ldap3::drive!(conn);

            match ldap.simple_bind(args.bind_dn.as_str(), args.bind_pw.as_str()).await {
                Ok(res) => {
                    match res.success() {
                        Ok(res) => {
                            ldap.unbind().await?;
                            Ok(ApiResult { result: ResultStatus::Success, details: res.to_string() })
                        },
                        Err(e) => {
                            ldap.unbind().await?;
                            Ok(ApiResult { result: ResultStatus::Fail, details: e.to_string() })
                        }
                    }
                },
                Err(e) => {
                    ldap.unbind().await?;
                    Ok(ApiResult { result: ResultStatus::Fail, details: e.to_string() })
                }
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
            let (_, pool) = authenticated_check().await?;
            
            match LdapArgs::get(&pool).await {
                Ok(args) => Ok(args),
                Err(e) => Err(e.into())
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[server(name=GetLdapCertificate, prefix="/api/admin/ldap", endpoint="get_certificate")]
#[instrument]
pub async fn get_certificate() -> Result<Option<Attachment>, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let (_, pool) = authenticated_check().await?;
            
            match Attachment::get_certificate(&pool).await {
                Ok(cert) => Ok(cert),
                Err(e) => Err(e.into())
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[server(name=GetLdapCertificateWithoutBlob, prefix="/api/admin/ldap", endpoint="get_certificate_metadata")]
#[instrument]
pub async fn get_certificate_without_blob() -> Result<Option<AttachmentWithoutBlob>, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let (_, pool) = authenticated_check().await?;
            
            match AttachmentWithoutBlob::get_certificate(&pool).await {
                Ok(cert) => Ok(cert),
                Err(e) => Err(e.into())
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[server(name=UpdateLdap, prefix="/api/admin", endpoint="ldap")]
#[instrument]
pub async fn update_ldap(args: LdapArgs, new_certificate: Option<AttachmentWithoutBlob>) -> Result<ApiResult<String>, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let (_, pool) = authenticated_check().await?;

            let existing_certificate = match Attachment::get_certificate(&pool).await {
                Ok(cert) => cert,
                Err(e) => return Err(e.into())
            };
            if let Some(existing_certificate) = &existing_certificate && new_certificate.is_none() {
                if let Err(e) = AttachmentWithoutBlob::delete(&AttachmentIdentifier::Id(existing_certificate.id.clone()), &pool).await {
                    tracing::error!(error = ?e);
                    return Err(AppError::InternalError(e.to_string()));
                }
            }

            // bind_pw should be hashed, but how to connect with a hashed password?
            match LdapArgs::update(&args.url, &args.bind_dn, &args.bind_pw, &args.base_dn, &args.enabled.0, &pool).await {
                Ok(_) => {
                    Ok(ApiResult { result: ResultStatus::Success, details: "successfully updated LDAP configuration".to_string() })
                },
                Err(e) => {
                    Ok(ApiResult { result: ResultStatus::Fail, details: format!("bind succeeded but failed to update DB row: {e}") })
                }
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[server(name=EnableLdap, prefix="/api/admin/ldap", endpoint="enable")]
#[instrument]
pub async fn enable_ldap() -> Result<ApiResult<String>, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let (_, pool) = authenticated_check().await?;

            match LdapArgs::enable(&pool).await {
                Ok(_) => Ok(ApiResult { result: ResultStatus::Success, details: "successfully enabled LDAP authentication".to_string() }),
                Err(e) => {
                    Ok(ApiResult { result: ResultStatus::Fail, details: format!("Failed to update DB row: {e}") })
                }
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[server(name=DisableLdap, prefix="/api/admin/ldap", endpoint="disable")]
#[instrument]
pub async fn disable_ldap() -> Result<ApiResult<String>, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let (_, pool) = authenticated_check().await?;
            
            match LdapArgs::disable(&pool).await {
                Ok(_) => Ok(ApiResult { result: ResultStatus::Success, details: "successfully disabled LDAP authentication".to_string() }),
                Err(e) => {
                    Ok(ApiResult { result: ResultStatus::Fail, details: format!("failed to update DB row: {e}") })
                }
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[server(name=GetProxmoxConf, prefix="/api/admin/proxmox", endpoint="config")]
#[instrument]
pub async fn get_proxmox_conf() -> Result<Option<ProxmoxArgs>, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let (_, pool) = authenticated_check().await?;
            
            match ProxmoxArgs::get(&pool).await {
                Ok(args) => Ok(args),
                Err(e) => Err(e.into())
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[server(name=UpdateProxmox, prefix="/api/admin/proxmox", endpoint="update")]
#[instrument]
pub async fn update_proxmox(args: ProxmoxArgs) -> Result<ApiResult<String>, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let (_, pool) = authenticated_check().await?;

            if let Err(e) = ProxmoxArgs::update(&args.base_url, &args.api_path, &args.templates_pool_id, &args.node, &args.username, &args.password, &args.api_token, &args.auth_type, &pool).await {
                return Ok(ApiResult { result: ResultStatus::Fail, details: format!("connection succeeded but failed to update DB row: {e}") });
            }

            if let Ok(Some(_)) = LdapArgs::get(&pool).await {
                crate::server::proxmox::create_realm().await?;
            };

            _ = crate::server::proxmox::create_user_role().await;
            _ = crate::server::proxmox::sync_realm().await;

            Ok(ApiResult { result: ResultStatus::Success, details: "successfully updated Proxmox configuration".to_string() })
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[server(name=TestProxmox, prefix="/api/admin/proxmox", endpoint="test")]
#[instrument]
pub async fn test_proxmox(args: ProxmoxArgs) -> Result<ApiResult<String>, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let (_, _) = authenticated_check().await?;
            
            match crate::server::proxmox::test_auth(&args).await {
                Ok(_) => Ok(ApiResult { result: ResultStatus::Success, details: "success".to_string()}),
                Err(e) => Ok(ApiResult { result: ResultStatus::Fail, details: e.to_string() })
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

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

#[server(name=AdminGetAllChallengeTemplates, prefix="/api/admin/challenges", endpoint="get_templates")]
#[instrument]
pub async fn get_all_challenge_templates() -> Result<Vec<ProxmoxVMTemplate>, AppError> {
    cfg_if! {
        if #[cfg(feature = "ssr")] {
            let (_, _) = authenticated_check().await?;

            match crate::server::proxmox::get_all_templates().await {
                Ok(templates) => Ok(templates),
                Err(e) => {
                    tracing::error!(error = ?e);
                    Err(e)
                }
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[server(name=GetAllHints, prefix="/api/admin/challenges", endpoint="get_hints")]
#[instrument]
pub async fn get_all_hints() -> Result<Vec<DbHint>, AppError> {
    cfg_if! {
        if #[cfg(feature = "ssr")] {
            let (_, pool) = authenticated_check().await?;

            match DbHint::get_all(&pool).await {
                Ok(hints) => Ok(hints),
                Err(e) => Err(e.into())
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}
