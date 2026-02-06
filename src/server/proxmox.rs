use crate::{error_template::AppError, server::{db::{self, get_db_ref, structs::{Challenge, DbUser, LdapArgs}}, pool}};
use serde::{Deserialize, Serialize};
use tracing::instrument;

#[instrument]
pub async fn create_proxmox_realm() -> Result<(), AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let client = reqwest::Client::builder()
                .danger_accept_invalid_certs(true)
                .build()?;

            let proxmox_args = match db::structs::ProxmoxArgs::get(get_db_ref()).await {
                Ok(res) => if res.is_some() { res.unwrap_or_default() } else { return Ok(()) },
                Err(e) => return Err(e.into())
            };
            let base_url = proxmox_args.base_url.trim_end_matches("/");
            let api_path = proxmox_args.api_path.trim_end_matches("/");
            let url = format!("{base_url}/{api_path}/access/domains");

            #[derive(Serialize, Deserialize)]
            struct Domains {
                realm: String,
                _type: String
            }
            match client.get(url.clone()).send().await {
                Ok(res) => {
                    let domains = res.json::<Vec<Domains>>().await?;
                    for domain in domains {
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

            let auth_value = format!("PVEAPIToken={}", proxmox_args.api_token.unwrap_or_default());
            let body = serde_urlencoded::to_string(&[
                ("type", "ldap"), 
                ("realm", "ctfpkhk"),
                ("mode", "ldap"), // in the future use ldaps
                ("server1", ldap_args.url.as_str()),
                ("base_dn", ldap_args.base_dn.as_str()),
                ("bind_dn", ldap_args.bind_dn.as_str()),
                ("user_attr", "sAMAccountName"),
                ("password", ldap_args.bind_pw.as_str()),
                ("verify", "0"),
            ]).unwrap_or_default();

            client
                .post(&url)
                .header(reqwest::header::AUTHORIZATION, auth_value)
                .body(body)
                .send()
                .await?;

            Ok(())
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[instrument]
pub async fn get_next_free_vm_id() -> Result<String, AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let client = reqwest::Client::builder()
                .danger_accept_invalid_certs(true)
                .build()?;

            let proxmox_args = match db::structs::ProxmoxArgs::get(get_db_ref()).await {
                Ok(res) => if res.is_some() { res.unwrap_or_default() } else { return Ok("".to_string()) },
                Err(e) => return Err(e.into())
            };
            let base_url = proxmox_args.base_url.trim_end_matches("/");
            let api_path = proxmox_args.api_path.trim_end_matches("/");
            let url = format!("{base_url}/{api_path}/cluster/nextid");

            match client.get(url.clone()).header(reqwest::header::AUTHORIZATION, proxmox_args.api_token.unwrap_or_default()).send().await {
                Ok(res) => {
                    // get value from res
                    Ok("".to_string()) // tmp placeholder
                },
                Err(e) => return Err(e.into())
            }
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}

#[instrument]
pub async fn start_vm(challenge: Challenge, user: DbUser) -> Result<String, AppError> {
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?;

    let proxmox_args = match db::structs::ProxmoxArgs::get(get_db_ref()).await {
        Ok(res) => if res.is_some() { res.unwrap_or_default() } else { return Err(AppError::InternalError("".to_string())) },
        Err(e) => return Err(e.into())
    };
    let base_url = proxmox_args.base_url.trim_end_matches("/");
    let api_path = proxmox_args.api_path.trim_end_matches("/");
    let start_url = format!("{base_url}/{api_path}/nodes/{}/qemu/{}/status/start", proxmox_args.node, challenge.vm_id.clone().unwrap_or_default());
    let clone_url = format!("{base_url}/{api_path}/nodes/{}/qemu/{}/clone", proxmox_args.node, challenge.vm_id.unwrap_or_default());
    let new_vm_id = get_next_free_vm_id().await?;

    let clone_body = serde_urlencoded::to_string(&[
        ("newid", new_vm_id.clone()), 
        ("name", challenge.name),
        ("full", "0".to_string()),
        ("pool", format!("CTFPKHK-{}", user.username)),
    ]).unwrap_or_default();

    // clone
    match client.post(clone_url).header(reqwest::header::AUTHORIZATION, proxmox_args.api_token.clone().unwrap_or_default()).body(clone_body).send().await {
        Ok(_) => {},
        Err(e) => return Err(e.into())
    }

    // start
    match client.post(start_url).header(reqwest::header::AUTHORIZATION, proxmox_args.api_token.unwrap_or_default()).send().await {
        Ok(_) => Ok(new_vm_id),
        Err(e) => return Err(e.into())
    }
}

pub async fn restart_vm(vm_id: String) -> Result<(), AppError> {
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?;

    let proxmox_args = match db::structs::ProxmoxArgs::get(get_db_ref()).await {
        Ok(res) => if res.is_some() { res.unwrap_or_default() } else { return Ok(()) },
        Err(e) => return Err(e.into())
    };
    let base_url = proxmox_args.base_url.trim_end_matches("/");
    let api_path = proxmox_args.api_path.trim_end_matches("/");
    let url = format!("{base_url}/{api_path}/nodes/{}/qemu/{vm_id}/status/reboot", proxmox_args.node);

    match client.post(url).header(reqwest::header::AUTHORIZATION, proxmox_args.api_token.unwrap_or_default()).send().await {
        Ok(_) => Ok(()),
        Err(e) => Err(e.into())
    }
}

#[instrument]
pub async fn destroy_vm(vm_id: String) -> Result<(), AppError> {
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?;

    let proxmox_args = match db::structs::ProxmoxArgs::get(get_db_ref()).await {
        Ok(res) => if res.is_some() { res.unwrap_or_default() } else { return Ok(()) },
        Err(e) => return Err(e.into())
    };
    let base_url = proxmox_args.base_url.trim_end_matches("/");
    let api_path = proxmox_args.api_path.trim_end_matches("/");
    let url = format!("{base_url}/{api_path}/nodes/{}/qemu/{vm_id}", proxmox_args.node);
    
    match client.delete(url).header(reqwest::header::AUTHORIZATION, proxmox_args.api_token.unwrap_or_default()).send().await {
        Ok(_) => Ok(()),
        Err(e) => Err(e.into())
    }
}

#[instrument]
pub async fn create_proxmox_user_pool(user: DbUser) -> Result<(), AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
            let client = reqwest::Client::builder()
                .danger_accept_invalid_certs(true)
                .build()?;

            let proxmox_args = match db::structs::ProxmoxArgs::get(get_db_ref()).await {
                Ok(res) => if res.is_some() { res.unwrap_or_default() } else { return Ok(()) },
                Err(e) => return Err(e.into())
            };
            let base_url = proxmox_args.base_url.trim_end_matches("/");
            let api_path = proxmox_args.api_path.trim_end_matches("/");
            let pools_url = format!("{base_url}/{api_path}/pools");
            let acl_url = format!("{base_url}/{api_path}/access/acl");
            let poolid = format!("CTFPKHK-{}", user.username);
            let auth_value = format!("PVEAPIToken={}", proxmox_args.api_token.unwrap_or_default());

            #[derive(Serialize, Deserialize)]
            struct Pools {
                poolid: String,
                _type: String
            }
            match client.get(pools_url.clone()).header(reqwest::header::AUTHORIZATION, auth_value.clone()).send().await {
                Ok(res) => {
                    let pools = res.json::<Vec<Pools>>().await?;
                    for pool in pools {
                        if pool.poolid.contains(&poolid) {
                            return Ok(())
                        }
                    }
                },
                Err(e) => return Err(e.into())
            };

            let body = serde_urlencoded::to_string(&[("poolid", poolid.clone())]).unwrap_or_default();
            client.post(&pools_url)
                .header(reqwest::header::AUTHORIZATION, auth_value.clone())
                .body(body)
                .send()
                .await?;

            let body = serde_urlencoded::to_string(&[
                ("path", format!("/pool/{poolid}")),
                ("users", user.email),
                ("roles", "PVEVMAdmin".to_string()),
                ("propagate", "1".to_string())
            ]).unwrap_or_default();
            client.put(&acl_url)
                .header(reqwest::header::AUTHORIZATION, auth_value)
                .body(body)
                .send()
                .await?;

            Ok(())
        } else {
            Err(AppError::NoServerConnection)
        }
    }
}
