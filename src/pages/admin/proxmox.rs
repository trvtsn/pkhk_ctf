use crate::{components::utils::{ComponentSize, HidePasswordButton, Spinner}, server::{admin::{get_proxmox_conf, test_proxmox, update_proxmox}, db::{enums::ProxmoxAuthType, structs::ProxmoxArgs}, enums::ResultStatus, structs::ApiResult}};
use leptos::{prelude::*, task::spawn_local};

/// Default Home Page
#[component]
pub fn Proxmox() -> impl IntoView {
    let api_token_hidden = RwSignal::new(true);
    let auth_status_ui = RwSignal::new("".to_string());
    let auth_success = RwSignal::new(false);

    let base_url = RwSignal::new("".to_string());
    let api_path = RwSignal::new("/api2/json".to_string());
    let templates_pool_id = RwSignal::new("templates".to_string());
    let node = RwSignal::new("".to_string());
    let api_token = RwSignal::new(None);
    let auth_type = RwSignal::new(ProxmoxAuthType::default());

    let proxmox_resource = Resource::new(move || (), move |_| async move {
        get_proxmox_conf().await.unwrap_or_default().unwrap_or_default()
    });

    let auth_status_classes = Memo::new(move |_| {
        let base = "rounded-full w-3 h-3";
        if auth_success.get() {
            format!("{} bg-green-600", base)
        } else {
            format!("{} bg-red-600", base)
        }
    });

    Effect::watch(
        move || proxmox_resource.get(),
        move |val, _, _| {
            if let Some(proxmox_args) = val.clone() {
                api_path.set(proxmox_args.api_path.clone());
                api_token.set(proxmox_args.api_token.clone());
                auth_type.set(proxmox_args.auth_type.clone());
                base_url.set(proxmox_args.base_url.clone());
                node.set(proxmox_args.node.clone());
                templates_pool_id.set(proxmox_args.templates_pool_id.clone());
                spawn_local(async move {
                    if let Ok(ApiResult { result, .. }) = test_proxmox(ProxmoxArgs {
                            base_url: proxmox_args.base_url,
                            api_path: proxmox_args.api_path,
                            templates_pool_id: proxmox_args.templates_pool_id,
                            node: proxmox_args.node,
                            username: None,
                            password: None,
                            api_token: proxmox_args.api_token,
                            auth_type: proxmox_args.auth_type,
                        })
                        .await
                    {
                        auth_success.set(result == ResultStatus::Success);
                    }
                });
            }
        },
        false
    );

    view! {
        <Transition fallback=move || {
            view! { <Spinner component_size=ComponentSize::Big /> }
        }>
            {move || {
                proxmox_resource.get();
                view! {
                    <div class="grid gap-2">
                        <div class="flex gap-2 items-center">
                            "Authentication Status"
                            <svg class=auth_status_classes>
                                <circle /> // on hover tooltip -> "Connected"/"No Connection"?
                            </svg>
                        </div>

                        <div class="grid gap-3 pt-2">
                            <div class="grid">
                                <label class=r#"block mb-1 text-sm font-medium text-text"#>"Base URL"</label>
                                <input
                                    class=r#"py-2 px-3 w-full text-sm rounded-md border border-input-border 
                                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                                    name="base_url"
                                    placeholder="e.g. https://192.168.1.21:8006"
                                    bind:value=base_url
                                />
                            </div>

                            <div class="grid">
                                <label class=r#"block mb-1 text-sm font-medium text-text"#>"API Path"</label>
                                <input
                                    class=r#"py-2 px-3 w-full text-sm rounded-md border border-input-border 
                                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                                    name="api_path"
                                    placeholder="Optional (Default: /api2/json)"
                                    bind:value=api_path
                                />
                            </div>

                            <div class="grid">
                                <label class=r#"block mb-1 text-sm font-medium text-text"#>"VM Templates Pool ID"</label>
                                <input
                                    class=r#"py-2 px-3 w-full text-sm rounded-md border border-input-border 
                                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                                    name="api_path"
                                    placeholder="Optional (Default: templates)"
                                    bind:value=templates_pool_id
                                />
                            </div>

                            <div class="grid">
                                <label class=r#"block mb-1 text-sm font-medium text-text"#>"Node"</label>
                                <input
                                    class=r#"py-2 px-3 w-full text-sm rounded-md border border-input-border 
                                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                                    name="node"
                                    bind:value=node
                                />
                            </div>

                            <Show when=move || auth_type.get() == ProxmoxAuthType::ApiToken>
                                <div class="grid">
                                    <label class=r#"block mb-1 text-sm font-medium text-text"#>"API Token"</label>
                                    <div class="flex gap-2">
                                        <input
                                            class=r#"py-2 px-3 w-full text-sm rounded-md border border-input-border 
                                            focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                                            type=move || if api_token_hidden.get() { "password" } else { "text" }
                                            name="api_token"
                                            placeholder="user@realm!token_id=uuid_secret"
                                            value=move || api_token.get().unwrap_or_default()
                                            on:change=move |ev| {
                                                let value = event_target_value(&ev);
                                                api_token.set(Some(value));
                                            }
                                        />
                                        <HidePasswordButton hidden=api_token_hidden />
                                    </div>
                                </div>
                            </Show>

                            <div class="flex gap-3 mt-2 pt-2">
                                <button
                                    class=r#"py-2 px-4 text-sm rounded-md border border-input-border hover:bg-background-hover"#
                                    on:click=move |_| {
                                        let base_url = base_url.get_untracked();
                                        let api_path = api_path.get_untracked();
                                        let templates_pool_id = templates_pool_id.get_untracked();
                                        let node = node.get_untracked();
                                        let api_token = api_token.get_untracked();
                                        let auth_type = auth_type.get_untracked();
                                        spawn_local(async move {
                                            let api_path = if api_path.is_empty() { "/api2/json".to_string() } else { api_path };
                                            if let Ok(ApiResult { result, details }) = test_proxmox(ProxmoxArgs {
                                                    base_url,
                                                    api_path,
                                                    templates_pool_id,
                                                    node,
                                                    username: None,
                                                    password: None,
                                                    api_token,
                                                    auth_type
                                                })
                                                .await
                                            {
                                                auth_status_ui.set(details);
                                                if result == ResultStatus::Success {
                                                    auth_success.set(true);
                                                } else {
                                                    auth_success.set(false);
                                                }
                                            }
                                        });
                                    }
                                >
                                    "Test Authentication"
                                </button>
                                    
                                <button
                                    class=r#"inline-flex items-center py-2 px-4 ml-auto text-sm font-semibold 
                                    text-white rounded-md shadow-sm focus:ring-2 focus:outline-none 
                                    bg-yale-blue-600 hover:bg-yale-blue-500 focus:ring-yale-blue-500"#
                                    on:click=move |_| {
                                        let base_url = base_url.get_untracked();
                                        let api_path = api_path.get_untracked();
                                        let templates_pool_id = templates_pool_id.get_untracked();
                                        let node = node.get_untracked();
                                        let api_token = api_token.get_untracked();
                                        let auth_type = auth_type.get_untracked();
                                        spawn_local(async move {
                                            let api_path = if api_path.is_empty() { "/api2/json".to_string() } else { api_path };
                                            if let Ok(ApiResult { result, details }) = update_proxmox(ProxmoxArgs {
                                                    base_url,
                                                    api_path,
                                                    templates_pool_id,
                                                    node,
                                                    username: None,
                                                    password: None,
                                                    api_token,
                                                    auth_type
                                                })
                                                .await
                                            {
                                                if result == ResultStatus::Success {
                                                    auth_success.set(true);
                                                } else {
                                                    auth_success.set(false);
                                                }
                                                
                                                auth_status_ui.set(details);
                                            }
                                        });
                                    }
                                >
                                    "Apply"
                                </button>
                            </div>
                            {move || auth_status_ui.get()}
                        </div>
                    </div>
                }
            }}
        </Transition>
    }
}
