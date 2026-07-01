use crate::{components::utils::{ComponentSize, FileTooltip, HidePasswordButton, Spinner}, server::{admin::{api::{disable_ldap, enable_ldap, get_certificate_without_blob, get_ldap, test_ldap, update_ldap, upload_certificate}}, db::structs::{LdapArgs, SqlBool}, enums::ResultStatus, structs::ApiResult}, utils::build_single_file_form_data};
use leptos::{prelude::*, task::spawn_local};
use zeroize::Zeroizing;

/// Admin LDAP configuration.
/// Connection settings, certificate upload, and connection testing.
#[component]
pub fn Ldap() -> impl IntoView {
    let certificate_ref = NodeRef::new();
    let refresh = RwSignal::new(0);
    let connect_status_ui = RwSignal::new("".to_string());
    let connect_success = RwSignal::new(false);
    let password_hidden = RwSignal::new(true);

    let ldap_url = RwSignal::new("".to_string());
    let bind_dn = RwSignal::new("".to_string());
    let bind_pw = RwSignal::new("".to_string());
    let base_dn = RwSignal::new("".to_string());
    let certificate = RwSignal::new(None);
    let enabled = RwSignal::new(SqlBool(true));

    let certificate_resource = Resource::new(move || refresh.get(), move |_| async move {
        get_certificate_without_blob().await.unwrap_or_default()
    });
    let ldap_resource = Resource::new(move || refresh.get(), move |_| async move {
        get_ldap().await.unwrap_or_default().unwrap_or_default()
    });

    let connect_status_classes = Memo::new(move |_| {
        let base = "rounded-full w-3 h-3";
        if connect_success.get() {
            format!("{base} bg-green-600")
        } else {
            format!("{base} bg-red-600")
        }
    });

    Effect::watch(
        move || ldap_resource.get(),
        move |val, _, _| {
            if let Some(ldap_args) = val.clone() {
                ldap_url.set(ldap_args.url.clone());
                bind_dn.set(ldap_args.bind_dn.clone());
                bind_pw.set(ldap_args.bind_pw.to_string());
                base_dn.set(ldap_args.base_dn.clone());
                enabled.set(ldap_args.enabled);
                if ldap_args.enabled.0 {
                    spawn_local(async move {
                        if let Ok(ApiResult { result, .. }) = test_ldap(LdapArgs {
                                url: ldap_args.url,
                                bind_dn: ldap_args.bind_dn,
                                bind_pw: ldap_args.bind_pw,
                                base_dn: ldap_args.base_dn,
                                enabled: ldap_args.enabled,
                            })
                            .await
                        {
                            connect_success.set(result == ResultStatus::Success);
                        }
                    });
                }
            }
        },
        false
    );

    Effect::watch(
        move || certificate_resource.get(),
        move |val, _, _| {
            if let Some(Some(cert)) = val {
                certificate.set(Some(cert.clone()));
            }
        },
        false,
    );

    view! {
        <Transition fallback=move || {
            view! { <Spinner component_size=ComponentSize::Big /> }
        }>
            {move || {
                ldap_resource.get();
                certificate_resource.get();
                view! {
                    <div class="grid gap-2">
                        <div class="flex gap-2">
                            <label>"Enable"</label>
                            <input
                                type="checkbox"
                                checked=move || enabled.get().0
                                on:input=move |ev| {
                                    let is_checked = event_target_checked(&ev);
                                    enabled.set(SqlBool(is_checked));
                                }
                            />
                        </div>

                        <div class="flex gap-2 items-center">
                            "Connection Status"
                            <svg class=connect_status_classes>
                                <circle /> // on hover tooltip -> "Connected"/"No Connection"?
                            </svg>
                        </div>

                        <div class="grid gap-3 pt-4">
                            <div class="grid">
                                <label class=r#"block mb-1 text-sm font-medium text-text"#>"LDAP URL"</label>
                                <input
                                    class=r#"py-2 px-3 w-full text-sm rounded-md border border-input-border 
                                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                                    name="name"
                                    placeholder="e.g. ldaps://192.168.1.11:636"
                                    bind:value=ldap_url
                                />
                            </div>

                            <div class="grid">
                                <label class=r#"block mb-1 text-sm font-medium text-text"#>"Bind DN"</label>
                                <input
                                    class=r#"py-2 px-3 w-full text-sm rounded-md border border-input-border 
                                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                                    name="name"
                                    placeholder="e.g. CN=binduser,CN=Users,DC=my,DC=ldapsite,DC=com"
                                    bind:value=bind_dn
                                />
                            </div>

                            <div class="grid">
                                <label class=r#"block mb-1 text-sm font-medium text-text"#>"Bind Password"</label>
                                <div class="flex gap-2">
                                    <input
                                        class=r#"py-2 px-3 w-full text-sm rounded-md border border-input-border 
                                        focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                                        type=move || if password_hidden.get() { "password" } else { "text" }
                                        name="password"
                                        bind:value=bind_pw
                                    />
                                    <HidePasswordButton hidden=password_hidden />
                                </div>
                            </div>

                            <div class="grid">
                                <label class=r#"block mb-1 text-sm font-medium text-text"#>"Base DN"</label>
                                <input
                                    class=r#"py-2 px-3 w-full text-sm rounded-md border border-input-border 
                                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                                    name="name"
                                    placeholder="e.g. CN=Users,DC=my,DC=ldapsite,DC=com"
                                    bind:value=base_dn
                                />
                            </div>

                            <div class="grid">
                                <label class=r#"block mb-1 text-sm font-medium"#>"Certificate (Optional)"</label>
                                <div class="grid gap-2">
                                    {move || {
                                        if let Some(cert) = certificate.get() {
                                            view! {
                                                <FileTooltip
                                                    file_name=cert.file_name.clone()
                                                    id=cert.id.clone()
                                                    on_download=format!("/file/{}", cert.id)
                                                    on_remove=Callback::new(move |_| certificate.set(None))
                                                />
                                            }.into_any()
                                        } else {
                                            "".into_any()
                                        }
                                    }}
                                    <input
                                        class=r#"bg-background w-full text-sm p-3 rounded-lg shadow-sm"#
                                        type="file"
                                        name="certificate"
                                        node_ref=certificate_ref
                                    />
                                </div>
                            </div>

                            <div class=r#"flex gap-3 mt-2 pt-2"#>
                                <button
                                    type="button"
                                    class=r#"py-2 px-4 text-sm rounded-md border border-input-border hover:bg-background-hover"#
                                    on:click=move |_| {
                                        let certificate_ref = certificate_ref.get_untracked();

                                        let url = ldap_url.get_untracked();
                                        let bind_dn = bind_dn.get_untracked();
                                        let bind_pw = Zeroizing::new(bind_pw.get_untracked());
                                        let base_dn = base_dn.get_untracked();
                                        let enabled = enabled.get_untracked();
                                        spawn_local(async move {
                                            if let Some(fd) = build_single_file_form_data(certificate_ref) {
                                                if let Ok(api_result) = upload_certificate(fd.into()).await {
                                                    certificate.set(Some(api_result.details));
                                                }
                                            }

                                            if let Ok(ApiResult { result, details }) = test_ldap(LdapArgs {
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
                                                
                                                connect_status_ui.set(details);
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
                                        let certificate_ref = certificate_ref.get_untracked();

                                        let url = ldap_url.get_untracked();
                                        let bind_dn = bind_dn.get_untracked();
                                        let bind_pw = Zeroizing::new(bind_pw.get_untracked());
                                        let base_dn = base_dn.get_untracked();
                                        let enabled = enabled.get_untracked();
                                        spawn_local(async move {
                                            if let Some(fd) = build_single_file_form_data(certificate_ref) {
                                                if let Ok(api_result) = upload_certificate(fd.into()).await {
                                                    certificate.set(Some(api_result.details));
                                                }
                                            }

                                            let certificate = certificate.get_untracked();

                                            if enabled.0 {
                                                _ = enable_ldap().await;
                                            } else {
                                                _ = disable_ldap().await;
                                            }

                                            if let Ok(ApiResult { result, details }) = update_ldap(LdapArgs {
                                                    url,
                                                    bind_dn,
                                                    bind_pw,
                                                    base_dn,
                                                    enabled
                                                }, certificate)
                                                .await
                                            {
                                                if result == ResultStatus::Success {
                                                    refresh.update(|n| *n += 1);
                                                }
                                                
                                                connect_status_ui.set(details);
                                            }
                                        });
                                    }
                                >
                                    "Apply"
                                </button>
                            </div>
                            {move || connect_status_ui.get()}
                        </div>
                    </div>
                }
            }}
        </Transition>
    }
}
