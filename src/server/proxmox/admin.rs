#[cfg(feature = "ssr")]
use crate::server::is_host_reachable;
use crate::{error_template::AppError, server::proxmox::{Config, Members, ProxmoxApiResponse, VmCurrentStatus}};
#[cfg(feature = "ssr")]
use crate::server::proxmox::{extract_args_from_description, get_proxmox_args, get_reqwest_client};
#[cfg(feature = "ssr")]
use reqwest::header;
use std::time::Duration;
use tracing::instrument;

#[cfg(feature = "ssr")]
#[instrument]
pub async fn start_vm(vm_id: &u32, username: &String) -> Result<u32, AppError> {
    let proxmox_args = get_proxmox_args().await?;
    is_host_reachable(&proxmox_args.base_url).await?;

    let client = get_reqwest_client();

    let auth_value = format!("PVEAPIToken={}", proxmox_args.api_token.unwrap_or_default());
    let base_url = proxmox_args.base_url.trim_end_matches("/");
    let api_path = proxmox_args.api_path.trim_start_matches("/").trim_end_matches("/");
    let url = format!("{base_url}/{api_path}/nodes/{}/qemu/{}/status/start", proxmox_args.node, vm_id);
    let poolid = format!("CTFPKHK-{}", username);
    let pools_url = format!("{base_url}/{api_path}/pools/{poolid}");

    match client.post(&url).header(header::AUTHORIZATION, &auth_value).send().await {
        Ok(_) => {
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
                    if vm_status == "running" && vm.vmid.unwrap_or_default() == *vm_id { return Ok(*vm_id); } else { continue };
                }
            }

            return Err(AppError::InternalError("VM failed to start within timeout".to_string()));
        },
        Err(e) => return Err(e.into())
    }
}

#[cfg(feature = "ssr")]
#[instrument]
pub async fn restart_vm(vm_id: &u32) -> Result<u32, AppError> {
    let proxmox_args = get_proxmox_args().await?;
    is_host_reachable(&proxmox_args.base_url).await?;

    let client = get_reqwest_client();

    let base_url = proxmox_args.base_url.trim_end_matches("/");
    let api_path = proxmox_args.api_path.trim_start_matches("/").trim_end_matches("/");
    let reboot_url = format!("{base_url}/{api_path}/nodes/{}/qemu/{vm_id}/status/reboot", proxmox_args.node);
    let status_url = format!("{base_url}/{api_path}/nodes/{}/qemu/{vm_id}/status/current", proxmox_args.node);
    let auth_value = format!("PVEAPIToken={}", proxmox_args.api_token.unwrap_or_default());

    let pre_uptime = match client.get(&status_url).header(header::AUTHORIZATION, &auth_value).send().await {
        Ok(res) => res.json::<ProxmoxApiResponse<VmCurrentStatus>>().await?.data.uptime.unwrap_or(0),
        Err(e) => return Err(e.into())
    };

    if let Err(e) = client.post(reboot_url).header(header::AUTHORIZATION, &auth_value).send().await {
        return Err(e.into());
    }

    for _ in 0..90 {
        tokio::time::sleep(Duration::from_secs(1)).await;
        let status = match client.get(&status_url).header(header::AUTHORIZATION, &auth_value).send().await {
            Ok(res) => res.json::<ProxmoxApiResponse<VmCurrentStatus>>().await?,
            Err(_) => continue
        };

        let vm_status = status.data.status.unwrap_or_default();
        let uptime = status.data.uptime.unwrap_or(0);

        if vm_status == "running" && uptime < pre_uptime {
            return Ok(*vm_id);
        }
    }

    Err(AppError::InternalError("Failed to restart VM within timeout".to_string()))
}

#[cfg(feature = "ssr")]
#[instrument]
pub async fn add_vm_time(vm_id: &u32) -> Result<u32, AppError> {
    let proxmox_args = get_proxmox_args().await?;
    is_host_reachable(&proxmox_args.base_url).await?;

    let client = get_reqwest_client();

    let base_url = proxmox_args.base_url.trim_end_matches("/");
    let api_path = proxmox_args.api_path.trim_start_matches("/").trim_end_matches("/");
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
        "id={vm_id}&challenge_id={}&origin_id={}&user_id={}&created_at={}&expire_at={}",
        args.challenge_id,
        args.origin_id,
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

    Ok(*vm_id)
}

// return Ok only when vm with template id doesn't exist in user pool anymore
#[cfg(feature = "ssr")]
#[instrument]
pub async fn destroy_vm(vm_id: &u32) -> Result<u32, AppError> {
    let proxmox_args = get_proxmox_args().await?;
    is_host_reachable(&proxmox_args.base_url).await?;

    let client = get_reqwest_client();

    let base_url = proxmox_args.base_url.trim_end_matches("/");
    let api_path = proxmox_args.api_path.trim_start_matches("/").trim_end_matches("/");
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
                        Ok(*vm_id)
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
