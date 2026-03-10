#[cfg(feature = "ssr")]
use crate::server::{db::get_db_ref, is_host_reachable};
use crate::{error_template::AppError, server::db::{self, structs::{Challenge, DbUser, LdapArgs, ProxmoxArgs}}, utils::html_local_to_datetime};
use chrono::{DateTime, Local};
#[cfg(feature = "ssr")]
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use tokio::sync::OnceCell;
use std::{collections::HashMap, time::Duration};
use tracing::instrument;

#[cfg(feature = "ssr")]
static REQWEST_CLIENT: OnceCell<Client> = OnceCell::const_new();

#[cfg(feature = "ssr")]
pub fn init_reqwest_client() {
    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .cookie_store(true)
        .build()
        .expect("failed to build HTTP client");
    REQWEST_CLIENT.set(client).expect("HTTP client already initialized");
}

#[cfg(feature = "ssr")]
fn get_reqwest_client() -> &'static Client {
    REQWEST_CLIENT.get().expect("HTTP client not initialized")
}

#[derive(Debug, Eq, PartialEq, Hash, Default, Clone, Deserialize, Serialize)]
pub struct ProxmoxVMTemplate {
    pub id: u32,
    pub name: String
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
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
    let proxmox_args = get_proxmox_args().await?;
    is_host_reachable(&proxmox_args.base_url).await?;

    let client = get_reqwest_client();

    let base_url = proxmox_args.base_url.trim_end_matches("/");
    let api_path = proxmox_args.api_path.trim_start_matches("/").trim_end_matches("/");
    let sync_url = format!("{base_url}/{api_path}/access/domains/CTFPKHK/sync");
    let url = format!("{base_url}/{api_path}/access/domains");

    let auth_value = format!("PVEAPIToken={}", proxmox_args.api_token.unwrap_or_default());
    match client.get(&url).header(header::AUTHORIZATION, &auth_value).send().await {
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
        Ok(Some(res)) => res,
        Ok(None) => return Err(AppError::InternalError("LDAP not configured".to_string())),
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
        .header(header::AUTHORIZATION, &auth_value)
        .body(body)
        .send()
        .await?;

    let body = serde_urlencoded::to_string(&[
        ("scope", "users"),
        ("enable-new", "1"),
        ("remove-vanished", "acl;entry;properties")
    ]).unwrap_or_default();

    if resp.status().is_success() {
        match client.post(&sync_url).header(header::AUTHORIZATION, &auth_value).body(body).send().await {
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
}

#[cfg(feature = "ssr")]
#[instrument]
pub async fn sync_realm() -> Result<(), AppError> {
    let proxmox_args = get_proxmox_args().await?;
    is_host_reachable(&proxmox_args.base_url).await?;

    let client = get_reqwest_client();

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

    match client.post(&sync_url).header(header::AUTHORIZATION, &auth_value).body(body).send().await {
        Ok(res) => {
            if res.status().is_success() {
                Ok(())
            } else {
                Err(AppError::InternalError("failed to sync realm for Proxmox".to_string()))
            }
        },
        Err(e) => return Err(e.into())
    }
}

#[cfg(feature = "ssr")]
#[instrument]
async fn get_next_free_vm_id() -> Result<u32, AppError> {
    let client = get_reqwest_client();

    let proxmox_args = get_proxmox_args().await?;
    let base_url = proxmox_args.base_url.trim_end_matches("/");
    let api_path = proxmox_args.api_path.trim_start_matches("/").trim_end_matches("/");
    let url = format!("{base_url}/{api_path}/cluster/nextid");
    let auth_value = format!("PVEAPIToken={}", proxmox_args.api_token.unwrap_or_default());

    match client.get(&url).header(header::AUTHORIZATION, &auth_value).send().await {
        Ok(res) => {
            let next_free_vm_id = res.json::<Vmid>().await?;
            let next_free_vm_id = next_free_vm_id.data.parse::<u32>()?;
            Ok(next_free_vm_id)
        },
        Err(e) => return Err(e.into())
    }
}

#[cfg(feature = "ssr")]
#[instrument]
pub async fn start_vm(template_id: &u32, challenge: &Challenge, user: &DbUser) -> Result<u32, AppError> {
    let proxmox_args = get_proxmox_args().await?;
    is_host_reachable(&proxmox_args.base_url).await?;

    let client = get_reqwest_client();

    let auth_value = format!("PVEAPIToken={}", proxmox_args.api_token.unwrap_or_default());
    let base_url = proxmox_args.base_url.trim_end_matches("/");
    let api_path = proxmox_args.api_path.trim_start_matches("/").trim_end_matches("/");
    let poolid = format!("CTFPKHK-{}", user.username);
    match get_user_vmid_from_template_id(user, template_id).await? {
        Some(vm_id) => {
            let start_url = format!("{base_url}/{api_path}/nodes/{}/qemu/{}/status/start", proxmox_args.node, vm_id);
            // start
            match client.post(&start_url).header(header::AUTHORIZATION, &auth_value).send().await {
                Ok(_) => Ok(vm_id),
                Err(e) => return Err(e.into())
            }
        },
        None => {
            let new_vm_id = clone_vm(template_id, challenge, user).await?;
            let start_url = format!("{base_url}/{api_path}/nodes/{}/qemu/{}/status/start", proxmox_args.node, new_vm_id);

            // start
            match client.post(&start_url).header(header::AUTHORIZATION, &auth_value).send().await {
                Ok(_) => {
                    schedule_vm_deletion(user.clone(), new_vm_id);
                    let pools_url = format!("{base_url}/{api_path}/pools/{poolid}");
                    for _ in 0..60 {
                        tokio::time::sleep(Duration::from_secs(1)).await;
                        let vms = match client.get(&pools_url).header(header::AUTHORIZATION, &auth_value).send().await {
                            Ok(res) => {
                                res.json::<ProxmoxApiResponse<Members>>().await?
                            },
                            Err(e) => return Err(e.into())
                        };

                        for vm in vms.data.members {
                            let vm_status = vm.status.unwrap_or_default();
                            let vm_id = vm.vmid.unwrap_or_default();
                            if vm_status == "running" && vm_id == new_vm_id { return Ok(new_vm_id); } else { continue };
                        }
                    }
                    
                    return Err(AppError::InternalError("VM failed to start within timeout".to_string()));
                },
                Err(e) => return Err(e.into())
            }
        }
    }
}

#[cfg(feature = "ssr")]
#[instrument]
async fn clone_vm(template_id: &u32, challenge: &Challenge, user: &DbUser) -> Result<u32, AppError> {
    let proxmox_args = get_proxmox_args().await?;
    is_host_reachable(&proxmox_args.base_url).await?;

    let client = get_reqwest_client();

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
    let clone_url = format!("{base_url}/{api_path}/nodes/{}/qemu/{}/clone", &proxmox_args.node, &template_id);
    if let Err(e) = client.post(&clone_url).header(header::AUTHORIZATION, &auth_value).body(clone_body).send().await {
        return Err(e.into())
    }

    let created_at = Local::now();
    let expire_at = created_at + chrono::Duration::hours(1);
    let vm_description = format!(
        "id={new_vm_id}&challenge_id={}&origin_id={}&user_id={}&created_at={}&expire_at={}",
        challenge.id,
        template_id,
        user.id,
        created_at,
        expire_at
    );

    let conf_url = format!("{base_url}/{api_path}/nodes/{}/qemu/{}/config", proxmox_args.node, new_vm_id);
    let conf_body = serde_urlencoded::to_string(&[
        ("description", vm_description),
    ]).unwrap_or_default();

    // update config
    match client.post(&conf_url).header(header::AUTHORIZATION, &auth_value).body(conf_body).send().await {
        Ok(_) => Ok(new_vm_id),
        Err(e) => return Err(e.into())
    }
}

// return Ok only when vm status shows running again
#[cfg(feature = "ssr")]
#[instrument]
pub async fn restart_vm(user: &DbUser, template_id: &u32) -> Result<u32, AppError> {
    let proxmox_args = get_proxmox_args().await?;
    is_host_reachable(&proxmox_args.base_url).await?;

    let client = get_reqwest_client();

    let base_url = proxmox_args.base_url.trim_end_matches("/");
    let api_path = proxmox_args.api_path.trim_start_matches("/").trim_end_matches("/");
    let vm_id = match get_user_vmid_from_template_id(user, template_id).await? {
        Some(vm_id) => vm_id,
        None =>  return Err(AppError::InternalError("".to_string()))
    };
    let url = format!("{base_url}/{api_path}/nodes/{}/qemu/{vm_id}/status/reboot", proxmox_args.node);
    let auth_value = format!("PVEAPIToken={}", proxmox_args.api_token.unwrap_or_default());

    match client.post(url).header(header::AUTHORIZATION, auth_value).send().await {
        Ok(_) => Ok(vm_id),
        Err(e) => Err(e.into())
    }
}

// return Ok only when vm with template id doesn't exist in user pool anymore
#[cfg(feature = "ssr")]
#[instrument]
pub async fn destroy_vm(user: &DbUser, template_id: &u32) -> Result<u32, AppError> {
    let proxmox_args = get_proxmox_args().await?;
    is_host_reachable(&proxmox_args.base_url).await?;

    let client = get_reqwest_client();

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
    match client.post(&stop_url).header(header::AUTHORIZATION, &auth_value).send().await {
        Ok(res) => {
            if res.status().is_success() {
                tokio::time::sleep(Duration::from_secs(3)).await;
                // destroy
                match client.delete(destroy_url).header(header::AUTHORIZATION, auth_value).send().await {
                    Ok(_) => {
                        Ok(vm_id)
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
pub async fn create_user_pool(user: &DbUser) -> Result<(), AppError> {
    let proxmox_args = get_proxmox_args().await?;
    is_host_reachable(&proxmox_args.base_url).await?;

    let client = get_reqwest_client();

    let base_url = proxmox_args.base_url.trim_end_matches("/");
    let api_path = proxmox_args.api_path.trim_start_matches("/").trim_end_matches("/");
    let pools_url = format!("{base_url}/{api_path}/pools");
    let acl_url = format!("{base_url}/{api_path}/access/acl");
    let poolid = format!("CTFPKHK-{}", user.username);
    let auth_value = format!("PVEAPIToken={}", proxmox_args.api_token.unwrap_or_default());

    match client.get(&pools_url).header(header::AUTHORIZATION, &auth_value).send().await {
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

    let body = serde_urlencoded::to_string(&[("poolid", &poolid)]).unwrap_or_default();
    client.post(&pools_url)
        .header(header::AUTHORIZATION, &auth_value)
        .body(body)
        .send()
        .await?;

    let realm_suffix = if user.auth_type == "ldap" { "CTFPKHK" } else { "pve" };
    let acl_body = serde_urlencoded::to_string(&[
        ("path", format!("/pool/{poolid}")),
        ("users", format!("{}@{realm_suffix}", user.username)),
        ("roles", "CTFCompetitor".to_string()),
        ("propagate", "1".to_string())
    ]).unwrap_or_default();
    client.put(&acl_url)
        .header(header::AUTHORIZATION, &auth_value)
        .body(acl_body)
        .send()
        .await?;

    Ok(())
}

#[cfg(feature = "ssr")]
#[instrument]
pub async fn test_auth(args: &ProxmoxArgs) -> Result<(), AppError> {
    is_host_reachable(&args.base_url).await?;

    let client = get_reqwest_client();

    let base_url = args.base_url.trim_end_matches("/");
    let api_path = args.api_path.trim_start_matches("/").trim_end_matches("/");
    let url = format!("{base_url}/{api_path}/nodes/status");
    let auth_value = format!("PVEAPIToken={}", args.api_token.as_deref().unwrap_or_default());

    match client.get(&url).header(header::AUTHORIZATION, &auth_value).send().await {
        Ok(res) => if res.status().is_success() { Ok(()) } else { Err(AppError::Unauthorized) },
        Err(e) => return Err(e.into())
    }
}

#[cfg(feature = "ssr")]
#[instrument]
fn schedule_vm_deletion(user: DbUser, vm_id: u32) {
    tokio::spawn(async move {
        let client = get_reqwest_client();

        let proxmox_args = match get_proxmox_args().await {
            Ok(args) => args,
            Err(e) => { tracing::error!(error = ?e, "failed to get proxmox args for scheduled deletion"); return; }
        };

        let base_url = proxmox_args.base_url.trim_end_matches("/");
        let api_path = proxmox_args.api_path.trim_start_matches("/").trim_end_matches("/");
        let conf_url = format!("{base_url}/{api_path}/nodes/{}/qemu/{}/config", &proxmox_args.node, vm_id);
        let auth_value = format!("PVEAPIToken={}", proxmox_args.api_token.unwrap_or_default());

        let mut intv = tokio::time::interval(Duration::from_secs(60 * 30));
        loop {
            intv.tick().await;

            let config = match client.get(&conf_url).header(header::AUTHORIZATION, &auth_value).send().await {
                Ok(res) => {
                    match res.json::<ProxmoxApiResponse<Config>>().await {
                        Ok(c) => c,
                        Err(e) => { tracing::error!(error = ?e, "failed to parse VM config"); return; }
                    }
                },
                Err(e) => { tracing::error!(error = ?e, "failed to fetch VM config"); return; }
            };
            let description = config.data.description.unwrap_or_default();
            let args = match extract_args_from_description(description) {
                Ok(a) => a,
                Err(e) => { tracing::error!(error = ?e, "failed to parse VM description"); return; }
            };

            let end_at = args.end_at.timestamp();
            let now = Local::now().timestamp();
            if now >= end_at {
                if let Err(e) = destroy_vm(&user, &vm_id).await {
                    tracing::error!(error = ?e, "failed to destroy expired VM");
                }
                return;
            }
        }
    });
}

#[cfg(feature = "ssr")]
#[instrument]
pub async fn add_vm_time(user: &DbUser, template_id: &u32) -> Result<u32, AppError> {
    let proxmox_args = get_proxmox_args().await?;
    is_host_reachable(&proxmox_args.base_url).await?;

    let client = get_reqwest_client();

    let base_url = proxmox_args.base_url.trim_end_matches("/");
    let api_path = proxmox_args.api_path.trim_start_matches("/").trim_end_matches("/");
    let vm_id = match get_user_vmid_from_template_id(user, template_id).await? {
        Some(vm_id) => vm_id,
        None =>  return Err(AppError::InternalError("".to_string()))
    };
    let conf_url = format!("{base_url}/{api_path}/nodes/{}/qemu/{}/config", &proxmox_args.node, vm_id);
    let auth_value = format!("PVEAPIToken={}", proxmox_args.api_token.unwrap_or_default());

    let config = match client.get(&conf_url).header(header::AUTHORIZATION, &auth_value).send().await {
        Ok(res) => res.json::<ProxmoxApiResponse<Config>>().await?,
        Err(e) => return Err(e.into())
    };
    let description = config.data.description.unwrap_or_default();
    let args = extract_args_from_description(description)?;
    let new_expire_at = args.end_at + chrono::Duration::minutes(30);

    let new_description = format!(
        "id={vm_id}&challenge_id={}&origin_id={template_id}&user_id={}&created_at={}&expire_at={}",
        args.challenge_id,
        args.user_id,
        args.created_at,
        new_expire_at
    );

    let conf_body = serde_urlencoded::to_string(&[
        ("description", new_description),
    ]).unwrap_or_default();

    // update config
    if let Err(e) = client.post(&conf_url).header(header::AUTHORIZATION, &auth_value).body(conf_body).send().await {
        return Err(e.into())
    }

    Ok(vm_id)
}

#[cfg(feature = "ssr")]
#[instrument]
pub async fn get_user_vms(user: &DbUser) -> Result<Vec<ProxmoxVMInstance>, AppError> {
    let proxmox_args = get_proxmox_args().await?;
    is_host_reachable(&proxmox_args.base_url).await?;

    let client = get_reqwest_client();

    let base_url = proxmox_args.base_url.trim_end_matches("/");
    let api_path = proxmox_args.api_path.trim_start_matches("/").trim_end_matches("/");
    let auth_value = format!("PVEAPIToken={}", proxmox_args.api_token.unwrap_or_default());
    let poolid = format!("CTFPKHK-{}", user.username);
    let url = format!("{base_url}/{api_path}/pools/{poolid}");

    let vms = match client.get(&url).header(header::AUTHORIZATION, &auth_value).send().await {
        Ok(res) => {
            res.json::<ProxmoxApiResponse<Members>>().await?
        },
        Err(e) => return Err(e.into())
    };

    // using parallelization because all vm configs are needed, so we fetch them concurrently
    let futures  = vms.data.members.into_iter().map(|vm| {
        let conf_url = format!("{base_url}/{api_path}/nodes/{}/qemu/{}/config", &proxmox_args.node, vm.vmid.unwrap_or_default());
        let auth_value = auth_value.clone();
        async move {
            let config = client.get(&conf_url).header(header::AUTHORIZATION, &auth_value).send().await
                .map_err(AppError::from)?
                .json::<ProxmoxApiResponse<Config>>().await?;
            let description = config.data.description.unwrap_or_default();
            let mut args = extract_args_from_description(description)?;
            args.running = vm.status.unwrap_or_default() == "running";
            Ok::<_, AppError>(args)
        }
    }).collect::<Vec<_>>();

    let results = futures::future::join_all(futures).await;
    results.into_iter().collect()
}

#[cfg(feature = "ssr")]
#[instrument]
fn extract_args_from_description(desc: String) -> Result<ProxmoxVMInstance, AppError> {
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
    let proxmox_args = get_proxmox_args().await?;
    is_host_reachable(&proxmox_args.base_url).await?;

    let client = get_reqwest_client();

    let base_url = proxmox_args.base_url.trim_end_matches("/");
    let api_path = proxmox_args.api_path.trim_start_matches("/").trim_end_matches("/");
    let auth_value = format!("PVEAPIToken={}", proxmox_args.api_token.unwrap_or_default());
    let url = format!("{base_url}/{api_path}/pools/{}", proxmox_args.templates_pool_id);

    let vms = match client.get(&url).header(header::AUTHORIZATION, &auth_value).send().await {
        Ok(res) => {
            res.json::<ProxmoxApiResponse<Members>>().await?
        },
        Err(e) => return Err(e.into())
    };

    let templates = vms.data.members.into_iter().map(|vm| {
        ProxmoxVMTemplate { id: vm.vmid.unwrap_or_default(), name: vm.name.unwrap_or_default() }
    }).collect();
    Ok(templates)
}

#[cfg(feature = "ssr")]
#[instrument]
async fn get_user_vmid_from_template_id(user: &DbUser, template_id: &u32) -> Result<Option<u32>, AppError> {
    let proxmox_args = get_proxmox_args().await?;
    is_host_reachable(&proxmox_args.base_url).await?;

    let client = get_reqwest_client();

    let base_url = proxmox_args.base_url.trim_end_matches("/");
    let api_path = proxmox_args.api_path.trim_start_matches("/").trim_end_matches("/");
    let auth_value = format!("PVEAPIToken={}", proxmox_args.api_token.unwrap_or_default());
    let poolid = format!("CTFPKHK-{}", user.username);
    let url = format!("{base_url}/{api_path}/pools/{poolid}");

    let vms = match client.get(&url).header(header::AUTHORIZATION, &auth_value).send().await {
        Ok(res) => res.json::<ProxmoxApiResponse<Members>>().await?,
        Err(e) => return Err(e.into())
    };

    // sequential because we return on the first match, so parallelizing would "over-fetch"
    for vm in vms.data.members {
        let vmid = vm.vmid.unwrap_or_default();
        let conf_url = format!("{base_url}/{api_path}/nodes/{}/qemu/{vmid}/config", &proxmox_args.node);
        let config = match client.get(&conf_url).header(header::AUTHORIZATION, &auth_value).send().await {
            Ok(res) => res.json::<ProxmoxApiResponse<Config>>().await?,
            Err(e) => return Err(e.into())
        };
        let description = config.data.description.unwrap_or_default();
        let args = extract_args_from_description(description)?;
        if args.origin_id == *template_id {
            return Ok(Some(vmid));
        }
    }

    Ok(None)
}

#[cfg(feature = "ssr")]
#[instrument]
pub async fn change_pool_owner(user: &DbUser, new_username: &String) -> Result<(), AppError> {
    let proxmox_args = get_proxmox_args().await?;
    is_host_reachable(&proxmox_args.base_url).await?;

    let client = get_reqwest_client();

    let auth_value = format!("PVEAPIToken={}", proxmox_args.api_token.unwrap_or_default());
    let base_url = proxmox_args.base_url.trim_end_matches("/");
    let api_path = proxmox_args.api_path.trim_start_matches("/").trim_end_matches("/");
    let pools_url = format!("{base_url}/{api_path}/pools");
    let url = format!("{base_url}/{api_path}/access/acl");
    let poolid = format!("CTFPKHK-{}", user.username);

    let res = client.get(&pools_url).header(header::AUTHORIZATION, &auth_value).send().await?;
    let pools = res.json::<ProxmoxApiResponse<Vec<Pools>>>().await?;
    if !pools.data.iter().any(|pool| pool.poolid.contains(&poolid)) {
        return Err(AppError::InternalError(format!("Unable to change pool owner. Pool '{poolid}' does not exist")));
    }

    let acl_body = serde_urlencoded::to_string(&[
        ("path", format!("/pool/{poolid}")),
        ("users", format!("{}@pve", new_username)),
        ("roles", "CTFCompetitor".to_string()),
        ("propagate", "1".to_string())
    ]).unwrap_or_default();
    client.put(&url)
        .header(header::AUTHORIZATION, &auth_value)
        .body(acl_body)
        .send()
        .await?;

    Ok(())
}

#[cfg(feature = "ssr")]
#[instrument]
pub async fn delete_user(db_user: &DbUser) -> Result<(), AppError> {
    let proxmox_args = get_proxmox_args().await?;
    is_host_reachable(&proxmox_args.base_url).await?;

    let client = get_reqwest_client();

    let auth_value = format!("PVEAPIToken={}", proxmox_args.api_token.unwrap_or_default());
    let base_url = proxmox_args.base_url.trim_end_matches("/");
    let api_path = proxmox_args.api_path.trim_start_matches("/").trim_end_matches("/");
    let url = format!("{base_url}/{api_path}/access/users/{}@pve", db_user.username);

    match client.delete(&url).header(header::AUTHORIZATION, &auth_value).send().await {
        Ok(res) => {
            if res.status().is_success() { Ok(()) } else { Err(AppError::InternalError("".to_string())) }
        },
        Err(e) => Err(e.into())
    }
}

#[cfg(feature = "ssr")]
#[instrument]
pub async fn create_user(email: &String, username: &String, password: &String) -> Result<(), AppError> {
    let proxmox_args = get_proxmox_args().await?;
    is_host_reachable(&proxmox_args.base_url).await?;

    let client = get_reqwest_client();

    let auth_value = format!("PVEAPIToken={}", proxmox_args.api_token.unwrap_or_default());
    let base_url = proxmox_args.base_url.trim_end_matches("/");
    let api_path = proxmox_args.api_path.trim_start_matches("/").trim_end_matches("/");
    let url = format!("{base_url}/{api_path}/access/users");

    match client.get(&url).header(header::AUTHORIZATION, &auth_value).send().await {
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
        ("password", password.to_string()),
        ("expire", 0.to_string()),
        ("enable", 1.to_string()),
        ("email", email.to_string()),
    ]).unwrap_or_default();

    match client.post(&url).header(header::AUTHORIZATION, auth_value).body(body).send().await {
        Ok(_) => Ok(()),
        Err(e) => return Err(e.into())
    }
}

#[cfg(feature = "ssr")]
#[instrument]
async fn get_roles() -> Result<Vec<Role>, AppError> {
    let proxmox_args = get_proxmox_args().await?;
    is_host_reachable(&proxmox_args.base_url).await?;

    let client = get_reqwest_client();

    let auth_value = format!("PVEAPIToken={}", proxmox_args.api_token.unwrap_or_default());
    let base_url = proxmox_args.base_url.trim_end_matches("/");
    let api_path = proxmox_args.api_path.trim_start_matches("/").trim_end_matches("/");
    let url = format!("{base_url}/{api_path}/access/roles");

    match client.get(&url).header(header::AUTHORIZATION, &auth_value).send().await {
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
    is_host_reachable(&proxmox_args.base_url).await?;

    let client = get_reqwest_client();

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
        ("privs", "VM.PowerMgmt"),
        ("privs", "Pool.Audit"),
    ]).unwrap_or_default();

    match client.post(&url).header(header::AUTHORIZATION, &auth_value).body(body).send().await {
        Ok(_) => Ok(()),
        Err(e) => return Err(e.into())
    }
}

#[cfg(feature = "ssr")]
#[instrument]
pub async fn get_template_info(template_id: &u32) -> Result<ProxmoxVMTemplate, AppError> {
    let proxmox_args = get_proxmox_args().await?;
    is_host_reachable(&proxmox_args.base_url).await?;

    let client = get_reqwest_client();

    let base_url = proxmox_args.base_url.trim_end_matches("/");
    let api_path = proxmox_args.api_path.trim_start_matches("/").trim_end_matches("/");
    let auth_value = format!("PVEAPIToken={}", proxmox_args.api_token.unwrap_or_default());
    let url = format!("{base_url}/{api_path}/pools/{}", proxmox_args.templates_pool_id);

    let vms = match client.get(&url).header(header::AUTHORIZATION, &auth_value).send().await {
        Ok(res) => {
            res.json::<ProxmoxApiResponse<Members>>().await?
        },
        Err(e) => return Err(e.into())
    };

    let mut template_info = ProxmoxVMTemplate::default();
    for vm in vms.data.members {
        if vm.vmid.unwrap_or_default() == *template_id {
            template_info = ProxmoxVMTemplate { id: vm.vmid.unwrap_or_default(), name: vm.name.unwrap_or_default() };
            break;
        }
    }
    Ok(template_info)
}

#[cfg(feature = "ssr")]
#[instrument]
pub async fn get_proxmox_base_url() -> Result<String, AppError> {
    let args = get_proxmox_args().await?;
    Ok(args.base_url)
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
