use crate::{components::utils::Spinner, server::{admin::{get_proxmox_conf, test_proxmox, update_proxmox}, db::{enums::ProxmoxAuthType, structs::ProxmoxArgs}, enums::ResultStatus, structs::ApiResult}};
use leptos::{prelude::*, task::spawn_local};

/// Default Home Page
#[component]
pub fn Proxmox() -> impl IntoView {
    let auth_status_ui = RwSignal::new("".to_string());
    let auth_success = RwSignal::new(false);

    let base_url = RwSignal::new("".to_string());
    let api_path = RwSignal::new("/api2/json".to_string());
    let templates_pool_id = RwSignal::new("templates".to_string());
    let node = RwSignal::new("".to_string());
    let api_token = RwSignal::new(None);
    let auth_type = RwSignal::new(ProxmoxAuthType::default());

    let proxmox_resource = Resource::new(move || (), move |_| async move {
        let proxmox_args = get_proxmox_conf().await.unwrap_or_default().unwrap_or_default();
        let test_args  = proxmox_args.clone();

        let base_url = test_args.base_url;
        let api_path = test_args.api_path;
        let templates_pool_id = test_args.templates_pool_id;
        let node = test_args.node;
        let api_token = test_args.api_token;
        let auth_type = test_args.auth_type;
        spawn_local(async move {
            if let Ok(ApiResult { result, .. }) = test_proxmox(ProxmoxArgs {
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
            }
        });

        proxmox_args
    });

    let auth_status_classes = Memo::new(move |_| {
        let base = "rounded-full w-3 h-3";
        if auth_success.get() {
            format!("{} bg-green-600", base)
        } else {
            format!("{} bg-red-600", base)
        }
    });

    view! {
        <Suspense fallback=move || {
            view! { <Spinner /> }
        }>
            {move || {
                let proxmox_args = proxmox_resource.get();
                if let Some(proxmox_args) = proxmox_args {
                    base_url.set(proxmox_args.base_url.clone());
                    api_path.set(proxmox_args.api_path.clone());
                    api_token.set(proxmox_args.api_token);
                    auth_type.set(proxmox_args.auth_type);
                }

                view! {
                    <div class="grid gap-2">
                        <div class="flex gap-2 items-center">
                            "Authentication Status"
                            <svg class=auth_status_classes>
                                <circle /> // on hover tooltip -> "Connected"/"No Connection"?
                            </svg>
                        </div>

                        <div class="grid gap-2 pt-2">
                            <label class=r#"block mb-1 text-sm font-medium text-text"#>"Base URL"</label>
                            <input
                                class=r#"py-2 px-3 w-full text-sm rounded-md border border-input-border 
                                focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                                name="base_url"
                                value=move || base_url.get()
                                bind:value=base_url
                            />

                            <label class=r#"block mb-1 text-sm font-medium text-text"#>"API Path"</label>
                            <input
                                class=r#"py-2 px-3 w-full text-sm rounded-md border border-input-border 
                                focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                                name="api_path"
                                placeholder="Optional (Default: /api2/json)"
                                value=move || api_path.get()
                                bind:value=api_path
                            />

                            <label class=r#"block mb-1 text-sm font-medium text-text"#>"VM Templates Pool ID"</label>
                            <input
                                class=r#"py-2 px-3 w-full text-sm rounded-md border border-input-border 
                                focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                                name="api_path"
                                placeholder="Optional (Default: templates)"
                                value=move || templates_pool_id.get()
                                bind:value=templates_pool_id
                            />

                            <label class=r#"block mb-1 text-sm font-medium text-text"#>"Node"</label>
                            <input
                                class=r#"py-2 px-3 w-full text-sm rounded-md border border-input-border 
                                focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                                name="node"
                                value=move || node.get()
                                bind:value=node
                            />

                            <Show when=move || auth_type.get() == ProxmoxAuthType::ApiToken>
                                <label class=r#"block mb-1 text-sm font-medium text-text"#>"API Token"</label>
                                <input
                                    class=r#"py-2 px-3 w-full text-sm rounded-md border border-input-border 
                                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                                    name="api_token"
                                    value=move || api_token.get().unwrap_or_default()
                                    on:change=move |ev| {
                                        let value = event_target_value(&ev);
                                        api_token.set(Some(value));
                                    }
                                />
                            </Show>

                            <div class="flex gap-3 mt-2 pt-2">
                                <button
                                    class=r#"py-2 px-4 text-sm rounded-md border border-input-border hover:bg-background-hover"#
                                    on:click=move |_| {
                                        let base_url = base_url.get();
                                        let api_path = api_path.get();
                                        let templates_pool_id = templates_pool_id.get();
                                        let node = node.get();
                                        let api_token = api_token.get();
                                        let auth_type = auth_type.get();
                                        spawn_local(async move {
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
                                                auth_status_ui.set(details.unwrap_or_default());
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
                                        let base_url = base_url.get();
                                        let api_path = api_path.get();
                                        let templates_pool_id = templates_pool_id.get();
                                        let node = node.get();
                                        let api_token = api_token.get();
                                        let auth_type = auth_type.get();
                                        spawn_local(async move {
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
                                                
                                                auth_status_ui.set(details.unwrap_or_default());
                                            }
                                        });
                                    }
                                >
                                    "Apply"
                                </button>
                            </div>
                            <Transition fallback=|| view! { "..." }>{move || auth_status_ui.get()}</Transition>
                        </div>
                    </div>
                }
            }}
        </Suspense>
    }
}
