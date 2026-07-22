/// src/server/api.rs
/// contains code which constructs Leptos `#[server]` API functions, exposed under the path `/api`.
/// These endpoints can be used by users with the role `UserRole::Admin`.

#[cfg(feature = "ssr")]
use crate::server::{admin::{authenticated_check, fetch_db_user}, db::DbResultExt, hash_string, build_and_broadcast, fetch_cwa, is_host_reachable, fetch_ewa, BroadcastScope};
use crate::{error_template::AppError, server::{ServerEventPayload, UserRole, admin::{ChallengeAction, EventAction, ProxmoxUserInfo, UserAction}, db::{self, enums::{AttachmentIdentifier, FileType, UserIdentifier}, structs::{Attachment, AttachmentWithoutBlob, DbHint, DbUser, Event, EventWithAttachments, LdapArgs, ProxmoxArgs, UserAvatar}}, enums::ResultStatus, proxmox::ProxmoxVMTemplate, structs::ApiResult}};
use crate::error_template::LogErr;
#[cfg(feature = "ssr")]
use ldap3::{LdapConnAsync, LdapConnSettings};
use leptos::{prelude::*, server_fn::codec::{GetUrl, MultipartData, MultipartFormData}};
use std::collections::{HashMap, HashSet};
use tracing::instrument;

#[server(name=AdminChallengeApi, prefix="/api/admin", endpoint="challenge")]
#[instrument]
pub async fn challenge(action: ChallengeAction) -> Result<ApiResult<String>, AppError> {
    let (_, pool) = authenticated_check().await?;

    match action {
        ChallengeAction::Create { event_id, name, description, category, difficulty, points, flag, visible_to_groups, attachments, illustration, vm_ids, hints } => {
            let flag_hash = hash_string(&flag).await?;
            let mut tx = pool.begin().await?;
            let Ok(new_challenge_id) = db::structs::Challenge::add(&event_id, &name, &description, &category, &difficulty, &points, &flag_hash, &visible_to_groups, &vm_ids, &mut *tx).await else {
                return Err(AppError::InternalError("Failed to add challenge".to_string()));
            };

            if let Some(attachments) = attachments {
                for attachment in attachments {
                    db::structs::Attachment::edit_challenge(&attachment.id, &new_challenge_id, &mut *tx).await?;
                }
            }

            if let Some(hints) = hints {
                for hint in hints {
                    if !hint.hint.is_empty() {
                        DbHint::add(&hint.hint, &new_challenge_id, &hint.points_penalty, &mut *tx).await?;
                    }
                }
            }
            
            let broadcast_id = new_challenge_id.clone();
            let broadcast_pool = pool.clone();
            match illustration {
                Some(illustration) => {
                    db::structs::Attachment::edit_illustration(&illustration.id, &db::enums::AttachmentIdentifier::ChallengeId(new_challenge_id), &mut *tx).await?;
                    tx.commit().await?;
                    tokio::spawn(async move {
                        match fetch_cwa(&broadcast_id, &broadcast_pool).await {
                            Ok(cwa) => { 
                                _ = build_and_broadcast(ServerEventPayload::NewChallengeCreated(cwa), vec![BroadcastScope::Events, BroadcastScope::Admin]).await; 
                            },
                            Err(e) => tracing::warn!("failed to fetch cwa for broadcast: {e}")
                        }
                    });
                    Ok(ApiResult { result: ResultStatus::Success, details: "Created challenge".to_string() })
                }
                None => {
                    tx.commit().await?;
                    tokio::spawn(async move {
                        match fetch_cwa(&broadcast_id, &broadcast_pool).await {
                            Ok(cwa) => { _ = build_and_broadcast(ServerEventPayload::NewChallengeCreated(cwa), vec![BroadcastScope::Events, BroadcastScope::Admin]).await; },
                            Err(e) => tracing::warn!("failed to fetch cwa for broadcast: {e}")
                        }
                    });
                    Ok(ApiResult { result: ResultStatus::Success, details: "Created challenge".to_string() })
                }
            }
        }
        ChallengeAction::Delete { id } => {
            match db::structs::Challenge::delete(&id, &pool).await {
                Ok(_) => {
                    tokio::spawn(async move {
                        _ = build_and_broadcast(ServerEventPayload::ChallengeDeleted(id), vec![BroadcastScope::Events, BroadcastScope::Admin]).await;
                    });
                    Ok(ApiResult { result: ResultStatus::Success, details: "Deleted challenge".to_string() })
                },
                Err(_) => {
                    return Ok(ApiResult { result: ResultStatus::Fail, details: "Failed to delete challenge".to_string() });
                }
            }
        }
        ChallengeAction::Edit { id, event_id, name, description, category, difficulty, points, flag, visible_to_groups, attachments, illustration, vm_ids, hints } => {
            let flag_hash = if flag.is_empty() { String::new() } else { hash_string(&flag).await? };

            let mut tx = pool.begin().await?;
            if db::structs::Challenge::edit(&id, &event_id, &name, &description, &category, &difficulty, &points, &flag_hash, &visible_to_groups, &vm_ids, &mut *tx).await.is_err() {
                return Ok(ApiResult { result: ResultStatus::Fail, details: "Failed to edit challenge".to_string() });
            }

            let all_challenge_attachment_ids = match AttachmentWithoutBlob::get_all(&Some(db::enums::AttachmentIdentifier::ChallengeId(id.clone())), &mut *tx).await {
                Ok(all_attachments) => all_attachments.into_iter().map(|a| a.id.clone()).collect::<Vec<String>>(),
                Err(e) => return Err(e.into())
            };
            
            if let Some(attachments) = &attachments {
                for attachment in attachments.iter() {
                    db::structs::Attachment::edit_challenge(&attachment.id, &id, &mut *tx).await?;
                }
            }

            let new_attachment_ids = attachments.unwrap_or_default().into_iter().map(|h| h.id).collect::<HashSet<String>>();
            for existing_attachment_id in all_challenge_attachment_ids {
                if !new_attachment_ids.contains(&existing_attachment_id) {
                    AttachmentWithoutBlob::delete(&AttachmentIdentifier::Id(existing_attachment_id), &mut *tx).await?;
                }
            }

            if let Some(hints) = hints {
                let all_challenge_hint_ids = match DbHint::get_all_from_challenge(&id, &mut *tx).await {
                    Ok(all_hints) => all_hints.into_iter().map(|h| h.id).collect::<HashSet<String>>(),
                    Err(e) => return Err(e.into())
                };
                
                let new_hints_ids = hints.iter().map(|h| h.id.clone()).collect::<HashSet<String>>();
                for hint in hints {
                    if !hint.hint.is_empty() && !all_challenge_hint_ids.contains(&hint.id) {
                        DbHint::add(&hint.hint, &id, &hint.points_penalty, &mut *tx).await?;
                    } else if !hint.hint.is_empty() && all_challenge_hint_ids.contains(&hint.id) {
                        DbHint::edit(&hint.id, &hint.hint, &id, &hint.points_penalty, &mut *tx).await?;
                    } else if hint.hint.is_empty() && hint.points_penalty == 0 && all_challenge_hint_ids.contains(&hint.id) {
                        DbHint::delete(&hint.id, &mut *tx).await?
                    }
                }

                for existing_hint_id in all_challenge_hint_ids {
                    if !new_hints_ids.contains(&existing_hint_id) {
                        DbHint::delete(&existing_hint_id, &mut *tx).await?;
                    }
                }
            }

            let broadcast_id = id.clone();
            let broadcast_pool = pool.clone();
            match illustration {
                Some(illustration) => {
                    db::structs::Attachment::edit_illustration(&illustration.id, &db::enums::AttachmentIdentifier::ChallengeId(id), &mut *tx).await?;
                    tx.commit().await?;
                    tokio::spawn(async move {
                        match fetch_cwa(&broadcast_id, &broadcast_pool).await {
                            Ok(cwa) => { _ = build_and_broadcast(ServerEventPayload::ChallengeEdited(cwa), vec![BroadcastScope::Events, BroadcastScope::Admin]).await; },
                            Err(e) => tracing::warn!("failed to fetch cwa for broadcast: {e}")
                        }
                    });
                    Ok(ApiResult { result: ResultStatus::Success, details: "Edited challenge".to_string() })
                }
                None => {
                    let existing_illustration_id = AttachmentWithoutBlob::get_illustration_id(&db::enums::AttachmentIdentifier::ChallengeId(id), &mut *tx).await?;

                    if let Some(attachment_id) = existing_illustration_id {
                        db::structs::Attachment::delete(&db::enums::AttachmentIdentifier::Id(attachment_id.clone()), &mut *tx).await?;

                        crate::server::invalidate_file_cache(&attachment_id).await;
                        tx.commit().await?;
                        let broadcast_id = broadcast_id.clone();
                        let broadcast_pool = broadcast_pool.clone();
                        tokio::spawn(async move {
                            match fetch_cwa(&broadcast_id, &broadcast_pool).await {
                                Ok(cwa) => { _ = build_and_broadcast(ServerEventPayload::ChallengeEdited(cwa), vec![BroadcastScope::Events, BroadcastScope::Admin]).await; },
                                Err(e) => tracing::warn!("failed to fetch cwa for broadcast: {e}")
                            }
                        });

                        return Ok(ApiResult { result: ResultStatus::Success, details: "Edited challenge".to_string() });
                    }

                    tx.commit().await?;
                    tokio::spawn(async move {
                        match fetch_cwa(&broadcast_id, &broadcast_pool).await {
                            Ok(cwa) => { _ = build_and_broadcast(ServerEventPayload::ChallengeEdited(cwa), vec![BroadcastScope::Events, BroadcastScope::Admin]).await; },
                            Err(e) => tracing::warn!("failed to fetch cwa for broadcast: {e}")
                        }
                    });
                    Ok(ApiResult { result: ResultStatus::Success, details: "Edited challenge".to_string() })
                }
            }
        }
    }
}

#[server(name=AdminEventApi, prefix="/api/admin", endpoint="event")]
#[instrument]
pub async fn event(action: EventAction) -> Result<ApiResult<String>, AppError> {
    let (_, pool) = authenticated_check().await?;

    match action {
        EventAction::Create { name, description, start_at, end_at, visible_to_groups, attachments, illustration } => {
            let mut tx = pool.begin().await?;
            let Ok(new_event_id) = db::structs::Event::add(&name, &description, &start_at, &end_at, &visible_to_groups, &mut *tx).await else {
                return Ok(ApiResult { result: ResultStatus::Fail, details: "Failed to create event".to_string() });
            };

            if let Some(attachments) = attachments {
                for attachment in attachments {
                    db::structs::Attachment::edit_event(&attachment.id, &new_event_id, &mut *tx).await?;
                }
            }

            let id = new_event_id.clone();
            match illustration {
                Some(illustration) => {
                    db::structs::Attachment::edit_illustration(&illustration.id, &db::enums::AttachmentIdentifier::EventId(new_event_id), &mut *tx).await?;

                    tx.commit().await?;
                    tokio::spawn(async move {
                        match fetch_ewa(&id, &pool).await {
                            Ok(ewa) => { _ = build_and_broadcast(ServerEventPayload::NewEventCreated(ewa), vec![BroadcastScope::Events, BroadcastScope::Admin]).await; },
                            Err(e) => tracing::warn!("failed to fetch ewa for broadcast: {e}")
                        }
                    });

                    Ok(ApiResult { result: ResultStatus::Success, details: "Created event".to_string() })
                }
                None => {
                    tx.commit().await?;
                    tokio::spawn(async move {
                        match fetch_ewa(&new_event_id, &pool).await {
                            Ok(ewa) => { _ = build_and_broadcast(ServerEventPayload::NewEventCreated(ewa), vec![BroadcastScope::Events, BroadcastScope::Admin]).await; },
                            Err(e) => tracing::warn!("failed to fetch ewa for broadcast: {e}")
                        }
                    });
                    Ok(ApiResult { result: ResultStatus::Success, details: "created event".to_string() })
                }
            }
        }
        EventAction::Delete { id } => {
            match db::structs::Event::delete(&id, &pool).await {
                Ok(_) => {
                    tokio::spawn(async {
                        _ = build_and_broadcast(ServerEventPayload::EventDeleted(id), vec![BroadcastScope::Events, BroadcastScope::Admin]).await;
                    });
                    Ok(ApiResult { result: ResultStatus::Success, details: "Deleted event".to_string() })
                },
                Err(_) => {
                    Ok(ApiResult { result: ResultStatus::Fail, details: "Failed to delete event".to_string() })
                }
            }
        }
        EventAction::Edit { id, name, description, start_at, end_at, visible_to_groups, attachments, illustration } => {
            let mut tx = pool.begin().await?;
            if db::structs::Event::edit(&id, &name, &description, &start_at, &end_at, &visible_to_groups, &mut *tx).await.is_err() {
                return Ok(ApiResult { result: ResultStatus::Fail, details: "Failed to edit event".to_string() });
            }

            let all_event_attachment_ids = match AttachmentWithoutBlob::get_all(&Some(db::enums::AttachmentIdentifier::EventId(id.clone())), &mut *tx).await {
                Ok(all_attachments) => all_attachments.into_iter().map(|a| a.id.clone()).collect::<Vec<String>>(),
                Err(e) => return Err(e.into())
            };

            if let Some(attachments) = &attachments {
                for attachment in attachments {
                    db::structs::Attachment::edit_event(&attachment.id, &id, &mut *tx).await?;
                }
            }

            let new_attachment_ids = attachments.unwrap_or_default().into_iter().map(|h| h.id).collect::<HashSet<String>>();
            for existing_attachment_id in all_event_attachment_ids {
                if !new_attachment_ids.contains(&existing_attachment_id) {
                    AttachmentWithoutBlob::delete(&AttachmentIdentifier::Id(existing_attachment_id), &mut *tx).await?;
                }
            }

            let broadcast_id = id.clone();
            match illustration {
                Some(illustration) => {
                    db::structs::Attachment::edit_illustration(&illustration.id, &db::enums::AttachmentIdentifier::EventId(id), &mut *tx).await?;
                    tx.commit().await?;
                    tokio::spawn(async move {
                        match fetch_ewa(&broadcast_id, &pool).await {
                            Ok(ewa) => { _ = build_and_broadcast(ServerEventPayload::EventEdited(ewa), vec![BroadcastScope::Events, BroadcastScope::Admin]).await; },
                            Err(e) => tracing::warn!("failed to fetch ewa for broadcast: {e}")
                        }
                    });
                    Ok(ApiResult { result: ResultStatus::Success, details: "Edited event".to_string() })
                }
                None => {
                    let existing_illustration_id = AttachmentWithoutBlob::get_illustration_id(&db::enums::AttachmentIdentifier::EventId(id), &mut *tx).await?;
                    
                    if let Some(attachment_id) = existing_illustration_id {
                        db::structs::Attachment::delete(&db::enums::AttachmentIdentifier::Id(attachment_id.clone()), &mut *tx).await?;

                        crate::server::invalidate_file_cache(&attachment_id).await;
                        tx.commit().await?;
                        tokio::spawn(async move {
                            match fetch_ewa(&broadcast_id, &pool).await {
                                Ok(ewa) => { _ = build_and_broadcast(ServerEventPayload::EventEdited(ewa), vec![BroadcastScope::Events, BroadcastScope::Admin]).await; },
                                Err(e) => tracing::warn!("failed to fetch ewa for broadcast: {e}")
                            }
                        });

                        return Ok(ApiResult { result: ResultStatus::Success, details: "Edited event".to_string() });
                    }

                    tx.commit().await?;
                    tokio::spawn(async move {
                        match fetch_ewa(&broadcast_id, &pool).await {
                            Ok(ewa) => { _ = build_and_broadcast(ServerEventPayload::EventEdited(ewa), vec![BroadcastScope::Events, BroadcastScope::Admin]).await; },
                            Err(e) => tracing::warn!("failed to fetch ewa for broadcast: {e}")
                        }
                    });
                    Ok(ApiResult { result: ResultStatus::Success, details: "Edited event".to_string() })
                }
            }
        }
    }
}

#[server(name=GetAllUserAvatarIds, prefix="/api/users", endpoint="avatar_ids", input=GetUrl)]
#[instrument]
pub async fn get_all_user_avatar_ids() -> Result<Vec<UserAvatar>, AppError> {
    let (_, pool) = authenticated_check().await?;

    let ids = DbUser::get_all_avatar_ids(&pool).await?;
    Ok(ids)
}

#[server(name=AdminUsersGetAll, prefix="/api/admin", endpoint="users", input=GetUrl)]
#[instrument]
pub async fn get_all_users() -> Result<Vec<DbUser>, AppError> {
    let (_, pool) = authenticated_check().await?;

    let users = DbUser::get_all(&pool).await?;
    Ok(users)
}

#[server(name=AdminUsersGetAllGroups, prefix="/api/admin/users", endpoint="groups", input=GetUrl)]
#[instrument]
pub async fn get_all_user_groups() -> Result<Vec<String>, AppError> {
    let (_, pool) = authenticated_check().await?;

    let groups = DbUser::get_all_groups(&pool).await?;
    Ok(groups)
}

#[server(name=AdminEventsGetAll, prefix="/api/admin", endpoint="events", input=GetUrl)]
#[instrument]
pub async fn get_all_events() -> Result<Vec<Event>, AppError> {
    let (_, pool) = authenticated_check().await?;

    let events = db::structs::Event::get_all(&pool).await?;
    Ok(events)
}

#[server(name=AdminEventsGetAllWithAttachments, prefix="/api/admin", endpoint="ewa", input=GetUrl)]
#[instrument]
pub async fn get_all_events_with_attachments() -> Result<Vec<EventWithAttachments>, AppError> {
    let (_, pool) = authenticated_check().await?;

    let events = db::structs::Event::get_all(&pool).await?;

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
}

#[server(input=MultipartFormData, name=AdminUploadFile, prefix="/api/admin/file", endpoint="upload")]
#[instrument(skip(files))]
pub async fn upload_files(files: MultipartData) -> Result<ApiResult<Vec<AttachmentWithoutBlob>>, AppError> {
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

        let insert_id = db::structs::Attachment::add(&None, &None, &None, &file_name, &file_blob, &db::enums::FileType::Attachment, &Some(mime_type), &pool).await?;

        if let Some(attachment) = db::structs::AttachmentWithoutBlob::get(&db::enums::AttachmentIdentifier::Id(insert_id.clone()), &pool).await? {
            attachments.push(attachment);
        }
    }

    if !found_file {
        Err(AppError::BadRequest("no files uploaded".to_string()))
    } else {
        Ok(ApiResult { result: ResultStatus::Success, details: attachments })
    }
}

#[server(input=MultipartFormData, name=AdminUploadCertificate, prefix="/api/admin/certificate", endpoint="upload")]
#[instrument(skip(file))]
pub async fn upload_certificate(file: MultipartData) -> Result<ApiResult<AttachmentWithoutBlob>, AppError> {
    let (_, pool) = authenticated_check().await?;

    let existing_certificate = AttachmentWithoutBlob::get_certificate(&pool).await?;
    if let Some(existing_certificate) = existing_certificate {
        db::structs::Attachment::delete(&db::enums::AttachmentIdentifier::Id(existing_certificate.id.clone()), &pool).await?;
        crate::server::invalidate_file_cache(&existing_certificate.id).await;
    }

    let mut attachment = AttachmentWithoutBlob::default();

    // To-Do: .into_inner() always returns Some on server
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

        let insert_id = db::structs::Attachment::add(&None, &None, &None, &file_name, &file_blob, &db::enums::FileType::Certificate, &Some(mime_type), &pool).await?;
        if let Some(attachment_result) = db::structs::AttachmentWithoutBlob::get(&db::enums::AttachmentIdentifier::Id(insert_id.clone()), &pool).await? {
            attachment = attachment_result;
        }
    }

    if !found_file {
        Err(AppError::BadRequest("no files uploaded".to_string()))
    } else {
        Ok(ApiResult { result: ResultStatus::Success, details: attachment })
    }
}

#[server(input=MultipartFormData, name=AdminUploadIllustration, prefix="/api/admin/illustration", endpoint="upload")]
#[instrument(skip(file))]
pub async fn upload_illustration(file: MultipartData) -> Result<ApiResult<AttachmentWithoutBlob>, AppError> {
    let (_, pool) = authenticated_check().await?;

    let mut attachment = AttachmentWithoutBlob::default();

    let mut data = match file.into_inner() {
        Some(inner_data) => inner_data,
        None => {
            return Err(AppError::InternalError("Failed to extract inner data from file".to_string()));
        }
    };
    while let Ok(Some(mut field)) = data.next_field().await {
        let Some(file_name) = field.file_name().map(|n| n.to_string()) else {
            continue
        };

        let mime_type = field.content_type().map(|ct| ct.to_string()).unwrap_or_default();

        let mut file_blob = Vec::<u8>::new();
        while let Ok(Some(chunk)) = field.chunk().await {
            file_blob.extend_from_slice(&chunk);
        }

        let insert_id = db::structs::Attachment::add(&None, &None, &None, &file_name, &file_blob, &db::enums::FileType::Illustration, &Some(mime_type), &pool).await?;

        if let Some(attachment_result) = db::structs::AttachmentWithoutBlob::get(&db::enums::AttachmentIdentifier::Id(insert_id.clone()), &pool).await? {
            attachment = attachment_result;
        }
    }

    Ok(ApiResult { result: ResultStatus::Success, details: attachment })
}

#[server(input=MultipartFormData, name=AdminUploadAvatar, prefix="/api/admin/avatar", endpoint="upload")]
#[instrument(skip(file))]
pub async fn upload_avatar(file: MultipartData) -> Result<ApiResult<UserAvatar>, AppError> {
    let (_, pool) = authenticated_check().await?;

    let mut avatar = UserAvatar::default();

    let mut data = match file.into_inner() {
        Some(inner_data) => inner_data,
        None => {
            return Err(AppError::InternalError("Failed to extract inner data from file".to_string()));
        }
    };
    while let Ok(Some(mut field)) = data.next_field().await {
        let Some(file_name) = field.file_name().map(|n| n.to_string()) else {
            continue
        };

        let mime_type = field.content_type().map(|ct| ct.to_string()).unwrap_or_default();

        let mut file_blob = Vec::<u8>::new();
        while let Ok(Some(chunk)) = field.chunk().await {
            file_blob.extend_from_slice(&chunk);
        }

        let insert_id= db::structs::Attachment::add(&None, &None, &None, &file_name, &file_blob, &db::enums::FileType::Avatar, &Some(mime_type), &pool).await?;

        let result = UserAvatar {
            attachment_id: insert_id,
            user_id: None,
            file_name
        };

        avatar = result;
    }

    Ok(ApiResult { result: ResultStatus::Success, details: avatar })
}

#[server(name=AdminGetAllCategories, prefix="/api/admin/challenges", endpoint="categories", input=GetUrl)]
#[instrument]
pub async fn get_all_challenge_categories() -> Result<Vec<String>, AppError> {
    let (_, pool) = authenticated_check().await?;

    let categories = db::structs::Challenge::get_all_categories(&pool).await?;
    Ok(categories)
}

#[server(name=AdminGetAllFiles, prefix="/api/admin/files", endpoint="all", input=GetUrl)]
#[instrument]
pub async fn get_all_files() -> Result<Vec<AttachmentWithoutBlob>, AppError> {
    let (_, pool) = authenticated_check().await?;

    let attachments = db::structs::AttachmentWithoutBlob::get_all(&None, &pool).await?;
    Ok(attachments)
}

#[server(name=AdminUserApi, prefix="/api/admin", endpoint="user")]
#[instrument]
pub async fn user(action: UserAction) -> Result<ApiResult<String>, AppError> {
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
            let Some(new_user_id) = DbUser::add(&new_user, &mut *tx).await.none_on_unique_violation()? else {
                return Ok(ApiResult { result: ResultStatus::Fail, details: "Username or e-mail already exists".to_string() });
            };

            let broadcast_id = new_user_id.clone();
            if let Some(avatar) = avatar {
                match AttachmentWithoutBlob::edit_avatar(&avatar.attachment_id, &new_user_id, &mut *tx).await {
                    Ok(_) => {
                        tx.commit().await?;
                        tokio::spawn(async move {
                            match fetch_db_user(&broadcast_id, &pool).await {
                                Ok(db_user) => { _ = build_and_broadcast(ServerEventPayload::UserCreated(db_user), vec![BroadcastScope::Admin]).await; },
                                Err(e) => tracing::warn!("failed to fetch cwa for broadcast: {e}")
                            }
                        });
                        Ok(ApiResult { result: ResultStatus::Success, details: "created user".to_string() })
                    },
                    Err(_) => {
                        Ok(ApiResult { result: ResultStatus::Fail, details: "failed to set user avatar".to_string() })
                    }
                }
            } else {
                tx.commit().await?;
                tokio::spawn(async move {
                    match fetch_db_user(&broadcast_id, &pool).await {
                        Ok(db_user) => { _ = build_and_broadcast(ServerEventPayload::UserCreated(db_user), vec![BroadcastScope::Admin]).await; },
                        Err(e) => tracing::warn!("failed to fetch cwa for broadcast: {e}")
                    }
                });
                Ok(ApiResult { result: ResultStatus::Success, details: "Created user".to_string() })
            }
        }
        UserAction::Delete { id } => {
            let Some(user) = DbUser::get(&UserIdentifier::Id(id.clone()), &pool).await? else {
                return Err(AppError::BadRequest("Invalid user id".to_string()));
            };

            if user.role == UserRole::Admin {
                return Err(AppError::InternalError("Cannot delete admin users".to_string()));
            }

            let broadcast_id = id.clone();
            match DbUser::delete(&user.id, &pool).await {
                Ok(_) => {
                    tokio::spawn(async move {
                        _ = build_and_broadcast(ServerEventPayload::UserDeleted(broadcast_id), vec![BroadcastScope::Admin]).await;
                    });
                    Ok(ApiResult { result: ResultStatus::Success, details: "deleted user".to_string() })
                },
                Err(_) => {
                    Ok(ApiResult { result: ResultStatus::Fail, details: "failed to delete user".to_string() })
                }
            }
        }
        UserAction::Edit { id, username, email, password, confirm_password, points, role, avatar, groups } => {
            let Some(user) = DbUser::get(&UserIdentifier::Id(id.clone()), &pool).await? else {
                return Err(AppError::BadRequest("Invalid user id".to_string()));
            };

            if user.role == UserRole::Admin {
                return Err(AppError::BadRequest("Cannot edit admin users".to_string()));
            }

            if password != confirm_password {
                return Err(AppError::BadRequest("password and confirm password must match".to_string()));
            }

            if username.is_empty() {
                return Err(AppError::BadRequest("username must not be empty".to_string()));
            }

            let mut tx = pool.begin().await?;
            if DbUser::edit_username(&user.id, &username, &mut *tx).await.is_unique_violation()? {
                return Ok(ApiResult { result: ResultStatus::Fail, details: "Username already in use".to_string() });
            }

            if DbUser::edit_email(&user.id, &email, &mut *tx).await.is_unique_violation()? {
                return Ok(ApiResult { result: ResultStatus::Fail, details: "E-mail already in use".to_string() });
            }

            if DbUser::edit_points(&user.id, &points, &mut *tx).await.is_err() {
                return Ok(ApiResult { result: ResultStatus::Fail, details: "failed to edit points".to_string() });
            }

            if DbUser::edit_role(&id, &role, &mut *tx).await.is_err() {
                return Ok(ApiResult { result: ResultStatus::Fail, details: "failed to edit role".to_string() });
            }

            if DbUser::edit_groups(&id, &groups, &mut *tx).await.is_err() {
                return Ok(ApiResult { result: ResultStatus::Fail, details: "failed to edit groups".to_string() });
            }

            if let Some(avatar) = avatar {
                if AttachmentWithoutBlob::edit_avatar(&avatar.attachment_id, &id, &mut *tx).await.is_err() {
                    return Ok(ApiResult { result: ResultStatus::Fail, details: "failed to set user avatar".to_string() });
                }
            } else {
                let Ok(existing_avatar) = DbUser::get_avatar(&UserIdentifier::Id(id.clone()), &mut *tx).await else {
                    return Ok(ApiResult { result: ResultStatus::Fail, details: "Failed to fetch existing avatar".to_string() });
                };

                if let Some(existing_avatar) = existing_avatar {
                    _ = db::structs::Attachment::delete(&db::enums::AttachmentIdentifier::Id(existing_avatar.id.clone()), &mut *tx).await?;
                    crate::server::invalidate_file_cache(&existing_avatar.id).await;
                }
            }

            tx.commit().await?;
            let broadcast_id = id.clone();
            tokio::spawn(async move {
                match fetch_db_user(&broadcast_id, &pool).await {
                    Ok(db_user) => { _ = build_and_broadcast(ServerEventPayload::UserEdited(db_user), vec![BroadcastScope::Admin]).await; },
                    Err(e) => tracing::warn!("failed to fetch cwa for broadcast: {e}")
                }
            });
            return Ok(ApiResult { result: ResultStatus::Success, details: "edited user".to_string() });
        }
        UserAction::EditPassword { id, password, confirm_password } => {
            let Some(user) = DbUser::get(&UserIdentifier::Id(id), &pool).await? else {
                return Err(AppError::BadRequest("Invalid user id".to_string()));
            };

            if user.role == UserRole::Admin {
                return Err(AppError::BadRequest("Cannot edit password on admin users".to_string()));
            }

            if password != confirm_password {
                return Err(AppError::BadRequest("password and confirm password must match".to_string()));
            }

            let hashed_pw = hash_string(&password).await?;

            match DbUser::edit_password(&user.id, &hashed_pw, &pool).await {
                Ok(_) => Ok(ApiResult { result: ResultStatus::Success, details: "edited user".to_string() }),
                Err(_) => {
                    Ok(ApiResult { result: ResultStatus::Fail, details: "failed to edit password".to_string() })
                }
            }
        }
    }
}

#[server(name=AdminDeleteFile, prefix="/api/admin/file", endpoint="delete")]
#[instrument]
pub async fn delete_file(id: String) -> Result<ApiResult<String>, AppError> {
    let (_, pool) = authenticated_check().await?;

    _ = db::structs::Attachment::delete(&db::enums::AttachmentIdentifier::Id(id.clone()), &pool).await?;
    crate::server::invalidate_file_cache(&id).await;
    Ok(ApiResult { result: ResultStatus::Success, details: "deleted file".to_string() })
}

#[server(name=AdminRenameFile, prefix="/api/admin/file", endpoint="rename")]
#[instrument]
pub async fn rename_file(id: String, file_name: String) -> Result<ApiResult<String>, AppError> {
    let (_, pool) = authenticated_check().await?;

    _ = db::structs::Attachment::edit_file_name(&id, &file_name, &pool).await?; 
    Ok(ApiResult { result: ResultStatus::Success, details: "renamed file".to_string() })
}

#[server(name=TestLdap, prefix="/api/admin", endpoint="test_ldap")]
#[instrument(skip(args))]
pub async fn test_ldap(args: LdapArgs) -> Result<ApiResult<String>, AppError> {
    let (_, pool) = authenticated_check().await?;
    
    let ldap_url = url::Url::parse(&args.url)?;
    if !is_host_reachable(&ldap_url.to_string()).await? {
        return Err(AppError::NetworkError("LDAP host unreachable".to_string()));
    }

    if !args.enabled.0 {
        return Err(AppError::InternalError("LDAP is disabled".to_string()));
    }

    let existing_certificate = Attachment::get_certificate(&pool).await?;

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
}

#[server(name=GetLdap, prefix="/api/admin", endpoint="ldap", input=GetUrl)]
#[instrument]
pub async fn get_ldap() -> Result<Option<LdapArgs>, AppError> {
    let (_, pool) = authenticated_check().await?;
    
    let args = LdapArgs::get(&pool).await?;
    Ok(args)
}

#[server(name=GetLdapCertificate, prefix="/api/admin/ldap", endpoint="certificate", input=GetUrl)]
#[instrument]
pub async fn get_certificate() -> Result<Option<Attachment>, AppError> {
    let (_, pool) = authenticated_check().await?;
    
    let cert = Attachment::get_certificate(&pool).await?;
    Ok(cert)
}

#[server(name=GetLdapCertificateWithoutBlob, prefix="/api/admin/ldap/certificate", endpoint="metadata", input=GetUrl)]
#[instrument]
pub async fn get_certificate_without_blob() -> Result<Option<AttachmentWithoutBlob>, AppError> {
    let (_, pool) = authenticated_check().await?;
    
    let cert = AttachmentWithoutBlob::get_certificate(&pool).await?;
    Ok(cert)
}

#[server(name=UpdateLdap, prefix="/api/admin", endpoint="ldap")]
#[instrument]
pub async fn update_ldap(args: LdapArgs, new_certificate: Option<AttachmentWithoutBlob>) -> Result<ApiResult<String>, AppError> {
    let (_, pool) = authenticated_check().await?;

    let existing_certificate = Attachment::get_certificate(&pool).await?;
    if let Some(existing_certificate) = &existing_certificate && new_certificate.is_none() {
        AttachmentWithoutBlob::delete(&AttachmentIdentifier::Id(existing_certificate.id.clone()), &pool).await?;
    }

    // bind_pw should be hashed, but how to connect with a hashed password?
    LdapArgs::update(&args.url, &args.bind_dn, &args.bind_pw, &args.base_dn, &args.enabled.0, &pool).await?;
    Ok(ApiResult { result: ResultStatus::Success, details: "successfully updated LDAP configuration".to_string() })
}

#[server(name=EnableLdap, prefix="/api/admin/ldap", endpoint="enable")]
#[instrument]
pub async fn enable_ldap() -> Result<ApiResult<String>, AppError> {
    let (_, pool) = authenticated_check().await?;

    LdapArgs::enable(&pool).await?;
    Ok(ApiResult { result: ResultStatus::Success, details: "successfully enabled LDAP authentication".to_string() })
}

#[server(name=DisableLdap, prefix="/api/admin/ldap", endpoint="disable")]
#[instrument]
pub async fn disable_ldap() -> Result<ApiResult<String>, AppError> {
    let (_, pool) = authenticated_check().await?;
    
    LdapArgs::disable(&pool).await?;
    Ok(ApiResult { result: ResultStatus::Success, details: "successfully disabled LDAP authentication".to_string() })
}

#[server(name=GetProxmoxConf, prefix="/api/admin/proxmox", endpoint="config", input=GetUrl)]
#[instrument]
pub async fn get_proxmox_conf() -> Result<Option<ProxmoxArgs>, AppError> {
    let (_, pool) = authenticated_check().await?;
    
    let args = ProxmoxArgs::get(&pool).await?;
    Ok(args)
}

#[server(name=UpdateProxmox, prefix="/api/admin/proxmox", endpoint="update")]
#[instrument]
pub async fn update_proxmox(args: ProxmoxArgs) -> Result<ApiResult<String>, AppError> {
    let (_, pool) = authenticated_check().await?;

    if ProxmoxArgs::update(&args.base_url, &args.api_path, &args.templates_pool_id, &args.node, &args.username, &args.password, &args.api_token, &args.auth_type, &pool).await.is_err() {
        return Ok(ApiResult { result: ResultStatus::Fail, details: "connection succeeded but failed to save Proxmox configuration".to_string() });
    }

    if let Ok(Some(_)) = LdapArgs::get(&pool).await {
        crate::server::proxmox::create_realm().await?;
    };

    _ = crate::server::proxmox::create_user_role().await;
    _ = crate::server::proxmox::sync_realm().await;

    Ok(ApiResult { result: ResultStatus::Success, details: "successfully updated Proxmox configuration".to_string() })
}

#[server(name=TestProxmox, prefix="/api/admin/proxmox", endpoint="test")]
#[instrument]
pub async fn test_proxmox(args: ProxmoxArgs) -> Result<ApiResult<String>, AppError> {
    let (_, _) = authenticated_check().await?;
    
    match crate::server::proxmox::test_auth(&args).await {
        Ok(_) => Ok(ApiResult { result: ResultStatus::Success, details: "success".to_string()}),
        Err(e) => Ok(ApiResult { result: ResultStatus::Fail, details: e.to_string() })
    }
}

#[server(name=AdminGetAllChallengeTemplates, prefix="/api/admin/challenges", endpoint="templates", input=GetUrl)]
#[instrument]
pub async fn get_all_challenge_templates() -> Result<Vec<ProxmoxVMTemplate>, AppError> {
    let (_, _) = authenticated_check().await?;

    match crate::server::proxmox::get_all_templates().await {
        Ok(templates) => Ok(templates),
        Err(e) => Err(e)
    }
}

#[server(name=GetAllHints, prefix="/api/admin/challenges", endpoint="hints", input=GetUrl)]
#[instrument]
pub async fn get_all_hints() -> Result<Vec<DbHint>, AppError> {
    let (_, pool) = authenticated_check().await?;

    let hints = DbHint::get_all(&pool).await?;
    Ok(hints)
}

#[server(name=GetProxmoxUsersInfo, prefix="/api/admin/proxmox", endpoint="users_info", input=GetUrl)]
#[instrument]
pub async fn get_proxmox_users_info() -> Result<Vec<ProxmoxUserInfo>, AppError> {
    let (_, pool) = authenticated_check().await?;

    let base_url = crate::server::proxmox::get_proxmox_base_url().await?;
    if !is_host_reachable(&base_url.to_string()).await? {
        return Err(AppError::NetworkError("Proxmox host unreachable".to_string()));
    }

    let all_users = DbUser::get_all(&pool).await?;

    let proxmox_userids = crate::server::proxmox::get_proxmox_userids().await.or_log_and_default();
    let proxmox_poolids = crate::server::proxmox::get_proxmox_poolids().await.or_log_and_default();

    let mut proxmox_users_info = Vec::<ProxmoxUserInfo>::new();
    for user in all_users {
        let vms = match crate::server::proxmox::get_user_vms(&user).await {
            Ok(vms) => vms,
            Err(_) => vec![]
        };

        let realm_suffix = if user.auth_type == "ldap" { "CTFPKHK" } else { "pve" };
        let expected_user_id = format!("{}@{realm_suffix}", user.username);
        let expected_pool = format!("CTFPKHK-{}", user.username);

        let pve_user_id = if proxmox_userids.contains(&expected_user_id) { Some(expected_user_id) } else { None };
        let pool = if proxmox_poolids.contains(&expected_pool) { Some(expected_pool) } else { None };

        proxmox_users_info.push(ProxmoxUserInfo { user, pve_user_id, pool, vms })
    }

    Ok(proxmox_users_info)
}

#[server(name=CreateProxmoxUser, prefix="/api/admin/proxmox", endpoint="create_user")]
#[instrument]
pub async fn create_proxmox_user(#[server(rename = "user_id")] user_db_id: String) -> Result<ApiResult<String>, AppError> {
    let (_, pool) = authenticated_check().await?;

    let Some(db_user) = DbUser::get(&UserIdentifier::Id(user_db_id.clone()), &pool).await? else {
        return Ok(ApiResult { result: ResultStatus::Fail, details: "Invalid user id".to_string() });
    };

    crate::server::proxmox::create_proxmox_user(&db_user).await?;
    Ok(ApiResult { result: ResultStatus::Success, details: "User created".to_string() })
}

#[server(name=DeleteProxmoxUser, prefix="/api/admin/proxmox", endpoint="delete_user")]
#[instrument]
pub async fn delete_proxmox_user(#[server(rename = "user_id")] user_db_id: String) -> Result<ApiResult<String>, AppError> {
    let (_, pool) = authenticated_check().await?;

    let Some(db_user) = DbUser::get(&UserIdentifier::Id(user_db_id.clone()), &pool).await? else {
        return Ok(ApiResult { result: ResultStatus::Fail, details: "Invalid user id".to_string() });
    };

    crate::server::proxmox::delete_proxmox_user(&db_user).await?;
    Ok(ApiResult { result: ResultStatus::Success, details: "User deleted".to_string() })
}

#[server(name=CreateProxmoxPool, prefix="/api/admin/proxmox", endpoint="create_pool")]
#[instrument]
pub async fn create_proxmox_pool(#[server(rename = "user_id")] user_db_id: String) -> Result<ApiResult<String>, AppError> {
    let (_, pool) = authenticated_check().await?;

    let Some(db_user) = DbUser::get(&UserIdentifier::Id(user_db_id.clone()), &pool).await? else {
        return Ok(ApiResult { result: ResultStatus::Fail, details: "Invalid user id".to_string() });
    };

    crate::server::proxmox::create_user_pool(&db_user).await?;
    Ok(ApiResult { result: ResultStatus::Success, details: "Pool created".to_string() })
}

#[server(name=DeleteProxmoxPool, prefix="/api/admin/proxmox", endpoint="delete_pool")]
#[instrument]
pub async fn delete_proxmox_pool(#[server(rename = "user_id")] user_db_id: String) -> Result<ApiResult<String>, AppError> {
    let (_, pool) = authenticated_check().await?;

    let Some(db_user) = DbUser::get(&UserIdentifier::Id(user_db_id.clone()), &pool).await? else {
        return Ok(ApiResult { result: ResultStatus::Fail, details: "Invalid user id".to_string() });
    };

    crate::server::proxmox::delete_user_pool(&db_user).await?;
    Ok(ApiResult { result: ResultStatus::Success, details: "Pool deleted".to_string() })
}

#[server(name=StartVM, prefix="/api/admin", endpoint="start_vm")]
#[instrument]
pub async fn start_vm(vm_id: u32, template_id: u32, user_id: String) -> Result<ApiResult<String>, AppError> {
    let (_, _) = authenticated_check().await?;

    let vm_id = crate::server::proxmox::admin::start_vm(&vm_id, &template_id, &user_id).await?;
    Ok(ApiResult { result: ResultStatus::Success, details: format!("Successfully started VM (ID: {vm_id})") })
}

#[server(name=RestartVM, prefix="/api/admin", endpoint="restart_vm")]
#[instrument]
pub async fn restart_vm(vm_id: u32, template_id: u32, user_id: String) -> Result<ApiResult<String>, AppError> {
    let (_, _) = authenticated_check().await?;

    let vm_id = crate::server::proxmox::admin::restart_vm(&vm_id, &template_id, &user_id).await?;
    Ok(ApiResult { result: ResultStatus::Success, details: format!("Successfully restarted VM (ID: {vm_id})") })
}

#[server(name=DestroyVM, prefix="/api/admin", endpoint="destroy_vm")]
#[instrument]
pub async fn destroy_vm(vm_id: u32, template_id: u32, user_id: String) -> Result<ApiResult<String>, AppError> {
    let (_, _) = authenticated_check().await?;

    let vm_id = crate::server::proxmox::admin::destroy_vm(&vm_id, &template_id, &user_id).await?;
    Ok(ApiResult { result: ResultStatus::Success, details: format!("Successfully destroyed VM (ID: {vm_id})") })
}

#[server(name=AddVMTime, prefix="/api/admin", endpoint="add_vm_time")]
#[instrument]
pub async fn add_vm_time(vm_id: u32, template_id: u32, user_id: String) -> Result<ApiResult<String>, AppError> {
    let (_, _) = authenticated_check().await?;

    let vm_id = crate::server::proxmox::admin::add_vm_time(&vm_id, &template_id, &user_id).await?;
    Ok(ApiResult { result: ResultStatus::Success, details: format!("Successfully added time to VM (ID: {vm_id})") })
}
