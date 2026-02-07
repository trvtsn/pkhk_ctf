use crate::{components::utils::HidePasswordButton, server::{admin::{get_proxmox_conf, test_proxmox, update_proxmox}, db::{enums::ProxmoxAuthType, structs::ProxmoxArgs}, enums::ResultStatus, structs::ApiResult}};
// use icondata as i;
use leptos::{prelude::*, task::spawn_local};
// use leptos_icons::Icon;

/// Default Home Page
#[component]
pub fn Proxmox() -> impl IntoView {
    let auth_status_ui = RwSignal::new("".to_string());
    let auth_success = RwSignal::new(false);
    // let password_hidden = RwSignal::new(true);

    let base_url = RwSignal::new("".to_string());
    let api_path = RwSignal::new("/api2/json".to_string());
    let node = RwSignal::new("".to_string());
    let api_token = RwSignal::new(None);
    let auth_type = RwSignal::new(ProxmoxAuthType::default());

    let proxmox_resource = Resource::new(move || (), move |_| async move {
        get_proxmox_conf().await.unwrap_or_default()
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
        // <nav class=r#"flex flex-col col-start-1 col-end-1 gap-2 p-4 bg-background-secondary text-text rounded-lg shadow-sm"#>
        //     <ul class=r#"flex flex-col gap-1"# role="menu" aria-label="Authentication type">
        //         <li class="bg-background">
        //             <p
        //                 class=r#"flex gap-3 items-center py-2 px-3 text-sm font-medium  
        //                 rounded-md hover:bg-gray-50 focus:ring-2 focus:outline-none 
        //                 focus:ring-yale-blue-500"#
        //                 on:click=move |_| auth_type.set(ProxmoxAuthType::Ticket)
        //             >
        //                 <Icon icon=i::LuSettings />
        //                 "Ticket"
        //             </p>
        //         </li>
        //         <li class="bg-background">
        //             <p
        //                 class=r#"flex gap-3 items-center py-2 px-3 text-sm font-medium  
        //                 rounded-md hover:bg-gray-50 focus:ring-2 focus:outline-none 
        //                 focus:ring-yale-blue-500"#
        //                 on:click=move |_| auth_type.set(ProxmoxAuthType::ApiToken)
        //             >
        //                 <Icon icon=i::LuSettings />
        //                 "API Token"
        //             </p>
        //         </li>
        //     </ul>
        // </nav>

        <Suspense fallback=move || {
            view! { <div>"Loading..."</div> }
        }>
            {move || {
                let proxmox_args = proxmox_resource.get().unwrap_or_default().unwrap_or_default();
                base_url.set(proxmox_args.base_url.clone());
                api_path.set(proxmox_args.api_path.clone());
                // username.set(proxmox_args.username.clone());
                // password.set(proxmox_args.password.clone());
                api_token.set(proxmox_args.api_token);
                auth_type.set(proxmox_args.auth_type);

                view! {
                    <div>
                        <div class="flex">
                            "Authentication Status"
                            <svg class=auth_status_classes>
                                <circle /> // on hover tooltip -> "Connected"/"No Connection"?
                            </svg>
                        </div>

                        <label class=r#"block mb-1 text-sm font-medium text-text"#>"Base URL"</label>
                        <input
                            class=r#"py-2 px-3 w-full text-sm rounded-md border border-gray-300 
                            focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                            name="base_url"
                            value=move || base_url.get()
                            bind:value=base_url
                        />

                        <label class=r#"block mb-1 text-sm font-medium text-text"#>"API Path"</label>
                        <input
                            class=r#"py-2 px-3 w-full text-sm rounded-md border border-gray-300 
                            focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                            name="api_path"
                            placeholder="Optional (Default: /api2/json)"
                            value=move || api_path.get()
                            bind:value=api_path
                        />

                        <label class=r#"block mb-1 text-sm font-medium text-text"#>"Node"</label>
                        <input
                            class=r#"py-2 px-3 w-full text-sm rounded-md border border-gray-300 
                            focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                            name="node"
                            value=move || node.get()
                            bind:value=node
                        />

                        // <Show when=move || auth_type.get() == ProxmoxAuthType::Ticket>
                        //     <label class=r#"block mb-1 text-sm font-medium text-text"#>"Username"</label>
                        //     <input
                        //         class=r#"py-2 px-3 w-full text-sm rounded-md border border-gray-300 
                        //         focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                        //         name="username"
                        //         value=move || username.get().unwrap_or_default()
                        //         placeholder="root@pam"
                        //         on:change=move |ev| {
                        //             let value = event_target_value(&ev);
                        //             username.set(Some(value));
                        //         }
                        //     />

                        //     <label class=r#"block mb-1 text-sm font-medium text-text"#>"Password"</label>
                        //     <input
                        //         class=r#"py-2 px-3 w-full text-sm rounded-md border border-gray-300 
                        //         focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                        //         type=move || if password_hidden.get() { "password" } else { "text" }
                        //         name="password"
                        //         value=move || password.get().unwrap_or_default()
                        //         on:change=move |ev| {
                        //             let value = event_target_value(&ev);
                        //             password.set(Some(value));
                        //         }
                        //     />
                        //     <HidePasswordButton hidden=password_hidden />
                        // </Show>

                        <Show when=move || auth_type.get() == ProxmoxAuthType::ApiToken>
                            <label class=r#"block mb-1 text-sm font-medium text-text"#>"API Token"</label>
                            <input
                                class=r#"py-2 px-3 w-full text-sm rounded-md border border-gray-300 
                                focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                                name="api_token"
                                value=move || api_token.get().unwrap_or_default()
                                on:change=move |ev| {
                                    let value = event_target_value(&ev);
                                    api_token.set(Some(value));
                                }
                            />
                        </Show>

                        <button
                            class=r#"inline-flex items-center py-2 px-4 ml-auto text-sm font-semibold 
                            text-white rounded-md shadow-sm focus:ring-2 focus:outline-none 
                            bg-yale-blue-600 hover:bg-yale-blue-500 focus:ring-yale-blue-500"#
                            on:click=move |_| {
                                let base_url = base_url.get();
                                let api_path = api_path.get();
                                let node = node.get();
                                // let username = username.get();
                                // let password = password.get();
                                let api_token = api_token.get();
                                let auth_type = auth_type.get();
                                spawn_local(async move {
                                    if let Ok(ApiResult { result, details }) = test_proxmox(ProxmoxArgs {
                                            base_url,
                                            api_path,
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
                            "Test Authentication"
                        </button>
                            
                        <button
                            class=r#"inline-flex items-center py-2 px-4 ml-auto text-sm font-semibold 
                            text-white rounded-md shadow-sm focus:ring-2 focus:outline-none 
                            bg-yale-blue-600 hover:bg-yale-blue-500 focus:ring-yale-blue-500"#
                            on:click=move |_| {
                                let base_url = base_url.get();
                                let api_path = api_path.get();
                                let node = node.get();
                                // let username = username.get();
                                // let password = password.get();
                                let api_token = api_token.get();
                                let auth_type = auth_type.get();
                                spawn_local(async move {
                                    if let Ok(ApiResult { result, details }) = update_proxmox(ProxmoxArgs {
                                            base_url,
                                            api_path,
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
                        <Transition fallback=|| view! { "..." }>{move || auth_status_ui.get()}</Transition>
                    </div>
                }
            }}
        </Suspense>
    }
}
