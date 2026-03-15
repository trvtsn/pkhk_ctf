use crate::{components::{toast::{ToastMessageType, push_new_toast}, utils::{ComponentSize, HidePasswordButton, Spinner, UserTooltip, VMTooltip}}, pages::admin::{AdminSections, ProxmoxSubSection}, server::{admin::{ProxmoxUserInfo, add_vm_time, create_proxmox_pool, create_proxmox_user, delete_proxmox_pool, delete_proxmox_user, destroy_vm, get_proxmox_conf, get_proxmox_users_info, restart_vm, start_vm, test_proxmox, update_proxmox}, db::{enums::ProxmoxAuthType, structs::ProxmoxArgs}, enums::ResultStatus, get_proxmox_base_url, structs::ApiResult}};
use itertools::Itertools;
use leptos::{prelude::*, task::spawn_local};

/// Default Home Page
#[component]
pub fn Proxmox() -> impl IntoView {
    let selected = expect_context::<RwSignal<AdminSections>>();

    view! {
        <div class="grid pb-4 mb-4">
            <ul class="flex flex-row gap-4">
                <li>
                    <button
                        class="py-1 px-3 text-sm rounded-md border border-input-border bg-background hover:bg-background-hover" 
                        disabled=move || selected.get() == AdminSections::Proxmox(ProxmoxSubSection::Config)
                        on:click=move |_| selected.set(AdminSections::Proxmox(ProxmoxSubSection::Config))
                    >
                        "Config"
                    </button>
                </li>
                <li>
                    <button
                        class="py-1 px-3 text-sm rounded-md border border-input-border bg-background hover:bg-background-hover" 
                        disabled=move || selected.get() == AdminSections::Proxmox(ProxmoxSubSection::Users)
                        on:click=move |_| selected.set(AdminSections::Proxmox(ProxmoxSubSection::Users))
                    >
                        "Users"
                    </button>
                </li>
            </ul>
        </div>

        {move || {
            if selected.get() == AdminSections::Proxmox(ProxmoxSubSection::Config) {
                view! { <Config /> }.into_any()
            } else if selected.get() == AdminSections::Proxmox(ProxmoxSubSection::Users) {
                view! { <Users /> }.into_any()
            } else {
                "".into_any()
            }
        }}
    }
}

#[component]
fn Config() -> impl IntoView {
    let api_token_hidden = RwSignal::new(true);
    let auth_status_ui = RwSignal::new("".to_string());
    let auth_success = RwSignal::new(false);

    let base_url = RwSignal::new("".to_string());
    let api_path = RwSignal::new("/api2/json".to_string());
    let templates_pool_id = RwSignal::new("templates".to_string());
    let node = RwSignal::new("".to_string());
    let api_token = RwSignal::new(None);
    let auth_type = RwSignal::new(ProxmoxAuthType::default());

    let auth_status_classes = Memo::new(move |_| {
        let base = "rounded-full w-3 h-3";
        if auth_success.get() {
            format!("{} bg-green-600", base)
        } else {
            format!("{} bg-red-600", base)
        }
    });

    let proxmox_resource = Resource::new(move || (), move |_| async move {
        get_proxmox_conf().await.unwrap_or_default().unwrap_or_default()
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

#[component]
fn Users() -> impl IntoView {
    let proxmox_users_info = RwSignal::new(Vec::<ProxmoxUserInfo>::new());
    let proxmox_users_resource = Resource::new(move || (), move |_| async move {
        get_proxmox_users_info().await
    });

    let proxmox_base_url = Resource::new(move || (), move |_| async move {
        get_proxmox_base_url().await.ok()
    });

    Effect::watch(
        move || proxmox_users_resource.get(),
        move |val, _, _| {
            if let Some(val) = val.clone() && let Ok(val) = val {
                proxmox_users_info.set(val);
            }
        },
        false
    );

    view! {
        <Transition fallback=move || {
            view! { <Spinner component_size=ComponentSize::Big /> }
        }>
            {move || {
                if let Some(val) = proxmox_users_resource.get() && let Err(e) = val {
                    view! { <p>{format!("An error occurred: {e}")}</p> }.into_any()
                } else {
                    view! {
                        <table class="w-full border-collapse border border-input-border">
                            <tr class="border-b border-input-border">
                                <th class="py-2 px-4">"No."</th>
                                <th class="py-2 px-4">"User"</th>
                                <th class="py-2 px-4">"Auth Type"</th>
                                <th class="py-2 px-4">"PVE User ID"</th>
                                <th class="py-2 px-4">"Pool"</th>
                                <th class="py-2 px-4">"Existing VMs"</th>
                            </tr>
                            {move || proxmox_users_info.get()
                                .into_iter()
                                .sorted_by(|a, b| a.user.username.to_lowercase().cmp(&b.user.username.to_lowercase()))
                                .enumerate()
                                .map(|(index, info)| 
                            {
                                let info = RwSignal::new(info);

                                let delete_proxmox_user_action = Action::new_local(move |user_id: &String| {
                                    let user_id = user_id.clone();
                                    async move {
                                        if delete_proxmox_user(user_id).await.is_ok() {
                                            spawn_local(async move {
                                                push_new_toast(ToastMessageType::ProxmoxUserDeleted);
                                            });
                                            proxmox_users_resource.refetch();
                                        } else {
                                            push_new_toast(ToastMessageType::ProxmoxUserDeleteFail);
                                        }
                                    }
                                });
                                let create_proxmox_user_action = Action::new_local(move |user_id: &String| {
                                    let user_id = user_id.clone();
                                    async move {
                                        if create_proxmox_user(user_id).await.is_ok() {
                                            spawn_local(async move {
                                                push_new_toast(ToastMessageType::ProxmoxUserCreated);
                                            });
                                            proxmox_users_resource.refetch();
                                        } else {
                                            push_new_toast(ToastMessageType::ProxmoxUserCreateFail);
                                        }
                                    }
                                });

                                let create_proxmox_pool_action = Action::new_local(move |user_id: &String| {
                                    let user_id = user_id.clone();
                                    async move {
                                        if create_proxmox_pool(user_id).await.is_ok() {
                                            spawn_local(async move {
                                                push_new_toast(ToastMessageType::ProxmoxPoolCreated);
                                            });
                                            proxmox_users_resource.refetch();
                                        } else {
                                            push_new_toast(ToastMessageType::ProxmoxPoolCreateFail);
                                        }
                                    }
                                });
                                let delete_proxmox_pool_action = Action::new_local(move |user_id: &String| {
                                    let user_id = user_id.clone();
                                    async move {
                                        if delete_proxmox_pool(user_id).await.is_ok() {
                                            spawn_local(async move {
                                                push_new_toast(ToastMessageType::ProxmoxPoolDeleted);
                                            });
                                            proxmox_users_resource.refetch();
                                        } else {
                                            push_new_toast(ToastMessageType::ProxmoxPoolDeleteFail);
                                        }
                                    }
                                });

                                view! {
                                    <tr class="border-b border-input-border">
                                        <td class="py-2 px-4">{index + 1}</td>
                                        <td class="py-2 px-4">
                                            <UserTooltip db_user=info.get().user />
                                        </td>
                                        <td class="py-2 px-4">
                                            {info.get().user.auth_type}
                                        </td>
                                        <td class="py-2 px-4">
                                            {if let Some(uid) = info.get().pve_user_id {
                                                view! {
                                                    <div class="flex gap-2 items-center">
                                                        {uid}
                                                        <button
                                                            class="py-1 px-2 text-xs text-red-600 rounded border border-red-600 hover:text-red-400 hover:border-red-400"
                                                            on:click=move |_| {
                                                                let user_id = info.get().user.id;
                                                                delete_proxmox_user_action.dispatch(user_id);
                                                            }
                                                        >
                                                            {move || {
                                                                if delete_proxmox_user_action.pending().get() {
                                                                    view! { <Spinner component_size=ComponentSize::Small /> }.into_any()
                                                                } else {
                                                                    "Delete".into_any()
                                                                }
                                                            }}
                                                        </button>
                                                    </div>
                                                }.into_any()
                                            } else {
                                                view! {
                                                    <button
                                                        class="py-1 px-2 text-xs text-green-600 rounded border border-green-600 hover:text-green-400 hover:border-green-400"
                                                        on:click=move |_| {
                                                            let user_id = info.get().user.id;
                                                            create_proxmox_user_action.dispatch(user_id);
                                                        }
                                                    >
                                                        {move || {
                                                            if create_proxmox_user_action.pending().get() {
                                                                view! { <Spinner component_size=ComponentSize::Small /> }.into_any()
                                                            } else {
                                                                "Create".into_any()
                                                            }
                                                        }}
                                                    </button>
                                                }.into_any()
                                            }}
                                        </td>
                                        <td class="py-2 px-4">
                                            {if let Some(pool) = info.get().pool {
                                                view! {
                                                    <div class="flex gap-2 items-center">
                                                        {pool}
                                                        <button
                                                            class="py-1 px-2 text-xs text-red-600 rounded border border-red-600 hover:text-red-400 hover:border-red-400"
                                                            on:click=move |_| {
                                                                let user_id = info.get().user.id;
                                                                delete_proxmox_pool_action.dispatch(user_id);
                                                            }
                                                        >
                                                            {move || {
                                                                if delete_proxmox_pool_action.pending().get() {
                                                                    view! { <Spinner component_size=ComponentSize::Small /> }.into_any()
                                                                } else {
                                                                    "Delete".into_any()
                                                                }
                                                            }}
                                                        </button>
                                                    </div>
                                                }.into_any()
                                            } else {
                                                view! {
                                                    <button
                                                        class="py-1 px-2 text-xs text-green-600 rounded border border-green-600 hover:text-green-400 hover:border-green-400"
                                                        on:click=move |_| {
                                                            let user_id = info.get().user.id;
                                                            create_proxmox_pool_action.dispatch(user_id);
                                                        }
                                                    >
                                                        {move || {
                                                            if create_proxmox_pool_action.pending().get() {
                                                                view! { <Spinner component_size=ComponentSize::Small /> }.into_any()
                                                            } else {
                                                                "Create".into_any()
                                                            }
                                                        }}
                                                    </button>
                                                }.into_any()
                                            }}
                                        </td>
                                        <td class="grid gap-2 py-2 px-4">
                                            {info.get().vms.into_iter().map(|vm| {
                                                let vm_id = vm.id;
                                                
                                                let start_vm_action = Action::new_local(move |(vm_id, username): &(u32, String)| {
                                                        let vm_id = vm_id.clone();
                                                        let username = username.clone();
                                                        async move {
                                                            if let Ok(result) = start_vm(vm_id, username).await {
                                                                spawn_local(async move {
                                                                    push_new_toast(ToastMessageType::Custom(result.details));
                                                                });
                                                                proxmox_users_resource.refetch();
                                                            } else {
                                                                spawn_local(async move {
                                                                    push_new_toast(ToastMessageType::VMStartFail);
                                                                });
                                                            }
                                                        }
                                                    });
                                                let restart_vm_action = Action::new_local(move |vm_id: &u32| {
                                                    let vm_id = vm_id.clone();
                                                    async move {
                                                        if let Ok(result) = restart_vm(vm_id).await {
                                                            spawn_local(async move {
                                                                push_new_toast(ToastMessageType::Custom(result.details));
                                                            });
                                                            proxmox_users_resource.refetch();
                                                        } else {
                                                            spawn_local(async move {
                                                                push_new_toast(ToastMessageType::VMRestartFail);
                                                            });
                                                        }
                                                    }
                                                });
                                                let add_vm_time_action = Action::new_local(move |vm_id: &u32| {
                                                    let vm_id = vm_id.clone();
                                                    async move {
                                                        if let Ok(result) = add_vm_time(vm_id).await {
                                                            spawn_local(async move {
                                                                push_new_toast(ToastMessageType::Custom(result.details));
                                                            });
                                                            proxmox_users_resource.refetch();
                                                        } else {
                                                            spawn_local(async move {
                                                                push_new_toast(ToastMessageType::VMAddTimeFail);
                                                            });
                                                        }
                                                    }
                                                });
                                                let destroy_vm_action = Action::new_local(move |vm_id: &u32| {
                                                    let vm_id = vm_id.clone();
                                                    async move {
                                                        if let Ok(result) = destroy_vm(vm_id).await {
                                                            spawn_local(async move {
                                                                push_new_toast(ToastMessageType::Custom(result.details));
                                                            });
                                                            proxmox_users_resource.refetch();
                                                        } else {
                                                            spawn_local(async move {
                                                                push_new_toast(ToastMessageType::VMDestroyFail);
                                                            });
                                                        }
                                                    }
                                                });

                                                view! {
                                                    <div class="flex gap-2 items-center">
                                                        {move || {
                                                            let href = proxmox_base_url.get()
                                                                .flatten()
                                                                .map(|url| format!("{}/#v1:0:=qemu%2F{}:4:::::::", url, vm_id))
                                                                .unwrap_or_default();
                                                            view! {
                                                                <VMTooltip
                                                                    vm_id
                                                                    href
                                                                    created_at=vm.created_at
                                                                    end_at=vm.end_at
                                                                />
                                                            }.into_any()
                                                        }}
                                                        <Show when=move || !vm.running>
                                                            <button
                                                                class=r#"col-start-1 col-end-1 gap-2 items-center py-2 px-4 text-sm font-medium text-white 
                                                                rounded-lg transition focus:ring-2 focus:outline-none active:scale-95 
                                                                bg-yale-blue-600 hover:bg-yale-blue-700 focus:ring-yale-blue-400"#
                                                                disabled=move || start_vm_action.pending().get()
                                                                on:click=move |_| {
                                                                    let username = info.get().user.username;
                                                                    start_vm_action.dispatch((vm_id, username));
                                                                }
                                                            >
                                                                {move || {
                                                                    if start_vm_action.pending().get() {
                                                                        view! { <Spinner component_size=ComponentSize::Small /> }.into_any()
                                                                    } else {
                                                                        "Start VM".into_any()
                                                                    }
                                                                }}
                                                            </button>
                                                        </Show>

                                                        <Show when=move || vm.running>
                                                            <button
                                                                class=r#"col-start-2 col-end-2 gap-2 items-center py-2 px-4 text-sm font-medium text-white 
                                                                rounded-lg transition focus:ring-2 focus:outline-none active:scale-95 
                                                                bg-yale-blue-600 hover:bg-yale-blue-700 focus:ring-yale-blue-400"#
                                                                disabled=move || restart_vm_action.pending().get()
                                                                on:click=move |_| {
                                                                    restart_vm_action.dispatch(vm_id);
                                                                }
                                                            >
                                                                {move || {
                                                                    if restart_vm_action.pending().get() {
                                                                        view! { <Spinner component_size=ComponentSize::Small /> }.into_any()
                                                                    } else {
                                                                        "Restart VM".into_any()
                                                                    }
                                                                }}
                                                            </button>

                                                            <button
                                                                class=r#"col-start-3 col-end-3 gap-2 items-center py-2 px-4 text-sm font-medium text-white 
                                                                rounded-lg transition focus:ring-2 focus:outline-none active:scale-95 
                                                                bg-yale-blue-600 hover:bg-yale-blue-700 focus:ring-yale-blue-400"#
                                                                disabled=move || add_vm_time_action.pending().get()
                                                                on:click=move |_| {
                                                                    add_vm_time_action.dispatch(vm_id);
                                                                }
                                                            >
                                                                {move || {
                                                                    if add_vm_time_action.pending().get() {
                                                                        view! { <Spinner component_size=ComponentSize::Small /> }.into_any()
                                                                    } else {
                                                                        "Add Time (+30 min)".into_any()
                                                                    }
                                                                }}
                                                            </button>

                                                            <button
                                                                class=r#"col-start-4 col-end-4 gap-2 items-center py-2 px-4 text-sm font-medium text-white 
                                                                rounded-lg transition focus:ring-2 focus:outline-none active:scale-95 
                                                                bg-yale-blue-600 hover:bg-yale-blue-700 focus:ring-yale-blue-400"#
                                                                disabled=move || destroy_vm_action.pending().get()
                                                                on:click=move |_| {
                                                                    destroy_vm_action.dispatch(vm_id);
                                                                }
                                                            >
                                                                {move || {
                                                                    if destroy_vm_action.pending().get() {
                                                                        view! { <Spinner component_size=ComponentSize::Small /> }.into_any()
                                                                    } else {
                                                                        "Destroy VM".into_any()
                                                                    }
                                                                }}
                                                            </button>
                                                        </Show>
                                                    </div>
                                                }
                                            }).collect_view()}
                                        </td>
                                    </tr>
                                }
                            }).collect_view()}
                        </table>
                    }.into_any()
                }
            }}
        </Transition>
    }
}
