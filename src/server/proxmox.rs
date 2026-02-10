#[cfg(feature = "ssr")]
use crate::server::db::get_db_ref;
use crate::{error_template::AppError, server::db::{self, structs::{Challenge, DbUser, LdapArgs}}, utils::html_local_to_datetime};
use chrono::{DateTime, Local};
#[cfg(feature = "ssr")]
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};
use tracing::instrument;

#[derive(Debug, Default, Clone, Deserialize, Serialize)]
pub struct ProxmoxVMInstance {
    pub id: u32,
    pub challenge_id: String,
    pub user_id: String,
    pub created_at: DateTime<Local>,
    pub end_at: DateTime<Local>
}

#[derive(Deserialize)]
struct ProxmoxApiResponse<T> {
    data: T
}

#[cfg(feature = "ssr")]
#[instrument]
pub async fn create_realm() -> Result<(), AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            match is_host_reachable().await {
                Ok(reachable) => if reachable {} else { return Err(AppError::InternalError("proxmox host not reachable".to_string())) },
                Err(_) => return Err(AppError::InternalError("proxmox host not reachable".to_string()))
            }

            let client = Client::builder()
                .danger_accept_invalid_certs(true)
                .cookie_store(true)
                .build()?;

            let proxmox_args = match db::structs::ProxmoxArgs::get(get_db_ref()).await {
                Ok(res) => if res.is_some() { res.unwrap_or_default() } else { return Err(AppError::InternalError("missing proxmox args".to_string())) },
                Err(e) => return Err(e.into())
            };
            let base_url = proxmox_args.base_url.trim_end_matches("/");
            let api_path = proxmox_args.api_path.trim_start_matches("/").trim_end_matches("/");
            let sync_url = format!("{base_url}/{api_path}/access/domains/ctfpkhk/sync");
            let url = format!("{base_url}/{api_path}/access/domains");

            #[derive(Serialize, Deserialize)]
            struct Domains {
                realm: String,
                r#type: String
            }
            let auth_value = format!("PVEAPIToken={}", proxmox_args.api_token.unwrap_or_default());
            match client.get(url.clone()).header(header::AUTHORIZATION, auth_value.clone()).send().await {
                Ok(res) => {
                    let domains = res.json::<ProxmoxApiResponse<Vec<Domains>>>().await?;
                    for domain in domains.data {
                        if domain.realm.contains("ctfpkhk") {
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
                ("realm", "ctfpkhk"),
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
                            Err(AppError::InternalError("".to_string()))
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
async fn get_next_free_vm_id() -> Result<u32, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let client = Client::builder()
                .danger_accept_invalid_certs(true)
                .build()?;

            let proxmox_args = match db::structs::ProxmoxArgs::get(get_db_ref()).await {
                Ok(res) => if res.is_some() { res.unwrap_or_default() } else { return Err(AppError::InternalError("missing proxmox args".to_string())) },
                Err(e) => return Err(e.into())
            };
            let base_url = proxmox_args.base_url.trim_end_matches("/");
            let api_path = proxmox_args.api_path.trim_start_matches("/").trim_end_matches("/");
            let url = format!("{base_url}/{api_path}/cluster/nextid");
            let auth_value = format!("PVEAPIToken={}", proxmox_args.api_token.unwrap_or_default());

            #[derive(Deserialize)]
            struct Vmid {
                data: String
            }
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
pub async fn start_vm(challenge: Challenge, user: DbUser) -> Result<String, AppError> {
    match is_host_reachable().await {
        Ok(reachable) => if reachable {} else { return Err(AppError::InternalError("proxmox host not reachable".to_string())) },
        Err(_) => return Err(AppError::InternalError("proxmox host not reachable".to_string()))
    }

    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?;

    let proxmox_args = match db::structs::ProxmoxArgs::get(get_db_ref()).await {
        Ok(res) => if res.is_some() { res.unwrap_or_default() } else { return Err(AppError::InternalError("missing proxmox args".to_string())) },
        Err(e) => return Err(e.into())
    };
    let created_at = Local::now();
    let expire_at = created_at + chrono::Duration::hours(1);
    let mut vm_id = String::new();
    let auth_value = format!("PVEAPIToken={}", proxmox_args.api_token.unwrap_or_default());

    let base_url = proxmox_args.base_url.trim_end_matches("/");
    let api_path = proxmox_args.api_path.trim_start_matches("/").trim_end_matches("/");

    let clone_url = format!("{base_url}/{api_path}/nodes/{}/qemu/{}/clone", proxmox_args.node, challenge.vm_id.unwrap_or_default());

    let active_vms = get_user_active_vms(user.clone()).await?;
    let mut vm_exists = false;
    for active_vm in active_vms {
        if active_vm.challenge_id == challenge.id {
            vm_exists = true;
            vm_id = active_vm.id.to_string();
        }
    }
    if !vm_exists {
		let new_vm_id = get_next_free_vm_id().await?;
		vm_id = new_vm_id.to_string();
        let conf_url = format!("{base_url}/{api_path}/nodes/{}/qemu/{}/config", proxmox_args.node, vm_id.clone());
        let clone_body = serde_urlencoded::to_string(&[
            ("newid", new_vm_id.to_string()), 
            ("name", challenge.name),
            ("full", "1".to_string()), // 0 - linked clone, 1 - full clone
            ("target", proxmox_args.node.clone()),
            ("pool", format!("CTFPKHK-{}", user.username)),
        ]).unwrap_or_default();

        // clone
        match client.post(clone_url).header(header::AUTHORIZATION, auth_value.clone()).body(clone_body).send().await {
            Ok(_) => {},
            Err(e) => return Err(e.into())
        }
		let vm_description = format!("id={new_vm_id}&challenge_id={}&user_id={}&created_at={}&expire_at={}", challenge.id.clone(), user.id.clone(), created_at.to_string(), expire_at.to_string());

        let conf_body = serde_urlencoded::to_string(&[
            ("description", vm_description), 
        ]).unwrap_or_default();

        // update config
        match client.post(conf_url).header(header::AUTHORIZATION, auth_value.clone()).body(conf_body).send().await {
            Ok(_) => {},
            Err(e) => return Err(e.into())
        }
    }

    let start_url = format!("{base_url}/{api_path}/nodes/{}/qemu/{}/status/start", proxmox_args.node, vm_id.clone());

    // start
    match client.post(start_url).header(header::AUTHORIZATION, auth_value).send().await {
        Ok(_) => {
            _ = schedule_vm_deletion(vm_id.parse::<u32>()?).await;
            Ok(vm_id.to_string())
        },
        Err(e) => return Err(e.into())
    }


}

#[cfg(feature = "ssr")]
#[instrument]
pub async fn restart_vm(vm_id: u32) -> Result<(), AppError> {
    match is_host_reachable().await {
        Ok(reachable) => if reachable {} else { return Err(AppError::InternalError("proxmox host not reachable".to_string())) },
        Err(_) => return Err(AppError::InternalError("proxmox host not reachable".to_string()))
    }

    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?;

    let proxmox_args = match db::structs::ProxmoxArgs::get(get_db_ref()).await {
        Ok(res) => if res.is_some() { res.unwrap_or_default() } else { return Err(AppError::InternalError("missing proxmox args".to_string())) },
        Err(e) => return Err(e.into())
    };
    let base_url = proxmox_args.base_url.trim_end_matches("/");
    let api_path = proxmox_args.api_path.trim_start_matches("/").trim_end_matches("/");
    let url = format!("{base_url}/{api_path}/nodes/{}/qemu/{vm_id}/status/reboot", proxmox_args.node);
    let auth_value = format!("PVEAPIToken={}", proxmox_args.api_token.unwrap_or_default());

    match client.post(url).header(header::AUTHORIZATION, auth_value).send().await {
        Ok(_) => Ok(()),
        Err(e) => Err(e.into())
    }
}

#[cfg(feature = "ssr")]
#[instrument]
pub async fn destroy_vm(vm_id: u32) -> Result<(), AppError> {
    match is_host_reachable().await {
        Ok(reachable) => if reachable {} else { return Err(AppError::InternalError("proxmox host not reachable".to_string())) },
        Err(_) => return Err(AppError::InternalError("proxmox host not reachable".to_string()))
    }

    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?;

    let proxmox_args = match db::structs::ProxmoxArgs::get(get_db_ref()).await {
        Ok(res) => if res.is_some() { res.unwrap_or_default() } else { return Err(AppError::InternalError("missing proxmox args".to_string())) },
        Err(e) => return Err(e.into())
    };
    let base_url = proxmox_args.base_url.trim_end_matches("/");
    let api_path = proxmox_args.api_path.trim_start_matches("/").trim_end_matches("/");
    let destroy_url = format!("{base_url}/{api_path}/nodes/{}/qemu/{vm_id}", proxmox_args.node);
    let stop_url = format!("{base_url}/{api_path}/nodes/{}/qemu/{vm_id}/status/stop", proxmox_args.node);
    let auth_value = format!("PVEAPIToken={}", proxmox_args.api_token.unwrap_or_default());
    
    // stop
    match client.post(stop_url).header(header::AUTHORIZATION, auth_value.clone()).send().await {
        Ok(_) => {
            // destroy
            match client.delete(destroy_url).header(header::AUTHORIZATION, auth_value).send().await {
                Ok(_) => Ok(()),
                Err(e) => Err(e.into())
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
            match is_host_reachable().await {
                Ok(reachable) => if reachable {} else { return Err(AppError::InternalError("proxmox host not reachable".to_string())) },
                Err(_) => return Err(AppError::InternalError("proxmox host not reachable".to_string()))
            }

            let client = Client::builder()
                .danger_accept_invalid_certs(true)
                .build()?;

            let proxmox_args = match db::structs::ProxmoxArgs::get(get_db_ref()).await {
                Ok(res) => if res.is_some() { res.unwrap_or_default() } else { return Err(AppError::InternalError("missing proxmox args".to_string())) },
                Err(e) => return Err(e.into())
            };
            let base_url = proxmox_args.base_url.trim_end_matches("/");
            let api_path = proxmox_args.api_path.trim_start_matches("/").trim_end_matches("/");
            let pools_url = format!("{base_url}/{api_path}/pools");
            let acl_url = format!("{base_url}/{api_path}/access/acl");
            let poolid = format!("CTFPKHK-{}", user.username);
            let auth_value = format!("PVEAPIToken={}", proxmox_args.api_token.unwrap_or_default());

            #[derive(Serialize, Deserialize)]
            struct Pools {
                poolid: String,
                r#type: Option<String>
            }
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

            let body = serde_urlencoded::to_string(&[
                ("path", format!("/pool/{poolid}")),
                ("users", format!("{}@ctfpkhk", user.username)),
                ("roles", "PVEVMAdmin".to_string()),
                ("propagate", "1".to_string())
            ]).unwrap_or_default();
            client.put(&acl_url)
                .header(header::AUTHORIZATION, auth_value)
                .body(body)
                .send()
                .await?;

            Ok(())
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[cfg(feature = "ssr")]
#[instrument]
pub async fn test_auth() -> Result<(), AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            match is_host_reachable().await {
                Ok(reachable) => if reachable {} else { return Err(AppError::InternalError("proxmox host not reachable".to_string())) },
                Err(_) => return Err(AppError::InternalError("proxmox host not reachable".to_string()))
            }

            let client = Client::builder()
                .danger_accept_invalid_certs(true)
                .cookie_store(true)
                .build()?;

            let proxmox_args = match db::structs::ProxmoxArgs::get(get_db_ref()).await {
                Ok(res) => if res.is_some() { res.unwrap_or_default() } else { return Err(AppError::InternalError("missing proxmox args".to_string())) },
                Err(e) => return Err(e.into())
            };
            let base_url = proxmox_args.base_url.trim_end_matches("/");
            let api_path = proxmox_args.api_path.trim_start_matches("/").trim_end_matches("/");
            let url = format!("{base_url}/{api_path}/nodes/status");
            let auth_value = format!("PVEAPIToken={}", proxmox_args.api_token.unwrap_or_default());

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
async fn is_host_reachable() -> Result<bool, AppError> {
    let proxmox_args = match db::structs::ProxmoxArgs::get(get_db_ref()).await {
        Ok(res) => if res.is_some() { res.unwrap_or_default() } else { return Err(AppError::InternalError("missing proxmox args".to_string())) },
        Err(e) => return Err(e.into())
    };
    let url = url::Url::parse(&proxmox_args.base_url)?;
    let host = url.host_str().unwrap_or_default();
    let timeout = Duration::from_millis(1000);
    let addrs = (host, 8006).to_socket_addrs()?;
    let start = Instant::now();

    for addr in addrs {
        let elapsed = start.elapsed();
        if elapsed >= timeout {
            return Ok(false);
        }
        let remaining = timeout - elapsed;

        match TcpStream::connect_timeout(&addr, remaining) {
            Ok(stream) => {
                let _ = stream.set_read_timeout(Some(Duration::from_millis(500)));
                let _ = stream.set_write_timeout(Some(Duration::from_millis(500)));
                return Ok(true);
            }
            Err(_e) => {
                continue;
            }
        }
    }

    Ok(false)
}

#[cfg(feature = "ssr")]
#[instrument]
async fn schedule_vm_deletion(vm_id: u32) -> Result<(), AppError> {
    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?;

    let proxmox_args = match db::structs::ProxmoxArgs::get(get_db_ref()).await {
        Ok(res) => if res.is_some() { res.unwrap_or_default() } else { return Err(AppError::InternalError("missing proxmox args".to_string())) },
        Err(e) => return Err(e.into())
    };

    let base_url = proxmox_args.base_url.trim_end_matches("/");
    let api_path = proxmox_args.api_path.trim_start_matches("/").trim_end_matches("/");
    let conf_url = format!("{base_url}/{api_path}/nodes/{}/qemu/{}/config", proxmox_args.node.clone(), vm_id);
    let auth_value = format!("PVEAPIToken={}", proxmox_args.api_token.unwrap_or_default());

    #[derive(Serialize, Deserialize)]
    struct Config {
        description: Option<String>
    }

    let handle = tokio::spawn(async move {
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
                return destroy_vm(vm_id).await;
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
pub async fn add_vm_time(vm_id: u32) -> Result<(), AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            match is_host_reachable().await {
                Ok(reachable) => if reachable {} else { return Err(AppError::InternalError("proxmox host not reachable".to_string())) },
                Err(_) => return Err(AppError::InternalError("proxmox host not reachable".to_string()))
            }

            let client = Client::builder()
                .danger_accept_invalid_certs(true)
                .build()?;

            let proxmox_args = match db::structs::ProxmoxArgs::get(get_db_ref()).await {
                Ok(res) => if res.is_some() { res.unwrap_or_default() } else { return Err(AppError::InternalError("missing proxmox args".to_string())) },
                Err(e) => return Err(e.into())
            };
            let base_url = proxmox_args.base_url.trim_end_matches("/");
            let api_path = proxmox_args.api_path.trim_start_matches("/").trim_end_matches("/");
            let conf_url = format!("{base_url}/{api_path}/nodes/{}/qemu/{}/config", proxmox_args.node.clone(), vm_id);
            let auth_value = format!("PVEAPIToken={}", proxmox_args.api_token.unwrap_or_default());

            #[derive(Serialize, Deserialize)]
            struct Config {
                description: Option<String>
            }
            let config = match client.get(conf_url.clone()).header(header::AUTHORIZATION, auth_value.clone()).send().await {
                Ok(res) => res.json::<ProxmoxApiResponse<Config>>().await?,
                Err(e) => return Err(e.into())
            };
            let description = config.data.description.unwrap_or_default();
            let args = extract_args_from_description(description).await?;
            let new_expire_at = args.end_at + chrono::Duration::minutes(30);
            
            let new_description = format!("id={vm_id}&challenge_id={}&user_id={}&created_at={}&expire_at={}", args.challenge_id, args.user_id, args.created_at.to_string(), new_expire_at.to_string());

            let conf_body = serde_urlencoded::to_string(&[
                ("description", new_description), 
            ]).unwrap_or_default();

            // update config
            match client.post(conf_url).header(header::AUTHORIZATION, auth_value.clone()).body(conf_body).send().await {
                Ok(_) => {},
                Err(e) => return Err(e.into())
            }

            Ok(())
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[cfg(feature = "ssr")]
#[instrument]
pub async fn get_user_active_vms(user: DbUser) -> Result<Vec<ProxmoxVMInstance>, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            match is_host_reachable().await {
                Ok(reachable) => if reachable {} else { return Err(AppError::InternalError("proxmox host not reachable".to_string())) },
                Err(_) => return Err(AppError::InternalError("proxmox host not reachable".to_string()))
            }

            let client = Client::builder()
                .danger_accept_invalid_certs(true)
                .build()?;

            let proxmox_args = match db::structs::ProxmoxArgs::get(get_db_ref()).await {
                Ok(res) => if res.is_some() { res.unwrap_or_default() } else { return Err(AppError::InternalError("missing proxmox args".to_string())) },
                Err(e) => return Err(e.into())
            };
            let base_url = proxmox_args.base_url.trim_end_matches("/");
            let api_path = proxmox_args.api_path.trim_start_matches("/").trim_end_matches("/");
            let auth_value = format!("PVEAPIToken={}", proxmox_args.api_token.unwrap_or_default());
            let poolid = format!("CTFPKHK-{}", user.username);
            let url = format!("{base_url}/{api_path}/pools/{poolid}");

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
            struct Config {
                description: Option<String>
            }

            let vms = match client.get(url.clone()).header(header::AUTHORIZATION, auth_value.clone()).send().await {
                Ok(res) => {
                    res.json::<ProxmoxApiResponse<Members>>().await?
                },
                Err(e) => return Err(e.into())
            };

            let mut active_vms = Vec::<ProxmoxVMInstance>::new();
            for vm in vms.data.members {
                if vm.status.unwrap_or_default() == "running" {
                    let conf_url = format!("{base_url}/{api_path}/nodes/{}/qemu/{}/config", proxmox_args.node.clone(), vm.vmid.unwrap_or_default());
                    let config = match client.get(conf_url.clone()).header(header::AUTHORIZATION, auth_value.clone()).send().await {
                        Ok(res) => res.json::<ProxmoxApiResponse<Config>>().await?,
                        Err(e) => return Err(e.into())
                    };
                    let description = config.data.description.unwrap_or_default();
                    let args = extract_args_from_description(description).await?;

                    active_vms.push(args);
                }
            }
            Ok(active_vms)
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
    let user_id = params.get("user_id").cloned().unwrap_or_default();
    let created_at = params.get("created_at").cloned().unwrap_or_default();
    let end_at = params.get("expire_at").cloned().unwrap_or_default();

    let created_at = html_local_to_datetime(created_at);
    let end_at = html_local_to_datetime(end_at);

    Ok(ProxmoxVMInstance { id, challenge_id, user_id, created_at, end_at })
}
