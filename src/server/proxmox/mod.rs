/// src/server/proxmox/mod.rs
/// contains code which essentially creates an easy wrapper for the API to use for
/// communicating with the Proxmox server.
/// All the code here is meant to be accessible to standard authenticated users.

#[cfg(feature = "ssr")]
use crate::server::{db::get_db_ref, is_host_reachable};
use crate::{error_template::AppError, server::db::{self, structs::{DbUser, LdapArgs, ProxmoxArgs}}, utils::local_string_to_datetime};
use chrono::{DateTime, Local};
#[cfg(feature = "ssr")]
use once_cell::sync::Lazy;
#[cfg(feature = "ssr")]
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
#[cfg(feature = "ssr")]
use tokio::sync::OnceCell;
use std::{collections::HashMap, time::Duration};
use tracing::instrument;
#[cfg(feature = "ssr")]
use std::sync::{Arc, Mutex as StdMutex};
#[cfg(feature = "ssr")]
use tokio::sync::Mutex as AsyncMutex;

pub mod admin;

#[cfg(feature = "ssr")]
static REQWEST_CLIENT: OnceCell<Client> = OnceCell::const_new();

// These locking functions might become redundant in the future if ever this 
// application is run in multiple instances. Consider using a DB-based lock, 
// a table like `active_vms` or `proxmox_vms`. For now, this is acceptable.
#[cfg(feature = "ssr")]
static VM_LOCKS: Lazy<StdMutex<HashMap<String, Arc<AsyncMutex<()>>>>> =
    Lazy::new(|| StdMutex::new(HashMap::new()));

#[cfg(feature = "ssr")]
pub struct VmLockGuard {
    key: String,
    guard: Option<tokio::sync::OwnedMutexGuard<()>>,
}

#[cfg(feature = "ssr")]
impl Drop for VmLockGuard {
    fn drop(&mut self) {
        self.guard.take();
        if let Ok(mut map) = VM_LOCKS.lock() {
            let is_idle = map.get(&self.key).map(|arc| Arc::strong_count(arc) == 1).unwrap_or(false);
            if is_idle { map.remove(&self.key); }
        }
    }
}

#[cfg(feature = "ssr")]
pub fn acquire_vm_lock(user_id: &str, template_id: &u32) -> Result<VmLockGuard, AppError> {
    let key = format!("{user_id}:{template_id}");
    let arc = {
        let mut map = VM_LOCKS.lock()?;
        map.entry(key.clone()).or_insert_with(|| Arc::new(AsyncMutex::new(()))).clone()
    };
    match arc.try_lock_owned() {
        Ok(guard) => Ok(VmLockGuard { key, guard: Some(guard) }),
        Err(_) => Err(AppError::BadRequest("An action is already in progress".into()))
    }
}

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

#[derive(Debug, Default, Clone, Eq, Hash, PartialEq, Deserialize, Serialize)]
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
pub struct ProxmoxApiResponse<T> {
    pub data: T
}

#[derive(Serialize, Deserialize)]
struct Domains {
    realm: String,
    r#type: String
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
struct VmCurrentStatus {
    status: Option<String>,
    uptime: Option<u64>,
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
pub struct ProxmoxClient {
    pub client: &'static Client,
    pub auth_value: String,
    pub api_base: String,
    pub node: String,
    pub templates_pool_id: String,
}

#[cfg(feature = "ssr")]
impl ProxmoxClient {
    pub async fn new() -> Result<Self, AppError> {
        let proxmox_args = get_proxmox_args().await?;
        if !is_host_reachable(&proxmox_args.base_url).await? {
            return Err(AppError::NetworkError("Proxmox host unreachable".to_string()));
        }

        let client = get_reqwest_client();
        let api_token = proxmox_args.api_token.ok_or(
            AppError::BadRequest("Proxmox config not setup: missing api_token".to_string())
        )?;
        let auth_value = format!("PVEAPIToken={api_token}");
        let base = proxmox_args.base_url.trim_end_matches("/").to_string();
        let path = proxmox_args.api_path.trim_start_matches("/").trim_end_matches("/").to_string();
        let api_base = format!("{base}/{path}");

        Ok(Self { client, auth_value, api_base, node: proxmox_args.node, templates_pool_id: proxmox_args.templates_pool_id })
    }

    pub fn append_to_qemu_url(&self, vm_id: u32) -> String {
        format!("{}/nodes/{}/qemu/{vm_id}", self.api_base, self.node)
    }

    pub fn append_to_api_url(&self, path: &str) -> String {
        format!("{}/{}", self.api_base, path.trim_start_matches("/"))
    }

    pub async fn get_req<T: serde::de::DeserializeOwned>(&self, url: &str) -> Result<ProxmoxApiResponse<T>, AppError> {
        let res = self.client.get(url)
            .header(header::AUTHORIZATION, &self.auth_value)
            .send().await?;
        Ok(res.json::<ProxmoxApiResponse<T>>().await?)
    }

    pub async fn post_req(&self, url: &str, body: Option<String>) -> Result<reqwest::Response, AppError> {
        if let Some(body) = body {
            let res = self.client.post(url)
                .header(header::AUTHORIZATION, &self.auth_value)
                .body(body)
                .send().await?;
            Ok(res)
        } else {
            let res = self.client.post(url)
                .header(header::AUTHORIZATION, &self.auth_value)
                .send().await?;
            Ok(res)
        }
    }

    pub async fn put_req(&self, url: &str, body: String) -> Result<reqwest::Response, AppError> {
        let res = self.client.put(url)
            .header(header::AUTHORIZATION, &self.auth_value)
            .body(body)
            .send().await?;
        Ok(res)
    }

    pub async fn delete_req(&self, url: &str) -> Result<reqwest::Response, AppError> {
        let res = self.client.delete(url)
            .header(header::AUTHORIZATION, &self.auth_value)
            .send().await?;
        Ok(res)
    }
}

#[cfg(feature = "ssr")]
#[instrument]
pub async fn create_realm() -> Result<(), AppError> {
    let pxc = ProxmoxClient::new().await?;

    let sync_url = pxc.append_to_api_url("access/domains/CTFPKHK/sync");
    let url = pxc.append_to_api_url("access/domains");

    let domains = pxc.get_req::<Vec<Domains>>(&url).await?;
    for domain in domains.data {
        if domain.realm.contains("CTFPKHK") {
            return Ok(())
        }
    }

    let Some(ldap_args) = LdapArgs::get(get_db_ref()).await? else {
        return Err(AppError::InternalError("LDAP not configured".to_string()));
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

    let resp = pxc.post_req(&url, Some(body)).await?;

    let body = serde_urlencoded::to_string(&[
        ("scope", "users"),
        ("enable-new", "1"),
        ("remove-vanished", "acl;entry;properties")
    ]).unwrap_or_default();

    if resp.status().is_success() {
        let res = pxc.post_req(&sync_url, Some(body)).await?;
        if res.status().is_success() {
            Ok(())
        } else {
            Err(AppError::InternalError("failed to create realm for Proxmox".to_string()))
        }
    } else {
        Err(AppError::InternalError("Request returned non-200 status code".to_string()))
    }
}

#[cfg(feature = "ssr")]
#[instrument]
pub async fn sync_realm() -> Result<(), AppError> {
    let pxc = ProxmoxClient::new().await?;

    let sync_url = pxc.append_to_api_url("access/domains/CTFPKHK/sync");

    let body = serde_urlencoded::to_string(&[
        ("realm", "CTFPKHK"),
        ("scope", "users"),
        ("enable-new", "1"),
        ("remove-vanished", "acl;entry;properties")
    ]).unwrap_or_default();

    let res = pxc.post_req(&sync_url, Some(body)).await?;
    if res.status().is_success() {
        Ok(())
    } else {
        Err(AppError::InternalError("failed to sync realm for Proxmox".to_string()))
    }
}

#[cfg(feature = "ssr")]
#[instrument]
async fn get_next_free_vm_id() -> Result<u32, AppError> {
    let pxc = ProxmoxClient::new().await?;

    let url = pxc.append_to_api_url("cluster/nextid");

    let next_free_vm_id = pxc.get_req::<String>(&url).await?;
    let next_free_vm_id = next_free_vm_id.data.parse::<u32>()?;
    Ok(next_free_vm_id)
}

#[cfg(feature = "ssr")]
#[instrument]
pub async fn start_vm(template_id: &u32, challenge_id: &str, user: &DbUser) -> Result<u32, AppError> {
    let pxc = ProxmoxClient::new().await?;

    let _guard = acquire_vm_lock(&user.id, template_id)?;

    match get_user_vmid_from_template_id(user, template_id).await? {
        Some(vm_id) => {
            let start_url = format!("{}/status/start", pxc.append_to_qemu_url(vm_id));
            let status_url = format!("{}/status/current", pxc.append_to_qemu_url(vm_id));

            pxc.post_req(&start_url, None).await?;

            for _ in 0..60 {
                tokio::time::sleep(Duration::from_secs(1)).await;
                let vm = pxc.get_req::<Member>(&status_url).await?;

                let vm_status = vm.data.status.unwrap_or_default();
                if vm_status == "running" { 
                    return Ok(vm_id);
                }
            }

            Err(AppError::InternalError("VM failed to start within timeout".to_string()))
        },
        None => {
            let new_vm_id = clone_vm(template_id, challenge_id, user).await?;
            let start_url = format!("{}/status/start", pxc.append_to_qemu_url(new_vm_id));
            let status_url = format!("{}/status/current", pxc.append_to_qemu_url(new_vm_id));

            pxc.post_req(&start_url, None).await?;
            schedule_vm_deletion(user.clone(), new_vm_id, *template_id);

            for _ in 0..60 {
                tokio::time::sleep(Duration::from_secs(1)).await;
                let vm = pxc.get_req::<Member>(&status_url).await?;

                let vm_status = vm.data.status.unwrap_or_default();
                if vm_status == "running" { 
                    return Ok(new_vm_id);
                }
            }

            Err(AppError::InternalError("VM failed to start within timeout".to_string()))
        }
    }
}

#[cfg(feature = "ssr")]
#[instrument]
async fn clone_vm(template_id: &u32, challenge_id: &str, user: &DbUser) -> Result<u32, AppError> {
    let pxc = ProxmoxClient::new().await?;

    let new_vm_id = get_next_free_vm_id().await?;
    let template_info = get_template_info(template_id).await?;

    let clone_body = serde_urlencoded::to_string(&[
        ("newid", new_vm_id.to_string()),
        ("name", template_info.name),
        ("full", "1".to_string()), // 0 - linked clone, 1 - full clone
        ("target", pxc.node.clone()),
        ("pool", format!("CTFPKHK-{}", user.username)),
    ]).unwrap_or_default();

    // clone
    let clone_url = format!("{}/clone", pxc.append_to_qemu_url(*template_id));
    pxc.post_req(&clone_url, Some(clone_body)).await?;

    let now = Local::now();
    let expire_at = (now + chrono::Duration::hours(1)).to_rfc3339();
    let created_at = now.to_rfc3339();

    let vm_description = serde_urlencoded::to_string(&[
        ("id", new_vm_id.to_string()),
        ("challenge_id", challenge_id.to_string()),
        ("origin_id", template_id.to_string()),
        ("user_id", user.id.to_string()),
        ("created_at", created_at),
        ("expire_at", expire_at),
    ]).unwrap_or_default();

    let conf_url = format!("{}/config", pxc.append_to_qemu_url(new_vm_id));
    let conf_body = serde_urlencoded::to_string(&[
        ("description", vm_description),
    ]).unwrap_or_default();

    // update config
    pxc.post_req(&conf_url, Some(conf_body)).await?;
    Ok(new_vm_id)
}

#[cfg(feature = "ssr")]
#[instrument]
pub async fn restart_vm(user: &DbUser, template_id: &u32) -> Result<u32, AppError> {
    let _guard = acquire_vm_lock(&user.id, template_id)?;

    let pxc = ProxmoxClient::new().await?;

    let Some(vm_id) = get_user_vmid_from_template_id(user, template_id).await? else {
        return Err(AppError::InternalError("Failed to get user VM ID from template ID".to_string()))
    };
    let reboot_url = format!("{}/status/reboot", pxc.append_to_qemu_url(vm_id));
    let status_url = format!("{}/status/current", pxc.append_to_qemu_url(vm_id));

    let pre_uptime = pxc.get_req::<VmCurrentStatus>(&status_url).await?.data.uptime.unwrap_or(0);

    pxc.post_req(&reboot_url, None).await?;

    for _ in 0..90 {
        tokio::time::sleep(Duration::from_secs(1)).await;
        let Ok(status) = pxc.get_req::<VmCurrentStatus>(&status_url).await else {
            continue
        };

        let vm_status = status.data.status.unwrap_or_default();
        let uptime = status.data.uptime.unwrap_or(0);

        if vm_status == "running" && uptime < pre_uptime {
            return Ok(vm_id);
        }
    }

    Err(AppError::InternalError("Failed to restart VM within timeout".to_string()))
}

#[cfg(feature = "ssr")]
#[instrument]
pub async fn destroy_vm(user: &DbUser, template_id: &u32) -> Result<u32, AppError> {
    let _guard = acquire_vm_lock(&user.id, template_id)?;
    
    let pxc = ProxmoxClient::new().await?;

    let Some(vm_id) = get_user_vmid_from_template_id(user, template_id).await? else {
        return Err(AppError::InternalError("Failed to get user VM ID from template ID".to_string()))
    };
    let stop_url = format!("{}/status/stop", pxc.append_to_qemu_url(vm_id));
    let status_url = format!("{}/status/current", pxc.append_to_qemu_url(vm_id));

    // stop
    let res = pxc.post_req(&stop_url, None).await?;
    if !res.status().is_success() {
        return Err(AppError::InternalError("Failed to stop VM".to_string()));
    }

    // poll until stopped
    for _ in 0..30 {
        tokio::time::sleep(Duration::from_secs(1)).await;
        let Ok(status) = pxc.get_req::<VmCurrentStatus>(&status_url).await else {
            continue
        };

        if status.data.status.unwrap_or_default() == "stopped" {
            // destroy
            let destroy_url = pxc.append_to_qemu_url(vm_id);
            let del_res = pxc.delete_req(&destroy_url).await?;
            if !del_res.status().is_success() {
                return Err(AppError::InternalError("Failed to destroy VM".to_string()));
            }
            return Ok(vm_id);
        }
    }

    Err(AppError::InternalError("VM failed to stop within timeout".to_string()))
}

#[cfg(feature = "ssr")]
#[instrument]
pub async fn create_user_pool(user: &DbUser) -> Result<(), AppError> {
    let pxc = ProxmoxClient::new().await?;

    let pools_url = pxc.append_to_api_url("pools");
    let acl_url = pxc.append_to_api_url("access/acl");
    let poolid = format!("CTFPKHK-{}", user.username);

    let pools = pxc.get_req::<Vec<Pools>>(&pools_url).await?;
    for pool in pools.data {
        if pool.poolid.contains(&poolid) {
            return Ok(());
        }
    }

    let body = serde_urlencoded::to_string(&[("poolid", &poolid)]).unwrap_or_default();
    pxc.post_req(&pools_url, Some(body)).await?;

    let realm_suffix = if user.auth_type == "ldap" { "CTFPKHK" } else { "pve" };
    let acl_body = serde_urlencoded::to_string(&[
        ("path", format!("/pool/{poolid}")),
        ("users", format!("{}@{realm_suffix}", user.username)),
        ("roles", "CTFCompetitor".to_string()),
        ("propagate", "1".to_string())
    ]).unwrap_or_default();
    pxc.put_req(&acl_url, acl_body).await?;

    Ok(())
}

#[cfg(feature = "ssr")]
#[instrument]
pub async fn test_auth(args: &ProxmoxArgs) -> Result<(), AppError> {
    if !is_host_reachable(&args.base_url).await? {
        return Err(AppError::NetworkError("Proxmox host unreachable".to_string()));
    }

    let client = get_reqwest_client();

    let api_token = args.api_token.clone().ok_or(AppError::BadRequest("Proxmox config not setup: missing api_token".to_string()))?;
    let auth_value = format!("PVEAPIToken={api_token}");
    let base_url = args.base_url.trim_end_matches("/");
    let api_path = args.api_path.trim_start_matches("/").trim_end_matches("/");
    let url = format!("{base_url}/{api_path}/nodes/status");

    let res = client.get(&url).header(header::AUTHORIZATION, &auth_value).send().await?;
    if res.status().is_success() { Ok(()) } else { Err(AppError::Unauthorized) }
}

#[cfg(feature = "ssr")]
#[instrument]
fn schedule_vm_deletion(user: DbUser, vm_id: u32, template_id: u32) {
    tokio::spawn(async move {
        let pxc = match ProxmoxClient::new().await {
            Ok(p) => p,
            Err(e) => { tracing::error!(error = ?e, "failed to initialize proxmox client for scheduled deletion"); return; }
        };

        let conf_url = format!("{}/config", pxc.append_to_qemu_url(vm_id));

        let mut intv = tokio::time::interval(Duration::from_secs(60 * 30));
        loop {
            intv.tick().await;

            let config = match pxc.get_req::<Config>(&conf_url).await {
                Ok(c) => c,
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
                let result = super::proxmox::admin::destroy_vm(&vm_id, &template_id, &user.id).await;
                if result.is_ok() { return; } else if let Err(e) = result { 
                    tracing::error!(error = ?e, "failed to destroy expired VM");
                }
            }
        }
    });
}

#[cfg(feature = "ssr")]
#[instrument]
pub async fn add_vm_time(user: &DbUser, template_id: &u32) -> Result<u32, AppError> {
    let _guard = acquire_vm_lock(&user.id, template_id)?;
    
    let pxc = ProxmoxClient::new().await?;

    let Some(vm_id) = get_user_vmid_from_template_id(user, template_id).await? else {
        return Err(AppError::InternalError("Failed to get user VM ID from template ID".to_string()))
    };
    let conf_url = format!("{}/config", pxc.append_to_qemu_url(vm_id));

    let config = pxc.get_req::<Config>(&conf_url).await?;
    let description = config.data.description.unwrap_or_default();
    let args = extract_args_from_description(description)?;
    let new_expire_at = args.end_at + chrono::Duration::minutes(30);

    let new_description = serde_urlencoded::to_string(&[
        ("id", vm_id.to_string()),
        ("challenge_id", args.challenge_id),
        ("origin_id", template_id.to_string()),
        ("user_id", args.user_id),
        ("created_at", args.created_at.to_rfc3339()),
        ("expire_at", new_expire_at.to_rfc3339()),
    ]).unwrap_or_default();

    let conf_body = serde_urlencoded::to_string(&[
        ("description", new_description),
    ]).unwrap_or_default();

    // update config
    let res = pxc.post_req(&conf_url, Some(conf_body)).await?;
    if !res.status().is_success() {
        return Err(AppError::InternalError("Failed to update VM config".to_string()));
    }

    Ok(vm_id)
}

#[cfg(feature = "ssr")]
#[instrument]
pub async fn get_user_vms(user: &DbUser) -> Result<Vec<ProxmoxVMInstance>, AppError> {
    let pxc = ProxmoxClient::new().await?;

    let poolid = format!("CTFPKHK-{}", user.username);
    let url = pxc.append_to_api_url(&format!("pools/{poolid}"));

    let vms = pxc.get_req::<Members>(&url).await?;

    // using parallelization because all vm configs are needed, so we fetch them concurrently
    let futures = vms.data.members.into_iter().map(|vm| {
        let conf_url = format!("{}/config", pxc.append_to_qemu_url(vm.vmid.unwrap_or_default()));
        let auth_value = pxc.auth_value.clone();
        let client = pxc.client;
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
    let params: HashMap<String, String> = serde_urlencoded::from_str(&desc)?;

    let id = params.get("id").cloned().unwrap_or_default().parse::<u32>().unwrap_or_default();
    let challenge_id = params.get("challenge_id").cloned().unwrap_or_default();
    let origin_id = params.get("origin_id").cloned().unwrap_or_default().parse::<u32>().unwrap_or_default();
    let user_id = params.get("user_id").cloned().unwrap_or_default();
    let created_at = params.get("created_at").cloned().unwrap_or_default();
    let end_at = params.get("expire_at").cloned().unwrap_or_default();

    let created_at = local_string_to_datetime(created_at)?;
    let end_at = local_string_to_datetime(end_at)?;

    Ok(ProxmoxVMInstance { id, challenge_id, origin_id, user_id, created_at, end_at, running: false })
}

#[cfg(feature = "ssr")]
#[instrument]
pub async fn get_all_templates() -> Result<Vec<ProxmoxVMTemplate>, AppError> {
    let pxc = ProxmoxClient::new().await?;

    let url = pxc.append_to_api_url(&format!("pools/{}", pxc.templates_pool_id));

    let vms = pxc.get_req::<Members>(&url).await?;

    let templates = vms.data.members.into_iter().map(|vm| {
        ProxmoxVMTemplate { id: vm.vmid.unwrap_or_default(), name: vm.name.unwrap_or_default() }
    }).collect();
    Ok(templates)
}

#[cfg(feature = "ssr")]
#[instrument]
async fn get_user_vmid_from_template_id(user: &DbUser, template_id: &u32) -> Result<Option<u32>, AppError> {
    let pxc = ProxmoxClient::new().await?;

    let poolid = format!("CTFPKHK-{}", user.username);
    let url = pxc.append_to_api_url(&format!("pools/{poolid}"));

    let vms = pxc.get_req::<Members>(&url).await?;

    // sequential because we return on the first match, so parallelizing would "over-fetch"
    for vm in vms.data.members {
        let vmid = vm.vmid.unwrap_or_default();
        let conf_url = format!("{}/config", pxc.append_to_qemu_url(vmid));
        let config = pxc.get_req::<Config>(&conf_url).await?;
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
pub async fn change_pool_owner(user: &DbUser, new_username: &str) -> Result<(), AppError> {
    let pxc = ProxmoxClient::new().await?;

    let pools_url = pxc.append_to_api_url("pools");
    let acl_url = pxc.append_to_api_url("access/acl");
    let poolid = format!("CTFPKHK-{}", user.username);

    let pools = pxc.get_req::<Vec<Pools>>(&pools_url).await?;
    if !pools.data.iter().any(|pool| pool.poolid.contains(&poolid)) {
        return Err(AppError::InternalError(format!("Unable to change pool owner. Pool '{poolid}' does not exist")));
    }

    let acl_body = serde_urlencoded::to_string(&[
        ("path", format!("/pool/{poolid}")),
        ("users", format!("{}@pve", new_username)),
        ("roles", "CTFCompetitor".to_string()),
        ("propagate", "1".to_string())
    ]).unwrap_or_default();
    pxc.put_req(&acl_url, acl_body).await?;

    Ok(())
}

#[cfg(feature = "ssr")]
#[instrument]
pub async fn delete_user(db_user: &DbUser) -> Result<(), AppError> {
    let pxc = ProxmoxClient::new().await?;

    let url = pxc.append_to_api_url(&format!("access/users/{}@pve", db_user.username));

    let res = pxc.delete_req(&url).await?;
    if res.status().is_success() { Ok(()) } else { Err(AppError::InternalError("Request returned non-200 status code".to_string())) }
}

#[cfg(feature = "ssr")]
#[instrument]
pub async fn create_user(email: &str, username: &str, password: &str) -> Result<(), AppError> {
    let pxc = ProxmoxClient::new().await?;

    let url = pxc.append_to_api_url("access/users");

    let users = pxc.get_req::<Vec<User>>(&url).await?;
    let userid = format!("{}@pve", username);
    for user in users.data {
        if user.userid.unwrap_or_default() == userid {
            return Ok(());
        }
    }

    let body = serde_urlencoded::to_string(&[
        ("userid", format!("{}@pve", username)),
        ("password", password.to_string()),
        ("expire", 0.to_string()),
        ("enable", 1.to_string()),
        ("email", email.to_string()),
    ]).unwrap_or_default();

    pxc.post_req(&url, Some(body)).await?;
    Ok(())
}

#[cfg(feature = "ssr")]
#[instrument]
async fn get_roles() -> Result<Vec<Role>, AppError> {
    let pxc = ProxmoxClient::new().await?;

    let url = pxc.append_to_api_url("access/roles");

    let roles = pxc.get_req::<Vec<Role>>(&url).await?;
    Ok(roles.data)
}

#[cfg(feature = "ssr")]
#[instrument]
pub async fn create_user_role() -> Result<(), AppError> {
    let pxc = ProxmoxClient::new().await?;

    let url = pxc.append_to_api_url("access/roles");

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

    pxc.post_req(&url, Some(body)).await?;
    Ok(())
}

#[cfg(feature = "ssr")]
#[instrument]
pub async fn get_template_info(template_id: &u32) -> Result<ProxmoxVMTemplate, AppError> {
    let pxc = ProxmoxClient::new().await?;

    let url = pxc.append_to_api_url(&format!("pools/{}", pxc.templates_pool_id));

    let vms = pxc.get_req::<Members>(&url).await?;

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
pub async fn get_proxmox_userids() -> Result<Vec<String>, AppError> {
    let pxc = ProxmoxClient::new().await?;

    let url = pxc.append_to_api_url("access/users");

    let users = pxc.get_req::<Vec<User>>(&url).await?;
    Ok(users.data.into_iter().filter_map(|u| u.userid).collect())
}

#[cfg(feature = "ssr")]
#[instrument]
pub async fn get_proxmox_poolids() -> Result<Vec<String>, AppError> {
    let pxc = ProxmoxClient::new().await?;

    let url = pxc.append_to_api_url("pools");

    let pools = pxc.get_req::<Vec<Pools>>(&url).await?;
    Ok(pools.data.into_iter().map(|p| p.poolid).collect())
}

#[cfg(feature = "ssr")]
#[instrument]
pub async fn delete_user_pool(db_user: &DbUser) -> Result<(), AppError> {
    let pxc = ProxmoxClient::new().await?;

    let poolid = format!("CTFPKHK-{}", db_user.username);
    let url = pxc.append_to_api_url(&format!("pools/{poolid}"));

    let res = pxc.delete_req(&url).await?;
    if res.status().is_success() { Ok(()) } else { Err(AppError::InternalError("failed to delete pool".to_string())) }
}

#[cfg(feature = "ssr")]
#[instrument]
pub async fn delete_proxmox_user(db_user: &DbUser) -> Result<(), AppError> {
    let pxc = ProxmoxClient::new().await?;

    let realm_suffix = if db_user.auth_type == "ldap" { "CTFPKHK" } else { "pve" };
    let url = pxc.append_to_api_url(&format!("access/users/{}@{realm_suffix}", db_user.username));

    let res = pxc.delete_req(&url).await?;
    if res.status().is_success() { Ok(()) } else { Err(AppError::InternalError("failed to delete user".to_string())) }
}

#[cfg(feature = "ssr")]
#[instrument]
pub async fn create_proxmox_user(db_user: &DbUser) -> Result<(), AppError> {
    let pxc = ProxmoxClient::new().await?;

    let url = pxc.append_to_api_url("access/users");

    if db_user.auth_type == "ldap" {
        match sync_realm().await {
            Ok(_) => Ok(()),
            Err(e) => Err(e)
        }
    } else {
        // use rand::Rng;

        let userid = format!("{}@pve", db_user.username);
        // let password = rand::rng().sample_iter(&rand::distr::Alphanumeric).take(12).map(char::from).collect();

        let params = vec![
            ("userid".to_string(), userid),
            ("expire".to_string(), "0".to_string()),
            ("enable".to_string(), "1".to_string()),
            ("email".to_string(), db_user.email.clone()),
            // ("password".to_string(), password)
            ("password".to_string(), "Reverse5".to_string())
        ];

        let body = serde_urlencoded::to_string(&params).unwrap_or_default();
        let res = pxc.post_req(&url, Some(body)).await?;
        if res.status().is_success() { Ok(()) } else { Err(AppError::InternalError("failed to create user".to_string())) }
    }
}

#[cfg(feature = "ssr")]
#[instrument]
async fn get_proxmox_args() -> Result<ProxmoxArgs, AppError> {
    if let Some(args) = db::structs::ProxmoxArgs::get(get_db_ref()).await? {
        Ok(args)
    } else {
        Err(AppError::InternalError("Proxmox config not setup".to_string()))
    }
}
