/// src/server/proxmox/admin.rs
/// contains code which essentially creates an easy wrapper for the API to use for
/// communicating with the Proxmox server.
/// All the code here is meant to be accessible only to users with the role `UserRole::Admin`.

#[cfg(feature = "ssr")]
use crate::server::proxmox::ProxmoxClient;
use crate::{error_template::AppError, server::proxmox::{Config, Member, VmCurrentStatus}};
#[cfg(feature = "ssr")]
use crate::server::proxmox::extract_args_from_description;
use std::time::Duration;
use tracing::instrument;

#[cfg(feature = "ssr")]
#[instrument]
pub async fn start_vm(vm_id: &u32, username: &str) -> Result<u32, AppError> {
    let pxc = ProxmoxClient::new().await?;

    let start_url = format!("{}/status/start", pxc.append_to_qemu_url(*vm_id));
    let status_url = format!("{}/status/current", pxc.append_to_qemu_url(*vm_id));

    pxc.post_req(&start_url, None).await?;

    for _ in 0..60 {
        tokio::time::sleep(Duration::from_secs(1)).await;
        let vm = pxc.get_req::<Member>(&status_url).await?;

        let vm_status = vm.data.status.unwrap_or_default();
        if vm_status == "running" { return Ok(*vm_id); }
    }

    Err(AppError::InternalError("VM failed to start within timeout".to_string()))
}

#[cfg(feature = "ssr")]
#[instrument]
pub async fn restart_vm(vm_id: &u32) -> Result<u32, AppError> {
    let pxc = ProxmoxClient::new().await?;

    let reboot_url = format!("{}/status/reboot", pxc.append_to_qemu_url(*vm_id));
    let status_url = format!("{}/status/current", pxc.append_to_qemu_url(*vm_id));

    let pre_uptime = pxc.get_req::<VmCurrentStatus>(&status_url).await?.data.uptime.unwrap_or(0);

    pxc.post_req(&reboot_url, None).await?;

    for _ in 0..90 {
        tokio::time::sleep(Duration::from_secs(1)).await;
        let status = match pxc.get_req::<VmCurrentStatus>(&status_url).await {
            Ok(s) => s,
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
    let pxc = ProxmoxClient::new().await?;

    let conf_url = format!("{}/config", pxc.append_to_qemu_url(*vm_id));

    let config = pxc.get_req::<Config>(&conf_url).await?;
    let description = config.data.description.unwrap_or_default();
    let args = extract_args_from_description(description)?;
    let new_expire_at = args.end_at + chrono::Duration::minutes(30);

    let new_description = serde_urlencoded::to_string(&[
        ("id", vm_id.to_string()),
        ("challenge_id", args.challenge_id),
        ("origin_id", args.origin_id.to_string()),
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

    Ok(*vm_id)
}

#[cfg(feature = "ssr")]
#[instrument]
pub async fn destroy_vm(vm_id: &u32) -> Result<u32, AppError> {
    let pxc = ProxmoxClient::new().await?;

    let stop_url = format!("{}/status/stop", pxc.append_to_qemu_url(*vm_id));
    let status_url = format!("{}/status/current", pxc.append_to_qemu_url(*vm_id));

    // stop
    let res = pxc.post_req(&stop_url, None).await?;
    if !res.status().is_success() {
        return Err(AppError::InternalError("Failed to stop VM".to_string()));
    }

    // poll until stopped
    for _ in 0..30 {
        tokio::time::sleep(Duration::from_secs(1)).await;
        let status = match pxc.get_req::<VmCurrentStatus>(&status_url).await {
            Ok(s) => s,
            Err(_) => continue
        };

        if status.data.status.unwrap_or_default() == "stopped" {
            // destroy
            let destroy_url = pxc.append_to_qemu_url(*vm_id);
            let del_res = pxc.delete_req(&destroy_url).await?;
            if !del_res.status().is_success() {
                return Err(AppError::InternalError("Failed to destroy VM".to_string()));
            }
            return Ok(*vm_id);
        }
    }

    Err(AppError::InternalError("VM failed to stop within timeout".to_string()))
}
