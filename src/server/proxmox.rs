#[cfg(feature = "ssr")]
use crate::server::{db::get_db_ref, is_host_reachable};
use crate::{error_template::AppError, server::db::{self, structs::{Challenge, DbUser, LdapArgs, ProxmoxArgs}}, utils::html_local_to_datetime};
use chrono::{DateTime, Local};
#[cfg(feature = "ssr")]
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time::Duration};
use tracing::instrument;

#[derive(Debug, Eq, PartialEq, Hash, Default, Clone, Deserialize, Serialize)]
pub struct ProxmoxVMTemplate {
    pub id: u32,
    pub name: String
}

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct ProxmoxVMInstance {
    pub id: u32,
    pub challenge_id: String,
    pub origin_id: u32,
    pub user_id: String,
    pub created_at: DateTime<Local>,
    pub end_at: DateTime<Local>,
    pub running: bool
}

#[derive(Deserialize)]
struct ProxmoxApiResponse<T> {
    data: T
}

#[derive(Serialize, Deserialize)]
struct Domains {
    realm: String,
    r#type: String
}

#[derive(Deserialize)]
struct Vmid {
    data: String
}

#[derive(Serialize, Deserialize)]
struct Members {
    members: Vec<Member>,
    poolid: Option<String>
}
#[derive(Serialize, Deserialize)]
struct Member {
    name: Option<String>,
    vmid: Option<u32>,
    status: Option<String>
}

#[derive(Serialize, Deserialize)]
struct Pools {
    poolid: String,
    r#type: Option<String>
}

#[derive(Serialize, Deserialize)]
struct Config {
    description: Option<String>
}

#[derive(Deserialize)]
struct User {
    userid: Option<String>
}

#[derive(Deserialize, PartialEq)]
struct Role {
    roleid: Option<String>
}

#[cfg(feature = "ssr")]
#[instrument]
pub async fn create_realm() -> Result<(), AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let proxmox_args = get_proxmox_args().await?;
            is_host_reachable(proxmox_args.base_url.clone()).await?;

            let client = Client::builder()
                .danger_accept_invalid_certs(true)
                .cookie_store(true)
                .build()?;

            let base_url = proxmox_args.base_url.trim_end_matches("/");
            let api_path = proxmox_args.api_path.trim_start_matches("/").trim_end_matches("/");
            let sync_url = format!("{base_url}/{api_path}/access/domains/CTFPKHK/sync");
            let url = format!("{base_url}/{api_path}/access/domains");

            let auth_value = format!("PVEAPIToken={}", proxmox_args.api_token.unwrap_or_default());
            match client.get(url.clone()).header(header::AUTHORIZATION, auth_value.clone()).send().await {
                Ok(res) => {
                    let domains = res.json::<ProxmoxApiResponse<Vec<Domains>>>().await?;
                    for domain in domains.data {
                        if domain.realm.contains("CTFPKHK") {
                            return Ok(())
                        }
                    }
                },
                Err(e) => return Err(e.into())
            };

            let ldap_args = match LdapArgs::get(get_db_ref()).await {
                Ok(res) => if res.is_some() { res.unwrap_or_default() } else { return Err(AppError::InternalError("LDAP not configured".to_string())) },
                Err(e) => return Err(e.into())
            };

            let ldap_url = url::Url::parse(&ldap_args.url)?;
            let body = serde_urlencoded::to_string(&[
                ("type", "ldap"), 
                ("realm", "CTFPKHK"),
                ("mode", "ldap"), // in the future use ldaps
                ("server1", ldap_url.host_str().unwrap_or_default()),
                ("base_dn", ldap_args.base_dn.as_str()),
                ("bind_dn", ldap_args.bind_dn.as_str()),
                ("user_attr", "sAMAccountName"),
                ("password", ldap_args.bind_pw.as_str()),
                ("verify", "0"),
            ]).unwrap_or_default();

            let resp = client
                .post(&url)
                .header(header::AUTHORIZATION, auth_value.clone())
                .body(body)
                .send()
                .await?;

            let body = serde_urlencoded::to_string(&[
                ("scope", "users"),
                ("enable-new", "1"),
                ("remove-vanished", "acl;entry;properties")
            ]).unwrap_or_default();

            if resp.status().is_success() {
                match client.post(sync_url.clone()).header(header::AUTHORIZATION, auth_value).body(body).send().await {
                    Ok(res) => {
                        if res.status().is_success() {
                            Ok(())
                        } else {
                            Err(AppError::InternalError("failed to create realm for Proxmox".to_string()))
                        }
                    },
                    Err(e) => return Err(e.into())
                }
            } else {
                Err(AppError::InternalError("".to_string()))
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[cfg(feature = "ssr")]
#[instrument]
pub async fn sync_realm() -> Result<(), AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let proxmox_args = get_proxmox_args().await?;
            is_host_reachable(proxmox_args.base_url.clone()).await?;

            let client = Client::builder()
                .danger_accept_invalid_certs(true)
                .cookie_store(true)
                .build()?;

            let auth_value = format!("PVEAPIToken={}", proxmox_args.api_token.unwrap_or_default());
            let base_url = proxmox_args.base_url.trim_end_matches("/");
            let api_path = proxmox_args.api_path.trim_start_matches("/").trim_end_matches("/");
            let sync_url = format!("{base_url}/{api_path}/access/domains/CTFPKHK/sync");

            let body = serde_urlencoded::to_string(&[
                ("realm", "CTFPKHK"),
                ("scope", "users"),
                ("enable-new", "1"),
                ("remove-vanished", "acl;entry;properties")
            ]).unwrap_or_default();

            match client.post(sync_url.clone()).header(header::AUTHORIZATION, auth_value).body(body).send().await {
                Ok(res) => {
                    if res.status().is_success() {
                        Ok(())
                    } else {
                        Err(AppError::InternalError("failed to sync realm for Proxmox".to_string()))
                    }
                },
                Err(e) => return Err(e.into())
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[cfg(feature = "ssr")]
#[instrument]
async fn get_next_free_vm_id() -> Result<u32, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let client = Client::builder()
                .danger_accept_invalid_certs(true)
                .build()?;

            let proxmox_args = get_proxmox_args().await?;
            let base_url = proxmox_args.base_url.trim_end_matches("/");
            let api_path = proxmox_args.api_path.trim_start_matches("/").trim_end_matches("/");
            let url = format!("{base_url}/{api_path}/cluster/nextid");
            let auth_value = format!("PVEAPIToken={}", proxmox_args.api_token.unwrap_or_default());

            match client.get(url.clone()).header(header::AUTHORIZATION, auth_value).send().await {
                Ok(res) => {
                    let next_free_vm_id = res.json::<Vmid>().await?;
                    let next_free_vm_id = next_free_vm_id.data.parse::<u32>()?;
                    Ok(next_free_vm_id)
                },
                Err(e) => return Err(e.into())
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[cfg(feature = "ssr")]
#[instrument]
pub async fn start_vm(template_id: u32, challenge: Challenge, user: DbUser) -> Result<(), AppError> {
    let proxmox_args = get_proxmox_args().await?;
    is_host_reachable(proxmox_args.base_url.clone()).await?;

    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?;

    let auth_value = format!("PVEAPIToken={}", proxmox_args.api_token.unwrap_or_default());
    let base_url = proxmox_args.base_url.trim_end_matches("/");
    let api_path = proxmox_args.api_path.trim_start_matches("/").trim_end_matches("/");
    let poolid = format!("CTFPKHK-{}", user.username);
    match get_user_vmid_from_template_id(user.clone(), template_id).await? {
        Some(vm_id) => {
            let start_url = format!("{base_url}/{api_path}/nodes/{}/qemu/{}/status/start", proxmox_args.node, vm_id);
            // start
            match client.post(start_url).header(header::AUTHORIZATION, auth_value.clone()).send().await {
                Ok(_) => Ok(()),
                Err(e) => return Err(e.into())
            }
        },
        None => {
            let new_vm_id = clone_vm(template_id, challenge, user.clone()).await?;
            let start_url = format!("{base_url}/{api_path}/nodes/{}/qemu/{}/status/start", proxmox_args.node, new_vm_id);

            // start
            match client.post(start_url).header(header::AUTHORIZATION, auth_value.clone()).send().await {
                Ok(_) => {
                    _ = schedule_vm_deletion(user, new_vm_id).await;
                    let pools_url = format!("{base_url}/{api_path}/pools/{poolid}");
                    loop {
                        async_std::task::sleep(Duration::from_secs(1)).await;
                        let vms = match client.get(pools_url.clone()).header(header::AUTHORIZATION, auth_value.clone()).send().await {
                            Ok(res) => {
                                res.json::<ProxmoxApiResponse<Members>>().await?
                            },
                            Err(e) => return Err(e.into())
                        };

                        for vm in vms.data.members {
                            let vm_status = vm.status.unwrap_or_default();
                            let vm_id = vm.vmid.unwrap_or_default();
                            if vm_status == "running" && vm_id == new_vm_id { return Ok(()); } else { continue };
                        }
                    }
                },
                Err(e) => return Err(e.into())
            }
        }
    }
}

#[cfg(feature = "ssr")]
#[instrument]
async fn clone_vm(template_id: u32, challenge: Challenge, user: DbUser) -> Result<u32, AppError> {
    let proxmox_args = get_proxmox_args().await?;
    is_host_reachable(proxmox_args.base_url.clone()).await?;

    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?;

    let auth_value = format!("PVEAPIToken={}", proxmox_args.api_token.unwrap_or_default());
    let base_url = proxmox_args.base_url.trim_end_matches("/");
    let api_path = proxmox_args.api_path.trim_start_matches("/").trim_end_matches("/");

    let new_vm_id = get_next_free_vm_id().await?;
    let template_info = get_template_info(template_id).await?;
    
    let clone_body = serde_urlencoded::to_string(&[
        ("newid", new_vm_id.to_string()), 
        ("name", template_info.name),
        ("full", "1".to_string()), // 0 - linked clone, 1 - full clone
        ("target", proxmox_args.node.clone()),
        ("pool", format!("CTFPKHK-{}", user.username)),
    ]).unwrap_or_default();

    // clone
    let clone_url = format!("{base_url}/{api_path}/nodes/{}/qemu/{}/clone", proxmox_args.node, template_id.clone());
    if let Err(e) = client.post(clone_url).header(header::AUTHORIZATION, auth_value.clone()).body(clone_body).send().await {
        return Err(e.into())
    }

    let created_at = Local::now();
    let expire_at = created_at + chrono::Duration::hours(1);
    let vm_description = format!(
        "id={new_vm_id}&challenge_id={}&origin_id={}&user_id={}&created_at={}&expire_at={}", 
        challenge.id.clone(), 
        template_id, 
        user.id.clone(), 
        created_at.to_string(), 
        expire_at.to_string()
    );

    let conf_url = format!("{base_url}/{api_path}/nodes/{}/qemu/{}/config", proxmox_args.node, new_vm_id);
    let conf_body = serde_urlencoded::to_string(&[
        ("description", vm_description), 
    ]).unwrap_or_default();

    // update config
    match client.post(conf_url).header(header::AUTHORIZATION, auth_value.clone()).body(conf_body).send().await {
        Ok(_) => Ok(new_vm_id),
        Err(e) => return Err(e.into())
    }
}

// return Ok only when vm status shows running again
#[cfg(feature = "ssr")]
#[instrument]
pub async fn restart_vm(user: DbUser, template_id: u32) -> Result<(), AppError> {
    let proxmox_args = get_proxmox_args().await?;
    is_host_reachable(proxmox_args.base_url.clone()).await?;

    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?;

    let base_url = proxmox_args.base_url.trim_end_matches("/");
    let api_path = proxmox_args.api_path.trim_start_matches("/").trim_end_matches("/");
    let vm_id = match get_user_vmid_from_template_id(user, template_id).await? {
        Some(vm_id) => vm_id,
        None =>  return Err(AppError::InternalError("".to_string()))
    };
    let url = format!("{base_url}/{api_path}/nodes/{}/qemu/{vm_id}/status/reboot", proxmox_args.node);
    let auth_value = format!("PVEAPIToken={}", proxmox_args.api_token.unwrap_or_default());

    match client.post(url).header(header::AUTHORIZATION, auth_value).send().await {
        Ok(_) => Ok(()),
        Err(e) => Err(e.into())
    }
}

// return Ok only when vm with template id doesn't exist in user pool anymore
#[cfg(feature = "ssr")]
#[instrument]
pub async fn destroy_vm(user: DbUser, template_id: u32) -> Result<(), AppError> {
    let proxmox_args = get_proxmox_args().await?;
    is_host_reachable(proxmox_args.base_url.clone()).await?;

    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?;

    let base_url = proxmox_args.base_url.trim_end_matches("/");
    let api_path = proxmox_args.api_path.trim_start_matches("/").trim_end_matches("/");
    let vm_id = match get_user_vmid_from_template_id(user, template_id).await? {
        Some(vm_id) => vm_id,
        None =>  return Err(AppError::InternalError("".to_string()))
    };
    let destroy_url = format!("{base_url}/{api_path}/nodes/{}/qemu/{vm_id}", proxmox_args.node);
    let stop_url = format!("{base_url}/{api_path}/nodes/{}/qemu/{vm_id}/status/stop", proxmox_args.node);
    let auth_value = format!("PVEAPIToken={}", proxmox_args.api_token.unwrap_or_default());
    
    // stop
    match client.post(stop_url).header(header::AUTHORIZATION, auth_value.clone()).send().await {
        Ok(res) => {
            if res.status().is_success() {
                async_std::task::sleep(Duration::from_secs(3)).await;
                // destroy
                match client.delete(destroy_url).header(header::AUTHORIZATION, auth_value).send().await {
                    Ok(res) => {
                        leptos::logging::log!("delete res: {}", res.text().await?);
                        Ok(())
                    },
                    Err(e) => Err(e.into())
                }
            } else {
                Err(AppError::InternalError("failed to stop vm".to_string()))
            }
        },
        Err(e) => return Err(e.into())
    }
}

#[cfg(feature = "ssr")]
#[instrument]
pub async fn create_user_pool(user: DbUser) -> Result<(), AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let proxmox_args = get_proxmox_args().await?;
            is_host_reachable(proxmox_args.base_url.clone()).await?;

            let client = Client::builder()
                .danger_accept_invalid_certs(true)
                .build()?;

            let base_url = proxmox_args.base_url.trim_end_matches("/");
            let api_path = proxmox_args.api_path.trim_start_matches("/").trim_end_matches("/");
            let pools_url = format!("{base_url}/{api_path}/pools");
            let acl_url = format!("{base_url}/{api_path}/access/acl");
            let poolid = format!("CTFPKHK-{}", user.username);
            let auth_value = format!("PVEAPIToken={}", proxmox_args.api_token.unwrap_or_default());

            match client.get(pools_url.clone()).header(header::AUTHORIZATION, auth_value.clone()).send().await {
                Ok(res) => {
                    let pools = res.json::<ProxmoxApiResponse<Vec<Pools>>>().await?;
                    for pool in pools.data {
                        if pool.poolid.contains(&poolid) {
                            return Ok(());
                        }
                    }
                },
                Err(e) => return Err(e.into())
            };

            let body = serde_urlencoded::to_string(&[("poolid", poolid.clone())]).unwrap_or_default();
            client.post(&pools_url)
                .header(header::AUTHORIZATION, auth_value.clone())
                .body(body)
                .send()
                .await?;

            if user.auth_type == "ldap" {
                let body = serde_urlencoded::to_string(&[
                    ("path", format!("/pool/{poolid}")),
                    ("users", format!("{}@CTFPKHK", user.username)),
                    ("roles", "CTFCompetitor".to_string()),
                    ("propagate", "1".to_string())
                ]).unwrap_or_default();
                client.put(&acl_url)
                    .header(header::AUTHORIZATION, auth_value)
                    .body(body)
                    .send()
                    .await?;
            } else {
                let acl_body = serde_urlencoded::to_string(&[
                    ("path", format!("/pool/{poolid}")),
                    ("users", format!("{}@pve", user.username)),
                    ("roles", "CTFCompetitor".to_string()),
                    ("propagate", "1".to_string())
                ]).unwrap_or_default();
                client.put(&acl_url)
                    .header(header::AUTHORIZATION, auth_value)
                    .body(acl_body)
                    .send()
                    .await?;
            }

            Ok(())
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[cfg(feature = "ssr")]
#[instrument]
pub async fn test_auth(args: ProxmoxArgs) -> Result<(), AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            is_host_reachable(args.base_url.clone()).await?;

            let client = Client::builder()
                .danger_accept_invalid_certs(true)
                .cookie_store(true)
                .build()?;

            let base_url = args.base_url.trim_end_matches("/");
            let api_path = args.api_path.trim_start_matches("/").trim_end_matches("/");
            let url = format!("{base_url}/{api_path}/nodes/status");
            let auth_value = format!("PVEAPIToken={}", args.api_token.unwrap_or_default());

            match client.get(url.clone()).header(header::AUTHORIZATION, auth_value.clone()).send().await {
                Ok(res) => if res.status().is_success() { Ok(()) } else { Err(AppError::Unauthorized) },
                Err(e) => return Err(e.into())
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[cfg(feature = "ssr")]
#[instrument]
async fn schedule_vm_deletion(user: DbUser, vm_id: u32) -> Result<(), AppError> {
    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?;

    let proxmox_args = get_proxmox_args().await?;
    is_host_reachable(proxmox_args.base_url.clone()).await?;

    let base_url = proxmox_args.base_url.trim_end_matches("/");
    let api_path = proxmox_args.api_path.trim_start_matches("/").trim_end_matches("/");
    let conf_url = format!("{base_url}/{api_path}/nodes/{}/qemu/{}/config", proxmox_args.node.clone(), vm_id);
    let auth_value = format!("PVEAPIToken={}", proxmox_args.api_token.unwrap_or_default());

    let handle = tokio::spawn(async move {
        let user = user.clone();
        let mut intv = tokio::time::interval(Duration::from_secs(60 * 30));
        loop {
            let config = match client.get(conf_url.clone()).header(header::AUTHORIZATION, auth_value.clone()).send().await {
                Ok(res) => {
                    res.json::<ProxmoxApiResponse<Config>>().await?
                },
                Err(e) => return Err(e.into())
            };
            let description = config.data.description.unwrap_or_default();
            let args = extract_args_from_description(description).await?;

            let end_at = args.end_at.timestamp();
            let now = Local::now().timestamp();
            if now >= end_at {
                return destroy_vm(user, vm_id).await;
            }

            intv.tick().await;
        }
    });

    match handle.await {
        Ok(result) => result,
        Err(e) => Err(e.into()),
    }
}

#[cfg(feature = "ssr")]
#[instrument]
pub async fn add_vm_time(user: DbUser, template_id: u32) -> Result<(), AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let proxmox_args = get_proxmox_args().await?;
            is_host_reachable(proxmox_args.base_url.clone()).await?;

            let client = Client::builder()
                .danger_accept_invalid_certs(true)
                .build()?;

            let base_url = proxmox_args.base_url.trim_end_matches("/");
            let api_path = proxmox_args.api_path.trim_start_matches("/").trim_end_matches("/");
            let vm_id = match get_user_vmid_from_template_id(user, template_id).await? {
                Some(vm_id) => vm_id,
                None =>  return Err(AppError::InternalError("".to_string()))
            };
            let conf_url = format!("{base_url}/{api_path}/nodes/{}/qemu/{}/config", proxmox_args.node.clone(), vm_id);
            let auth_value = format!("PVEAPIToken={}", proxmox_args.api_token.unwrap_or_default());

            let config = match client.get(conf_url.clone()).header(header::AUTHORIZATION, auth_value.clone()).send().await {
                Ok(res) => res.json::<ProxmoxApiResponse<Config>>().await?,
                Err(e) => return Err(e.into())
            };
            let description = config.data.description.unwrap_or_default();
            let args = extract_args_from_description(description).await?;
            let new_expire_at = args.end_at + chrono::Duration::minutes(30);
            
            let new_description = format!(
                "id={vm_id}&challenge_id={}&origin_id={template_id}&user_id={}&created_at={}&expire_at={}", 
                args.challenge_id, 
                args.user_id, 
                args.created_at.to_string(), 
                new_expire_at.to_string()
            );

            let conf_body = serde_urlencoded::to_string(&[
                ("description", new_description), 
            ]).unwrap_or_default();

            // update config
            if let Err(e) = client.post(conf_url).header(header::AUTHORIZATION, auth_value.clone()).body(conf_body).send().await {
                return Err(e.into())
            }

            Ok(())
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[cfg(feature = "ssr")]
#[instrument]
pub async fn get_user_vms(user: DbUser) -> Result<Vec<ProxmoxVMInstance>, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let proxmox_args = get_proxmox_args().await?;
            is_host_reachable(proxmox_args.base_url.clone()).await?;

            let client = Client::builder()
                .danger_accept_invalid_certs(true)
                .build()?;

            let base_url = proxmox_args.base_url.trim_end_matches("/");
            let api_path = proxmox_args.api_path.trim_start_matches("/").trim_end_matches("/");
            let auth_value = format!("PVEAPIToken={}", proxmox_args.api_token.unwrap_or_default());
            let poolid = format!("CTFPKHK-{}", user.username);
            let url = format!("{base_url}/{api_path}/pools/{poolid}");

            let vms = match client.get(url.clone()).header(header::AUTHORIZATION, auth_value.clone()).send().await {
                Ok(res) => {
                    res.json::<ProxmoxApiResponse<Members>>().await?
                },
                Err(e) => return Err(e.into())
            };

            let mut user_vms = Vec::<ProxmoxVMInstance>::new();
            for vm in vms.data.members {
                let vm_status = vm.status.unwrap_or_default();
                let conf_url = format!("{base_url}/{api_path}/nodes/{}/qemu/{}/config", proxmox_args.node.clone(), vm.vmid.unwrap_or_default());
                let config = match client.get(conf_url.clone()).header(header::AUTHORIZATION, auth_value.clone()).send().await {
                    Ok(res) => res.json::<ProxmoxApiResponse<Config>>().await?,
                    Err(e) => return Err(e.into())
                };
                let description = config.data.description.unwrap_or_default();
                let mut args = extract_args_from_description(description).await?;
                args.running = if vm_status == "running" { true } else { false };

                user_vms.push(args);
            }
            Ok(user_vms)
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[cfg(feature = "ssr")]
#[instrument]
async fn extract_args_from_description(desc: String) -> Result<ProxmoxVMInstance, AppError> {
    let params: HashMap<String, String> = url::form_urlencoded::parse(desc.as_bytes()).into_owned().collect();

    let id = params.get("id").cloned().unwrap_or_default().parse::<u32>().unwrap_or_default();
    let challenge_id = params.get("challenge_id").cloned().unwrap_or_default();
    let origin_id = params.get("origin_id").cloned().unwrap_or_default().parse::<u32>().unwrap_or_default();
    let user_id = params.get("user_id").cloned().unwrap_or_default();
    let created_at = params.get("created_at").cloned().unwrap_or_default();
    let end_at = params.get("expire_at").cloned().unwrap_or_default();

    let created_at = html_local_to_datetime(created_at);
    let end_at = html_local_to_datetime(end_at);

    Ok(ProxmoxVMInstance { id, challenge_id, origin_id, user_id, created_at, end_at, running: false })
}

#[cfg(feature = "ssr")]
#[instrument]
pub async fn get_all_templates() -> Result<Vec<ProxmoxVMTemplate>, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let proxmox_args = get_proxmox_args().await?;
            is_host_reachable(proxmox_args.base_url.clone()).await?;

            let client = Client::builder()
                .danger_accept_invalid_certs(true)
                .build()?;

            let base_url = proxmox_args.base_url.trim_end_matches("/");
            let api_path = proxmox_args.api_path.trim_start_matches("/").trim_end_matches("/");
            let auth_value = format!("PVEAPIToken={}", proxmox_args.api_token.unwrap_or_default());
            let url = format!("{base_url}/{api_path}/pools/{}", proxmox_args.templates_pool_id);

            let vms = match client.get(url.clone()).header(header::AUTHORIZATION, auth_value.clone()).send().await {
                Ok(res) => {
                    res.json::<ProxmoxApiResponse<Members>>().await?
                },
                Err(e) => return Err(e.into())
            };

            let mut templates = Vec::<ProxmoxVMTemplate>::new();
            for vm in vms.data.members {
                templates.push(ProxmoxVMTemplate { id: vm.vmid.unwrap_or_default(), name: vm.name.unwrap_or_default() });
            }
            Ok(templates)
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[cfg(feature = "ssr")]
#[instrument]
async fn get_user_vmid_from_template_id(user: DbUser, template_id: u32) -> Result<Option<u32>, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let user_vms = get_user_vms(user).await?;
            let mut vm_id = None;
            for user_vm in user_vms {
                if user_vm.origin_id == template_id {
                    vm_id = Some(user_vm.id);
                    break;
                }
            }
            Ok(vm_id)
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[cfg(feature = "ssr")]
#[instrument]
pub async fn create_user(email: String, username: String, password: String) -> Result<(), AppError> {
    let proxmox_args = get_proxmox_args().await?;
    is_host_reachable(proxmox_args.base_url.clone()).await?;

    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?;

    let auth_value = format!("PVEAPIToken={}", proxmox_args.api_token.unwrap_or_default());
    let base_url = proxmox_args.base_url.trim_end_matches("/");
    let api_path = proxmox_args.api_path.trim_start_matches("/").trim_end_matches("/");
    let url = format!("{base_url}/{api_path}/access/users");

    match client.get(url.clone()).header(header::AUTHORIZATION, auth_value.clone()).send().await {
        Ok(res) => {
            let users = res.json::<ProxmoxApiResponse<Vec<User>>>().await?;
            let userid = format!("{}@pve", username);
            for user in users.data {
                if user.userid.unwrap_or_default() == userid {
                    return Ok(());
                }
            }
        },
        Err(e) => return Err(e.into())
    };

    let body = serde_urlencoded::to_string(&[
        ("userid", format!("{}@pve", username)),
        ("password", password),
        ("expire", 0.to_string()),
        ("enable", 1.to_string()),
        ("email", email),
    ]).unwrap_or_default();

    match client.post(&url).header(header::AUTHORIZATION, auth_value).body(body).send().await {
        Ok(_) => Ok(()),
        Err(e) => return Err(e.into())
    }
}

// {"message":"Permission check failed (URI '/access/password' not available with API token, need proper ticket.\n)\n","data":null}
// #[cfg(feature = "ssr")]
// #[instrument]
// pub async fn change_user_password(user: DbUser, password: String) -> Result<(), AppError> {
//     match is_host_reachable().await {
//         Ok(reachable) => if reachable {} else { return Err(AppError::NetworkError("proxmox host not reachable".to_string())) },
//         Err(_) => return Err(AppError::NetworkError("proxmox host not reachable".to_string()))
//     }

//     let client = Client::builder()
//         .danger_accept_invalid_certs(true)
//         .build()?;

//     let proxmox_args = match db::structs::ProxmoxArgs::get(get_db_ref()).await {
//         Ok(res) => if res.is_some() { res.unwrap_or_default() } else { return Err(AppError::InternalError("missing proxmox args".to_string())) },
//         Err(e) => return Err(e.into())
//     };
//     let auth_value = format!("PVEAPIToken={}", proxmox_args.api_token.unwrap_or_default());

//     let base_url = proxmox_args.base_url.trim_end_matches("/");
//     let api_path = proxmox_args.api_path.trim_start_matches("/").trim_end_matches("/");
//     let url = format!("{base_url}/{api_path}/access/password");

//     let body = serde_urlencoded::to_string(&[
//         ("userid", format!("{}@pve", user.username)),
//         ("password", password),
//     ]).unwrap_or_default();

//     match client.put(&url).header(header::AUTHORIZATION, auth_value).body(body).send().await {
//         Ok(res) => {leptos::logging::log!("res is: {}", res.text().await?); Ok(())},
//         Err(e) => return Err(e.into())
//     }
// }

#[cfg(feature = "ssr")]
#[instrument]
async fn get_roles() -> Result<Vec<Role>, AppError> {
    let proxmox_args = get_proxmox_args().await?;
    is_host_reachable(proxmox_args.base_url.clone()).await?;

    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?;

    let auth_value = format!("PVEAPIToken={}", proxmox_args.api_token.unwrap_or_default());
    let base_url = proxmox_args.base_url.trim_end_matches("/");
    let api_path = proxmox_args.api_path.trim_start_matches("/").trim_end_matches("/");
    let url = format!("{base_url}/{api_path}/access/roles");

    match client.get(url).header(header::AUTHORIZATION, auth_value.clone()).send().await {
        Ok(res) => {
            let roles = res.json::<ProxmoxApiResponse<Vec<Role>>>().await?;
            Ok(roles.data)
        }
        Err(e) => Err(e.into())
    }
}

#[cfg(feature = "ssr")]
#[instrument]
pub async fn create_user_role() -> Result<(), AppError> {
    let proxmox_args = get_proxmox_args().await?;
    is_host_reachable(proxmox_args.base_url.clone()).await?;

    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?;

    let auth_value = format!("PVEAPIToken={}", proxmox_args.api_token.unwrap_or_default());
    let base_url = proxmox_args.base_url.trim_end_matches("/");
    let api_path = proxmox_args.api_path.trim_start_matches("/").trim_end_matches("/");
    let url = format!("{base_url}/{api_path}/access/roles");

    let roles = get_roles().await?;
    let roleid = "CTFCompetitor";

    if roles.contains(&Role { roleid: Some(roleid.to_string()) }) {
        return Ok(());
    }

    let body = serde_urlencoded::to_string(&[
        ("roleid", roleid), 
        ("privs", "VM.Audit"), 
        ("privs", "VM.Console"), 
        // ("privs", "VM.GuestAgent.Audit"), 
        // ("privs", "VM.GuestAgent.FileRead"), 
        // ("privs", "VM.GuestAgent.FileSystemMgmt"), 
        // ("privs", "VM.GuestAgent.FileWrite"), 
        ("privs", "VM.PowerMgmt"), 
        ("privs", "Pool.Audit"), 
    ]).unwrap_or_default();

    match client.post(url).header(header::AUTHORIZATION, auth_value.clone()).body(body).send().await {
        Ok(_) => Ok(()),
        Err(e) => return Err(e.into())
    }
}

#[cfg(feature = "ssr")]
#[instrument]
pub async fn get_template_info(template_id: u32) -> Result<ProxmoxVMTemplate, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let proxmox_args = get_proxmox_args().await?;
            is_host_reachable(proxmox_args.base_url.clone()).await?;

            let client = Client::builder()
                .danger_accept_invalid_certs(true)
                .build()?;

            let base_url = proxmox_args.base_url.trim_end_matches("/");
            let api_path = proxmox_args.api_path.trim_start_matches("/").trim_end_matches("/");
            let auth_value = format!("PVEAPIToken={}", proxmox_args.api_token.unwrap_or_default());
            let url = format!("{base_url}/{api_path}/pools/{}", proxmox_args.templates_pool_id);

            let vms = match client.get(url.clone()).header(header::AUTHORIZATION, auth_value.clone()).send().await {
                Ok(res) => {
                    res.json::<ProxmoxApiResponse<Members>>().await?
                },
                Err(e) => return Err(e.into())
            };

            let mut template_info = ProxmoxVMTemplate::default();
            for vm in vms.data.members {
                if vm.vmid.unwrap_or_default() == template_id {
                    template_info = ProxmoxVMTemplate { id: vm.vmid.unwrap_or_default(), name: vm.name.unwrap_or_default() };
                    break;
                }
            }
            Ok(template_info)
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[cfg(feature = "ssr")]
#[instrument]
async fn get_proxmox_args() -> Result<ProxmoxArgs, AppError> {
    match db::structs::ProxmoxArgs::get(get_db_ref()).await {
        Ok(res) => {
            if let Some(res) = res { 
                Ok(res)
            } else { 
                tracing::error!("failed to fetch proxmox args");
                Err(AppError::InternalError("internal error".to_string())) 
            }
        }
        Err(e) => {
            tracing::error!(error = ?e);
            Err(AppError::InternalError("internal error".to_string())) 
        }
    }
}
