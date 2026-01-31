use crate::{components::utils::HidePasswordButton, server::{admin::get_ldap, db::structs::{LdapArgs, SqlBool}, enums::ResultStatus, structs::ApiResult}};
use leptos::{prelude::*, task::spawn_local};

/// Default Home Page
#[component]
pub fn Ldap() -> impl IntoView {
    let connect_status_ui = RwSignal::new("".to_string());
    let connect_success = RwSignal::new(false);
    let password_hidden = RwSignal::new(true);

    let ldap_url = RwSignal::new("".to_string());
    let bind_dn = RwSignal::new("".to_string());
    let bind_pw = RwSignal::new("".to_string());
    let base_dn = RwSignal::new("".to_string());
    let enabled = RwSignal::new(SqlBool(true));

    let ldap_args = RwSignal::new(LdapArgs::default());
    let ldap_resource = Resource::new(move || (), move |_| async move {
        get_ldap().await.unwrap_or_default()
    });

    let connect_status_classes = Memo::new(move |_| {
        let base = "rounded-full w-3 h-3";
        if connect_success.get() {
            format!("{} bg-green-600", base)
        } else {
            format!("{} bg-red-600", base)
        }
    });

    view! {
        <Suspense fallback=move || {
            view! { <div>"Loading..."</div> }
        }>
            {move || {
                Suspend::new(async move {
                    match ldap_resource.await {
                        Some(args) => ldap_args.set(args),
                        None => {}
                    }
                });

                view! {
                    <div 
                        class=move || if enabled.get().0 { "" } else { "rounded-md color-white-600/50" }
                    >
                        <label>"Enable"</label>
                        <input
                            type="checkbox"
                            checked=move || enabled.get().0
                            on:input=move |ev| {
                                let is_checked = event_target_checked(&ev);
                                if is_checked {
                                    enabled.set(SqlBool(true))
                                } else {
                                    enabled.set(SqlBool(false))
                                };
                            }
                        />

                        <div class="flex">
                            "Connection Status"
                            <svg class=connect_status_classes>
                                <circle /> // on hover tooltip -> "Connected"/"No Connection"?
                            </svg>
                        </div>

                        <label class=r#"block mb-1 text-sm font-medium text-text"#>"LDAP URL"</label>
                        <input
                            class=r#"py-2 px-3 w-full text-sm rounded-md border border-gray-300 
                            focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                            name="name"
                            value=move || ldap_args.get().url
                            bind:value=ldap_url
                        />

                        <label class=r#"block mb-1 text-sm font-medium text-text"#>"Bind DN"</label>
                        <input
                            class=r#"py-2 px-3 w-full text-sm rounded-md border border-gray-300 
                            focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                            name="name"
                            value=move || ldap_args.get().bind_dn
                            bind:value=bind_dn
                        />

                        <label class=r#"block mb-1 text-sm font-medium text-text"#>"Bind Password"</label>
                        <input
                            class=r#"py-2 px-3 w-full text-sm rounded-md border border-gray-300 
                            focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                            type=move || if password_hidden.get() { "password" } else { "text" }
                            name="password"
                            value=move || ldap_args.get().bind_pw
                            bind:value=bind_pw
                        />
                        <HidePasswordButton hidden=password_hidden />

                        <label class=r#"block mb-1 text-sm font-medium text-text"#>"Base DN"</label>
                        <input
                            class=r#"py-2 px-3 w-full text-sm rounded-md border border-gray-300 
                            focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                            name="name"
                            value=move || ldap_args.get().base_dn
                            bind:value=base_dn
                        />

                        <div class=r#"flex gap-3 mt-2"#>
                            <button
                                type="button"
                                class=r#"py-2 px-4 text-sm rounded-md border border-gray-300 hover:bg-gray-50"#
                                on:click=move |_| {
                                    let url = ldap_url.get();
                                    let bind_dn = bind_dn.get();
                                    let bind_pw = bind_pw.get();
                                    let base_dn = base_dn.get();
                                    let enabled = enabled.get();
                                    spawn_local(async move {
                                        if let Ok(ApiResult { result, details }) = crate::server::admin::test_ldap(LdapArgs {
                                                url,
                                                bind_dn,
                                                bind_pw,
                                                base_dn,
                                                enabled
                                            })
                                            .await
                                        {
                                            if result == ResultStatus::Success {
                                                connect_success.set(true);
                                            } else {
                                                connect_success.set(false);
                                            }
                                            
                                            connect_status_ui.set(details.unwrap_or_default());
                                        }
                                    });
                                }
                            >
                                "Test Connection"
                            </button>
                            
                            <button
                                class=r#"inline-flex items-center py-2 px-4 ml-auto text-sm font-semibold 
                                text-white rounded-md shadow-sm focus:ring-2 focus:outline-none 
                                bg-yale-blue-600 hover:bg-yale-blue-500 focus:ring-yale-blue-500"#
                                on:click=move |_| {
                                    let url = ldap_url.get();
                                    let bind_dn = bind_dn.get();
                                    let bind_pw = bind_pw.get();
                                    let base_dn = base_dn.get();
                                    let enabled = enabled.get();
                                    spawn_local(async move {
                                        if let Ok(ApiResult { result, details }) = crate::server::admin::update_ldap(LdapArgs {
                                                url,
                                                bind_dn,
                                                bind_pw,
                                                base_dn,
                                                enabled
                                            })
                                            .await
                                        {
                                            if result == ResultStatus::Success {
                                                connect_success.set(true);
                                            } else {
                                                connect_success.set(false);
                                            }
                                            
                                            connect_status_ui.set(details.unwrap_or_default());
                                        }
                                    });
                                }
                            >
                                "Apply"
                            </button>
                        </div>
                        <Transition fallback=|| view! { "..." }>{move || connect_status_ui.get()}</Transition>
                    </div>
                }
            }}
        </Suspense>
    }
}
