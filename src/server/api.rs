/// src/server/api.rs
/// contains code which constructs Leptos `#[server]` API functions, exposed under the path `/api`.
/// These endpoints can be used by all authenticated users, with some exceptions, e.g. `login_user,
/// `register_user`, `user_exists`, and `logout_user`, which, as expected, do not require the user
/// to be logged in.

#[cfg(feature = "ssr")]
use crate::server::db::DbResultExt;
#[cfg(feature = "ssr")]
use crate::server::{authenticated_check, build_and_broadcast, get_db_user, BroadcastScope, backend::{AuthSession, hash_string, verify_hash}};
use crate::{error_template::{AppError}, server::{backend::enums::AuthType, db::{self, enums::{FileType, UserIdentifier}, structs::{AttachmentWithoutBlob, ChallengeWithAttachments, DbHint, DbUser, DbUserWithoutPII, Event, HintWithoutHint, HintsUsed, LdapArgs}}, enums::{ResultStatus, ServerEventPayload}, proxmox::{ProxmoxVMInstance, ProxmoxVMTemplate}, structs::{ApiResult, Credentials, LeaderboardData, PivotRow, User}}, utils::{get_context, is_visible_to, offset_to_datetime, parse_vm_ids}};
use crate::error_template::LogErr;
#[cfg(feature = "ssr")]
use axum_login::AuthnBackend;
use chrono::{DateTime, Local};
use leptos::server_fn::codec::GetUrl;
use leptos::{prelude::{use_context}, server, server_fn::codec::{MultipartData, MultipartFormData}};
#[cfg(feature = "ssr")]
use leptos_axum::ResponseOptions;
use tracing::instrument;
use zeroize::Zeroizing;
use std::collections::{BTreeSet, HashMap};
#[cfg(feature = "ssr")]
use axum::http::StatusCode;
use crate::server::db::enums::AttachmentIdentifier;

#[server(name=Challenges, prefix="/api", endpoint="challenges", input=GetUrl)]
#[instrument]
pub async fn get_all_challenges_with_attachments() -> Result<Vec<ChallengeWithAttachments>, AppError> {
    let (user, pool) = authenticated_check().await?;

    let db_user = get_db_user(&user, &pool).await?;
    let challenges = db::structs::Challenge::get_all(&pool).await?;
    let all_attachments = AttachmentWithoutBlob::get_all(&None, &pool).await?;

    let mut attachments_by_challenge = HashMap::<String, Vec<AttachmentWithoutBlob>>::new();
    let mut illustration_by_challenge = HashMap::<String, AttachmentWithoutBlob>::new();

    for att in all_attachments {
        if let Some(event_id) = &att.challenge_id {
            if att.file_type == FileType::Illustration {
                illustration_by_challenge.insert(event_id.clone(), att);
            } else if att.file_type == FileType::Attachment {
                attachments_by_challenge.entry(event_id.clone()).or_default().push(att);
            }
        }
    }

    let mut cwa: Vec<ChallengeWithAttachments> = Vec::new();
    for challenge in challenges {
        if is_visible_to(&db_user, &challenge.visible_to_groups) {
            let attachments = attachments_by_challenge.remove(&challenge.id).unwrap_or_default();
            let illustration = illustration_by_challenge.remove(&challenge.id);
            cwa.push(ChallengeWithAttachments { challenge, attachments, illustration });
        }
    }
    Ok(cwa)
}

#[server(name=Leaderboard, prefix="/api", endpoint="leaderboard", input=GetUrl)]
#[instrument]
pub async fn build_leaderboard_data() -> Result<LeaderboardData, AppError> {
    let (user, pool) = authenticated_check().await?;

    let db_user = get_db_user(&user, &pool).await?;

    // To-Do: should be active_event_ids in the future
    let active_event_id = match get_active_events().await {
        Ok(active_events) => {
            let mut active_event_id = "".to_string();
            for active_event in active_events {
                if is_visible_to(&db_user, &active_event.visible_to_groups) {
                    active_event_id = active_event.id;
                    break;
                }
            }
            active_event_id
        },
        Err(_) => {
            return Err(AppError::InternalError("Failed to fetch active events".to_string()));
        }
    };

    let meta = db::structs::Event::get_metadata(&active_event_id, &pool).await?;

    let event_name = meta.name.unwrap_or_default();
    if let Some(first_submission) = meta.first_submission && let Some(last_submission) = meta.last_submission {
        let x_min = offset_to_datetime(first_submission);
        let x_max = offset_to_datetime(last_submission);

        let y_max = db::structs::Event::get_total_possible_points(&active_event_id, &pool).await?;

        let solves = sqlx::query!(
            r#"
            WITH first_solves AS (
                SELECT user_id, challenge_id, MIN(solved_at) AS solved_at
                FROM submissions
                WHERE event_id = ?
                GROUP BY user_id, challenge_id
            )
            SELECT fs.user_id, u.username, fs.solved_at, c.points
            FROM first_solves fs
            JOIN challenges c ON c.id = fs.challenge_id
            JOIN users u ON u.id = fs.user_id
            ORDER BY fs.solved_at
            "#,
            active_event_id
        )
        .fetch_all(&pool)
        .await.log_err()?;

        let users: Vec<String> = solves.iter().map(|r| r.username.clone()).collect::<BTreeSet<_>>().into_iter().collect();

        let mut timestamps = BTreeSet::new();

        #[derive(Debug)]
        struct Solve { username: String, ts: DateTime<Local>, points: f64 }

        let mut solves_parsed: Vec<Solve> = Vec::new();
        for r in solves {
            let ts = match r.solved_at {
                Some(ts) => offset_to_datetime(ts),
                None => chrono::Local::now()
            };
            timestamps.insert(ts);
            solves_parsed.push(Solve {
                username: r.username,
                ts,
                points: r.points as f64,
            });
        }

        let times = timestamps.into_iter().collect::<Vec<DateTime<Local>>>();

        let mut solves_by_ts = HashMap::<DateTime<Local>, Vec<&Solve>>::new();
        for s in &solves_parsed {
            solves_by_ts.entry(s.ts).or_default().push(s);
        }

        let user_index = users.iter().enumerate().map(|(i, u)| (u.as_str(), i)).collect::<HashMap<&str, usize>>();
        let mut cumulative = vec![0.0_f64; users.len()];
        let mut rows = Vec::<PivotRow>::with_capacity(times.len());

        for ts in times {
            if let Some(slist) = solves_by_ts.get(&ts) {
                for s in slist {
                    if let Some(&idx) = user_index.get(s.username.as_str()) {
                        cumulative[idx] += s.points;
                    }
                }
            }

            let values = users.iter().enumerate()
                .map(|(i, u)| (u.clone(), cumulative[i]))
                .collect::<HashMap<String, f64>>();
            rows.push(PivotRow { ts, values });
        }

        Ok(LeaderboardData { event_name, x_min, x_max, y_max: y_max as f64, users, rows })
    } else {
        Ok(LeaderboardData::default())
    }
}

#[server(name=LoginUser, prefix="/api", endpoint="login")]
#[instrument(skip(creds))]
pub async fn login_user(creds: Credentials) -> Result<ApiResult<User>, AppError> {
    let mut auth = get_context::<AuthSession>()?;
    let auth_type = creds.auth_type.clone();
    let user = auth.backend.authenticate(creds).await?;

    if let Some(user) = user.as_ref() {
        auth.login(user).await?;
        let db_user = get_db_user(&user, &auth.backend.pool).await?;
        let last_active_at = chrono::Local::now();
        _ = DbUser::edit_last_active(&user.id, &last_active_at, &auth.backend.pool).await;
        
        // normal users have a user + pool created on register,
        // ldap users already have a user, but no pool, so create one on login
        if auth_type == AuthType::Ldap {
            _ = crate::server::proxmox::create_user_pool(&db_user).await;
        }
        
        Ok(ApiResult { result: ResultStatus::Success, details: user.clone() })
    } else {
        Err(AppError::BadRequest("invalid credentials".to_string()))
    }
}

#[server(name=GetUser, prefix="/api", endpoint="user", input=GetUrl)]
#[instrument]
pub async fn get_user() -> Result<Option<User>, AppError> {
    let response = get_context::<ResponseOptions>()?;
    match use_context::<AuthSession>() {
        Some(session) => Ok(session.user),
        None => {
            response.set_status(StatusCode::FORBIDDEN);
            Ok(None)
        }
    }
}

#[server(name=GetUserPoints, prefix="/api/user", endpoint="points", input=GetUrl)]
#[instrument]
pub async fn get_user_points() -> Result<u32, AppError> {
    let (user, pool) = authenticated_check().await?;

    let points = db::structs::Submission::get_user_points(&user.id, &pool).await?;
    Ok(points)
}

#[server(name=GetDbUserWithoutPII, prefix="/api/user", endpoint="info")]
#[instrument]
pub async fn get_db_user_without_pii(username: Option<String>) -> Result<Option<DbUserWithoutPII>, AppError> {
    let (user, pool) = authenticated_check().await?;

    if username.is_some() {
        let user = DbUserWithoutPII::get(&UserIdentifier::Username(username.unwrap_or_default()), &pool).await?;
        Ok(user)
    } else {
        let user = DbUserWithoutPII::get(&UserIdentifier::Id(user.id), &pool).await?;
        Ok(user)
    }
}

#[server(name=Register, prefix="/api", endpoint="register")]
#[instrument(skip(password))]
pub async fn register_user(email: Zeroizing<String>, password: Zeroizing<String>, confirm_password: Zeroizing<String>) -> Result<ApiResult<User>, AppError> {
    let mut auth_session = get_context::<AuthSession>()?;

    if password != confirm_password {
        return Err(AppError::BadRequest("password and confirm password must match".to_string()));
    }

    let user = auth_session.backend.add_user(&email, password.clone()).await?;

    auth_session.login(&user).await?;
    let db_user = get_db_user(&user, &auth_session.backend.pool).await?;
    _ = crate::server::proxmox::create_user(&email, &db_user.username, &password).await;
    _ = crate::server::proxmox::create_user_pool(&db_user).await;
    tokio::spawn(async move {
        _ = build_and_broadcast(ServerEventPayload::UserCreated(db_user), vec![BroadcastScope::Admin]).await;
    });
    Ok(ApiResult { result: ResultStatus::Success, details: user })
}

#[server(name=CheckFlag, prefix="/api", endpoint="check_flag")]
#[instrument]
pub async fn check_flag(flag: String, challenge_id: String) -> Result<ApiResult<String>, AppError> {
    let (user, pool) = authenticated_check().await?;
    let db_user = get_db_user(&user, &pool).await?;

    let mut tx = pool.begin().await?;

    let challenge = db::structs::Challenge::get(&challenge_id, &mut *tx).await.or_not_found(
        AppError::BadRequest("Invalid challenge id".to_string())
    )?;

    let solved = db::structs::Submission::get_user_solved_challenges(&user.id, &mut *tx).await?;
    if solved.contains(&challenge.id) {
        return Ok(ApiResult { result: ResultStatus::Fail, details: "challenge already solved".to_string() });
    }

    let challenge_flag_hash = db::structs::Challenge::get_flag_hash(&challenge.id, &mut *tx).await?;

    tx.commit().await?;
    if verify_hash(&flag, &challenge_flag_hash).await.is_err() {
        return Ok(ApiResult { result: ResultStatus::Fail, details: "incorrect solution".to_string() });
    }

    // Re-check solved status and insert atomically to prevent duplicate submissions
    let mut tx = pool.begin().await?;
    let solved = db::structs::Submission::get_user_solved_challenges(&user.id, &mut *tx).await?;
    if solved.contains(&challenge.id) {
        return Ok(ApiResult { result: ResultStatus::Fail, details: "challenge already solved".to_string() });
    }

    if db::structs::Submission::add(&challenge.id, &challenge.event_id, &user.id, &challenge.points, &chrono::Local::now(), &mut *tx).await.is_unique_violation()? {
        return Ok(ApiResult { result: ResultStatus::Fail, details: "challenge already solved".to_string() });
    }

    tx.commit().await?;

    if let Some(vm_ids_string) = challenge.vm_ids {
        let template_ids = parse_vm_ids(&vm_ids_string);
        for template_id in template_ids.iter() {
            _ = crate::server::proxmox::destroy_vm(&db_user, template_id).await;
        }
    };

    tokio::spawn(async move {
        _ = build_and_broadcast(ServerEventPayload::ChallengeSolved, vec![BroadcastScope::Events]).await;
    });
    Ok(ApiResult { result: ResultStatus::Success, details: "correct solution".to_string() })

}

#[server(name=EditUsername, prefix="/api/user", endpoint="username")]
#[instrument]
pub async fn edit_username(username: String) -> Result<ApiResult<String>, AppError> {
    let (user, pool) = authenticated_check().await?;
    let db_user = get_db_user(&user, &pool).await?;

    if username == db_user.username {
        return Ok(ApiResult { result: ResultStatus::Fail, details: "Username already exists".to_string() });
    } else if username.is_empty() || !username.is_ascii() {
        return Err(AppError::InternalError("Invalid username".to_string()));
    } else {
        if DbUser::edit_username(&user.id, &username, &pool).await.is_unique_violation()? {
            return Ok(ApiResult { result: ResultStatus::Fail, details: "Username already exists".to_string() });
        }
        Ok(ApiResult { result: ResultStatus::Success, details: "Changed username".to_string() })
    }
}

#[server(input=MultipartFormData, name=EditAvatar, prefix="/api/user", endpoint="edit_avatar")]
#[instrument(skip(avatar))]
pub async fn edit_avatar(avatar: MultipartData) -> Result<ApiResult<String>, AppError> {
    let (user, pool) = authenticated_check().await?;
    
    let mut data = match avatar.into_inner() {
        Some(inner_data) => inner_data,
        None => {
            return Err(AppError::InternalError("Failed to extract inner data from avatar".to_string()));
        }
    };
    let mut file_name = String::new();
    let mut file_blob = Vec::<u8>::new();
    let mut mime_type = String::new();
    while let Ok(Some(mut field)) = data.next_field().await {
        if let Some(field_file_name) = field.file_name() {
            file_name = field_file_name.to_string();
        } else {
            return Err(AppError::InternalError("Failed to extract file name".to_string()))
        }

        if let Some(field_content_type) = field.content_type() {
            mime_type = field_content_type.to_string();
        } else {
            return Err(AppError::InternalError("Failed to extract content type".to_string()))
        }

        while let Ok(Some(chunk)) = field.chunk().await {
            file_blob.extend_from_slice(&chunk);
        }
    }

    let mut tx = pool.begin().await?;

    DbUser::delete_avatar(&user.id, &mut *tx).await?;

    DbUser::edit_avatar(&user.id, &file_name, &file_blob, &mime_type, &mut *tx).await?;
    tx.commit().await?;
    Ok(ApiResult { result: ResultStatus::Success, details: "changed avatar".to_string() })
}

#[server(name=GetAvatarId, prefix="/api/user", endpoint="avatar_id")]
#[instrument]
pub async fn get_avatar_id(identifier: UserIdentifier) -> Result<Option<String>, AppError> {
    let (_, pool) = authenticated_check().await?;

    let id = DbUser::get_avatar_id(&identifier, &pool).await?;
    Ok(id)
}

#[server(name=GetAttachmentId, prefix="/api", endpoint="attachment_id")]
#[instrument]
pub async fn get_attachment_id(identifier: AttachmentIdentifier) -> Result<Option<String>, AppError> {
    let (_, pool) = authenticated_check().await?;

    let id = AttachmentWithoutBlob::get_id(&identifier, &pool).await?;
    Ok(id)
}

#[server(name=GetAllIllustrations, prefix="/api", endpoint="get_all_illustrations", input=GetUrl)]
#[instrument]
pub async fn get_all_illustrations() -> Result<Vec<AttachmentWithoutBlob>, AppError> {
    let (_, pool) = authenticated_check().await?;

    let illustrations = AttachmentWithoutBlob::get_all_illustrations(&pool).await?;
    Ok(illustrations)
}

#[server(name=GetIllustrationId, prefix="/api", endpoint="illustration_id")]
#[instrument]
pub async fn get_illustration_id(identifier: AttachmentIdentifier) -> Result<Option<String>, AppError> {
    let (_, pool) = authenticated_check().await?;

    let id = AttachmentWithoutBlob::get_illustration_id(&identifier, &pool).await?;
    Ok(id)
}

#[server(name=SolvedChallenges, prefix="/api/challenges", endpoint="solved", input=GetUrl)]
#[instrument]
pub async fn get_user_solved_challenges() -> Result<Vec<String>, AppError> {
    let (user, pool) = authenticated_check().await?;

    let solved = db::structs::Submission::get_user_solved_challenges(&user.id, &pool).await?;
    Ok(solved)
}

#[server(name=GetActiveEvents, prefix="/api", endpoint="active_events", input=GetUrl)]
#[instrument]
pub async fn get_active_events() -> Result<Vec<Event>, AppError> {
    let auth = get_context::<AuthSession>()?;
    let events = db::structs::Event::get_all(&auth.backend.pool).await?;

    let mut active_events = Vec::new();
    let now = chrono::Local::now();
    for event in events.into_iter() {
        if now >= event.start_at && now <= event.end_at {
            active_events.push(event);
        } 
    }

    Ok(active_events)
}

#[server(name=EditPassword, prefix="/api/user", endpoint="password")]
#[instrument(skip(old_password, new_password, confirm_new_password))]
pub async fn edit_password(old_password: Zeroizing<String>, new_password: Zeroizing<String>, confirm_new_password: Zeroizing<String>) -> Result<ApiResult<String>, AppError> {
    let (user, pool) = authenticated_check().await?;

    let old_password_hash = DbUser::get_password_hash(&UserIdentifier::Id(user.id.clone()), &pool).await?;
    if verify_hash(&old_password, &old_password_hash).await.is_err() {
        return Err(AppError::BadRequest("old password does not match current password".to_string()));
    }

    if new_password != confirm_new_password {
        return Err(AppError::BadRequest("new password and confirm new password must match".to_string()));
    }

    if old_password == new_password {
        return Ok(ApiResult { result: ResultStatus::Fail, details: "new password is same as old password".to_string() });
    }

    let pw_hash = hash_string(&new_password).await?;
    DbUser::edit_password(&user.id, &pw_hash, &pool).await?;
    Ok(ApiResult { result: ResultStatus::Success, details: "changed password".to_string() })

    // match crate::server::proxmox::change_user_password(db_user, new_password).await {
    //     Ok(_) => Ok(ApiResult { result: ResultStatus::Success, details: "changed password".to_string() }),
    //     Err(e) => Err(e.into())
    // }
}

#[server(name=UserExists, prefix="/api",endpoint="user_exists")]
pub async fn user_exists(email: String) -> Result<bool, AppError> {
    let auth = get_context::<AuthSession>()?;

    let result = DbUser::is_user_available(&email, &auth.backend.pool).await?;
    Ok(result)
}

#[server(name=LogoutUser, prefix="/api",endpoint="logout")]
pub async fn logout_user() -> Result<(), AppError> {
    let mut auth = get_context::<AuthSession>()?;

    auth.logout().await?;
    leptos_axum::redirect("/");
    Ok(())
}

#[server(name=IsLdapEnabled, prefix="/api", endpoint="ldap_enabled", input=GetUrl)]
#[instrument]
pub async fn is_ldap_enabled() -> Result<bool, AppError> {
    let auth = get_context::<AuthSession>()?;
    
    let enabled = LdapArgs::get_status(&auth.backend.pool).await?;
    Ok(enabled)
}

#[server(name=StartVM, prefix="/api", endpoint="start_vm")]
#[instrument]
pub async fn start_vm(template_id: u32, challenge_id: String) -> Result<ApiResult<String>, AppError> {
    let (user, pool) = authenticated_check().await?;
    let db_user = get_db_user(&user, &pool).await?;

    let challenge = db::structs::Challenge::get(&challenge_id, &pool).await.or_not_found(
        AppError::BadRequest("Invalid challenge id".to_string())
    )?;
    let template_ids = challenge.vm_ids.as_deref().map(parse_vm_ids).unwrap_or_default();
    if !template_ids.contains(&template_id) {
        return Err(AppError::BadRequest("Invalid template id".to_string()));
    }

    let vm_id = crate::server::proxmox::start_vm(&template_id, &challenge_id, &db_user).await?;
    Ok(ApiResult { result: ResultStatus::Success, details: format!("Successfully started VM (ID: {vm_id})") })
}

#[server(name=RestartVM, prefix="/api", endpoint="restart_vm")]
#[instrument]
pub async fn restart_vm(template_id: u32) -> Result<ApiResult<String>, AppError> {
    let (user, pool) = authenticated_check().await?;
    let db_user = get_db_user(&user, &pool).await?;

    let vm_id = crate::server::proxmox::restart_vm(&db_user, &template_id).await?;
    Ok(ApiResult { result: ResultStatus::Success, details: format!("Successfully restarted VM (ID: {vm_id})") })
}

#[server(name=DestroyVM, prefix="/api", endpoint="destroy_vm")]
#[instrument]
pub async fn destroy_vm(template_id: u32) -> Result<ApiResult<String>, AppError> {
    let (user, pool) = authenticated_check().await?;
    let db_user = get_db_user(&user, &pool).await?;

    let vm_id = crate::server::proxmox::destroy_vm(&db_user, &template_id).await?;
    Ok(ApiResult { result: ResultStatus::Success, details: format!("Successfully destroyed VM (ID: {vm_id})") })
}

#[server(name=AddVMTime, prefix="/api", endpoint="add_vm_time")]
#[instrument]
pub async fn add_vm_time(template_id: u32) -> Result<ApiResult<String>, AppError> {
    let (user, pool) = authenticated_check().await?;
    let db_user = get_db_user(&user, &pool).await?;
    
    let vm_id = crate::server::proxmox::add_vm_time(&db_user, &template_id).await?;
    Ok(ApiResult { result: ResultStatus::Success, details: format!("Successfully added time to VM (ID: {vm_id})") })
}

#[server(name=GetUserActiveVMs, prefix="/api", endpoint="get_active_vms", input=GetUrl)]
#[instrument]
pub async fn get_user_active_vms() -> Result<Vec<ProxmoxVMInstance>, AppError> {
    let (user, pool) = authenticated_check().await?;
    let db_user = get_db_user(&user, &pool).await?;

    let vms = match crate::server::proxmox::get_user_vms(&db_user).await {
        Ok(vms) => vms,
        Err(e) => return Err(e)
    };

    let mut active_vms = Vec::<ProxmoxVMInstance>::new();
    for vm in vms {
        if vm.running { active_vms.push(vm) }
    }
    Ok(active_vms)
}

#[server(name=GetUserVMs, prefix="/api", endpoint="get_vms", input=GetUrl)]
#[instrument]
pub async fn get_user_vms() -> Result<Vec<ProxmoxVMInstance>, AppError> {
    let (user, pool) = authenticated_check().await?;
    let db_user = get_db_user(&user, &pool).await?;

    let vms = crate::server::proxmox::get_user_vms(&db_user).await?;
    Ok(vms)
}

#[server(name=GetUsedHints, prefix="/api/user", endpoint="hints_used", input=GetUrl)]
#[instrument]
pub async fn get_used_hints() -> Result<Vec<HintsUsed>, AppError> {
    let (user, pool) = authenticated_check().await?;
    let db_user = get_db_user(&user, &pool).await?;

    let hints_used = HintsUsed::get(&db_user, &pool).await?;
    Ok(hints_used)
}

#[server(name=GetHint, prefix="/api/challenge", endpoint="hint")]
#[instrument]
pub async fn get_hint(challenge_id: String, hint_id: String) -> Result<crate::server::db::structs::Hint, AppError> {
    let (user, pool) = authenticated_check().await?;
    let db_user = get_db_user(&user, &pool).await?;

    let mut tx = pool.begin().await?;

    let used_hints = HintsUsed::get(&db_user, &mut *tx).await?;
    let used_hint_ids = used_hints.into_iter().map(|h| h.hint_id).collect::<Vec<String>>();
    if used_hint_ids.contains(&hint_id) {
        tx.rollback().await?;
        let hint = crate::server::db::structs::Hint::get(&hint_id, &pool).await?;
        Ok(hint)
    } else {
        let hint = DbHint::get(&hint_id, &mut *tx).await.or_not_found(
            AppError::BadRequest("Invalid hint id".to_string())
        )?;
        if hint.challenge_id != challenge_id {
            return Err(AppError::BadRequest("Invalid hint id".to_string()));
        }
        
        if HintsUsed::add(&challenge_id, &user.id, &hint_id, &mut *tx).await.is_unique_violation()? {
            tx.rollback().await?;
            let hint = DbHint::get(&hint_id, &pool).await?;
            return Ok(hint);
        }

        let hint = crate::server::db::structs::Hint::get(&hint_id, &mut *tx).await?;
        db_user.deduct_points(&hint.points_penalty, &mut *tx).await?;
        tx.commit().await?;

        Ok(hint)
    }
}

#[server(name=GetAllHintsWithoutHints, prefix="/api/challenges", endpoint="hints", input=GetUrl)]
#[instrument]
pub async fn get_all_hints_without_hints() -> Result<Vec<HintWithoutHint>, AppError> {
    let (_, pool) = authenticated_check().await?;

    let hints = HintWithoutHint::get_all_hints(&pool).await?;
    Ok(hints)
}

#[server(name=GetChallengeHintsWithoutHints, prefix="/api/challenge", endpoint="hints")]
#[instrument]
pub async fn get_challenge_hints_without_hints(challenge_id: String) -> Result<Vec<HintWithoutHint>, AppError> {
    let (_, pool) = authenticated_check().await?;

    let hints = HintWithoutHint::get_challenge_hints(&challenge_id, &pool).await?;
    Ok(hints)
}

#[server(name=GetProxmoxBaseUrl, prefix="/api", endpoint="get_proxmox_url", input=GetUrl)]
#[instrument]
pub async fn get_proxmox_base_url() -> Result<String, AppError> {
    let (_, _) = authenticated_check().await?;

    let base_url = crate::server::proxmox::get_proxmox_base_url().await?;
    Ok(base_url)
}

#[server(name=GetTemplateInfo, prefix="/api", endpoint="get_template_info", input=GetUrl)]
#[instrument]
pub async fn get_all_templates() -> Result<Vec<ProxmoxVMTemplate>, AppError> {
    let (_, _) = authenticated_check().await?;

    let templates = crate::server::proxmox::get_all_templates().await?;
    Ok(templates)
}
