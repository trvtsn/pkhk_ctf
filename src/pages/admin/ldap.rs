use crate::{components::utils::{ComponentSize, HidePasswordButton, Spinner}, server::{admin::{disable_ldap, enable_ldap, get_certificate_without_blob, get_ldap, test_ldap, update_ldap, upload_certificate}, db::structs::{LdapArgs, SqlBool}, enums::ResultStatus, structs::ApiResult}};
use icondata as i;
use leptos::{prelude::*, task::spawn_local, wasm_bindgen::JsCast, web_sys::{Event, FormData, HtmlInputElement}};
use leptos_icons::Icon;

/// Default Home Page
#[component]
pub fn Ldap() -> impl IntoView {
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
        let ldap_args = get_ldap().await.unwrap_or_default().unwrap_or_default();
        let test_args = ldap_args.clone();
        if ldap_args.enabled.0 {
            let url = test_args.url;
            let bind_dn = test_args.bind_dn;
            let bind_pw = test_args.bind_pw;
            let base_dn = test_args.base_dn;
            let enabled = test_args.enabled;
            spawn_local(async move {
                if let Ok(ApiResult { result, .. }) = test_ldap(LdapArgs {
                        url,
                        bind_dn,
                        bind_pw,
                        base_dn,
                        enabled,
                    })
                    .await
                {
                    if result == ResultStatus::Success {
                        connect_success.set(true);
                    } else {
                        connect_success.set(false);
                    }
                }
            });
        }

        ldap_args
    });

    let cert_upload_action = Action::new_local(move |data: &FormData| {
        let data = data.clone();
        async move {
            if let Ok(api_result) = upload_certificate(data.into()).await {
                certificate.set(Some(api_result.details.clone()));
            }
        }
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
        <Transition fallback=move || {
            view! { <Spinner component_size=ComponentSize::Big /> }
        }>
            {move || {
                if let Some(ldap_args) = ldap_resource.get() {
                    ldap_url.set(ldap_args.url.clone());
                    bind_dn.set(ldap_args.bind_dn.clone());
                    bind_pw.set(ldap_args.bind_pw.clone());
                    base_dn.set(ldap_args.base_dn.clone());
                    enabled.set(ldap_args.enabled);
                }

                if let Some(Some(cert)) = certificate_resource.get() {
                    certificate.set(Some(cert));
                }

                view! {
                    <div class="grid gap-2">
                        <div class="flex gap-2">
                            <label>"Enable"</label>
                            <input
                                type="checkbox"
                                checked=move || enabled.get().0 
                                on:input=move |ev| {
                                    let is_checked = event_target_checked(&ev);
                                    if is_checked {
                                        enabled.set(SqlBool(true));
                                        spawn_local(async move {
                                            _ = enable_ldap().await; // call only on "Apply", not on every check/uncheck
                                        });
                                    } else {
                                        enabled.set(SqlBool(false));
                                        spawn_local(async move {
                                            _ = disable_ldap().await; // call only on "Apply", not on every check/uncheck
                                        });
                                    };
                                }
                            />
                        </div>

                        <div 
                            hidden=move || !enabled.get().0
                        >
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
                                        value=move || ldap_url.get()
                                        bind:value=ldap_url
                                    />
                                </div>

                                <div class="grid">
                                    <label class=r#"block mb-1 text-sm font-medium text-text"#>"Bind DN"</label>
                                    <input
                                        class=r#"py-2 px-3 w-full text-sm rounded-md border border-input-border 
                                        focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                                        name="name"
                                        value=move || bind_dn.get()
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
                                            value=move || bind_pw.get()
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
                                        value=move || base_dn.get()
                                        bind:value=base_dn
                                    />
                                </div>

                                <div class="grid">
                                    <label class=r#"block mb-1 text-sm font-medium"#>"Certificate (Optional)"</label>
                                    <div class="grid gap-2">
                                        {move || {
                                            if let Some(cert) = certificate.get() {
                                                view! {
                                                    <div class="flex gap-2 items-center">
                                                        {cert.file_name.clone()}
                                                        <a
                                                            download
                                                            href=move || format!("/file/{}", cert.id.clone())
                                                        >
                                                            <Icon icon=i::LuDownload />
                                                        </a>
                                                        <button 
                                                            class="cursor-pointer"
                                                            on:click=move |_| {
                                                                certificate.set(None);
                                                            } 
                                                        >
                                                            <Icon icon=i::LuX />
                                                        </button>
                                                    </div>
                                                }.into_any()
                                            } else {
                                                "".into_any()
                                            }
                                        }}
                                        <input
                                            class=r#"bg-background w-full text-sm p-3 rounded-lg shadow-sm"#
                                            type="file"
                                            name="certificate"
                                            on:change=move |ev: Event| {
                                                let input = ev.target().unwrap().unchecked_into::<HtmlInputElement>();
                                                if let Some(files) = input.files() && files.length() > 0 {
                                                    let file = files.get(0).unwrap();
                                                    let fd = FormData::new().unwrap();
                                                    fd.append_with_blob_and_filename("file", &file, &file.name()).unwrap();
                                                    cert_upload_action.dispatch_local(fd);
                                                }
                                            }
                                        /><p>{move || {
                                            if cert_upload_action.pending().get() {
                                                "Uploading..."
                                            } else {
                                                ""
                                            }
                                        }}</p>
                                    </div>
                                </div>

                                <div class=r#"flex gap-3 mt-2 pt-2"#>
                                    <button
                                        type="button"
                                        class=r#"py-2 px-4 text-sm rounded-md border border-input-border hover:bg-background-hover"#
                                        on:click=move |_| {
                                            let url = ldap_url.get();
                                            let bind_dn = bind_dn.get();
                                            let bind_pw = bind_pw.get();
                                            let base_dn = base_dn.get();
                                            let enabled = enabled.get();
                                            spawn_local(async move {
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
                                            let url = ldap_url.get();
                                            let bind_dn = bind_dn.get();
                                            let bind_pw = bind_pw.get();
                                            let base_dn = base_dn.get();
                                            let certificate = certificate.get();
                                            let enabled = enabled.get();
                                            spawn_local(async move {
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
                                <Transition fallback=|| view! { "..." }>{move || connect_status_ui.get()}</Transition>
                            </div>
                        </div>
                    </div>
                }
            }}
        </Transition>
    }
}
