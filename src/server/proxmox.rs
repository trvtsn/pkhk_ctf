#[cfg(feature = "ssr")]
use crate::server::db::get_db_ref;
use crate::{error_template::AppError, server::{db::{self, structs::{Challenge, DbUser, LdapArgs}}}};
#[cfg(feature = "ssr")]
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
use tracing::instrument;

#[derive(Deserialize)]
struct ProxmoxApiResponse<T> {
    data: T
}

// #[derive(Deserialize)]
// struct TicketData {
//     pub ticket: String,
//     pub CSRFPreventionToken: Option<String>,
//     pub username: Option<String>,
// }

#[cfg(feature = "ssr")]
#[instrument]
pub async fn create_realm() -> Result<(), AppError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "ssr")] {
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
pub async fn get_next_free_vm_id() -> Result<String, AppError> {
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
                    Ok(next_free_vm_id.data)
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
    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?;

    let proxmox_args = match db::structs::ProxmoxArgs::get(get_db_ref()).await {
        Ok(res) => if res.is_some() { res.unwrap_or_default() } else { return Err(AppError::InternalError("missing proxmox args".to_string())) },
        Err(e) => return Err(e.into())
    };
    let base_url = proxmox_args.base_url.trim_end_matches("/");
    let api_path = proxmox_args.api_path.trim_start_matches("/").trim_end_matches("/");
    let clone_url = format!("{base_url}/{api_path}/nodes/{}/qemu/{}/clone", proxmox_args.node, challenge.vm_id.unwrap_or_default());
    let new_vm_id = get_next_free_vm_id().await?;
    let start_url = format!("{base_url}/{api_path}/nodes/{}/qemu/{}/status/start", proxmox_args.node, new_vm_id.clone());
    let auth_value = format!("PVEAPIToken={}", proxmox_args.api_token.unwrap_or_default());

    let clone_body = serde_urlencoded::to_string(&[
        ("newid", new_vm_id.to_string()), 
        ("name", challenge.name),
        ("full", "1".to_string()), // 0 - linked clone, 1 - full clone
        ("target", proxmox_args.node),
        ("pool", format!("CTFPKHK-{}", user.username)),
    ]).unwrap_or_default();

    // clone
    match client.post(clone_url).header(header::AUTHORIZATION, auth_value.clone()).body(clone_body).send().await {
        Ok(_) => {},
        Err(e) => return Err(e.into())
    }

    // start
    match client.post(start_url).header(header::AUTHORIZATION, auth_value).send().await {
        Ok(_) => Ok(new_vm_id.to_string()),
        Err(e) => return Err(e.into())
    }
}

#[cfg(feature = "ssr")]
#[instrument]
pub async fn restart_vm(vm_id: u32) -> Result<(), AppError> {
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
    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?;

    let proxmox_args = match db::structs::ProxmoxArgs::get(get_db_ref()).await {
        Ok(res) => if res.is_some() { res.unwrap_or_default() } else { return Err(AppError::InternalError("missing proxmox args".to_string())) },
        Err(e) => return Err(e.into())
    };
    let base_url = proxmox_args.base_url.trim_end_matches("/");
    let api_path = proxmox_args.api_path.trim_start_matches("/").trim_end_matches("/");
    let url = format!("{base_url}/{api_path}/nodes/{}/qemu/{vm_id}", proxmox_args.node);
    let auth_value = format!("PVEAPIToken={}", proxmox_args.api_token.unwrap_or_default());
    
    match client.delete(url).header(header::AUTHORIZATION, auth_value).send().await {
        Ok(_) => Ok(()),
        Err(e) => Err(e.into())
    }
}

#[cfg(feature = "ssr")]
#[instrument]
pub async fn create_user_pool(user: DbUser) -> Result<(), AppError> {
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
