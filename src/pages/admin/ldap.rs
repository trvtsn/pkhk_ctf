use crate::{components::utils::HidePasswordButton, server::{admin::{get_all_files, upload_files}, db, enums::ResultStatus, structs::ApiResult}};
use leptos::{prelude::*, task::spawn_local, wasm_bindgen::JsCast, web_sys::{FormData, HtmlFormElement, SubmitEvent}};

/// Default Home Page
#[component]
pub fn Ldap() -> impl IntoView {
    let connect_success = RwSignal::new(false);
    let password_hidden = RwSignal::new(true);

    let ldap_url = RwSignal::new("".to_string());
    let bind_dn = RwSignal::new("".to_string());
    let bind_pw = RwSignal::new("".to_string());
    let base_dn = RwSignal::new("".to_string());

    let connect_status_classes = Memo::new(move |_| {
        let base = "rounded-full w-24 h-24";
        if connect_success.get() {
            format!("{} bg-green", base)
        } else {
            format!("{} bg-red", base)
        }
    });

    view! {
        <div>
            "Connection Status"
            <div class=connect_status_classes></div>
        </div>

        <label class=r#"block mb-1 text-sm font-medium text-text"#>"LDAP URL"</label>
        <input
            class=r#"py-2 px-3 w-full text-sm rounded-md border border-gray-300 
            focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
            name="name"
            bind:value=ldap_url
        />

        <label class=r#"block mb-1 text-sm font-medium text-text"#>"Bind DN"</label>
        <input
            class=r#"py-2 px-3 w-full text-sm rounded-md border border-gray-300 
            focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
            name="name"
            bind:value=bind_dn
        />

        <label class=r#"block mb-1 text-sm font-medium text-text"#>"Bind Password"</label>
        <input
            class=r#"py-2 px-3 w-full text-sm rounded-md border border-gray-300 
            focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
            type=move || if password_hidden.get() { "password" } else { "text" }
            name="password"
            value=move || bind_pw.get()
            bind:value=bind_pw
        />
        <HidePasswordButton hidden=password_hidden />

        <label class=r#"block mb-1 text-sm font-medium text-text"#>"Base DN"</label>
        <input
            class=r#"py-2 px-3 w-full text-sm rounded-md border border-gray-300 
            focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
            name="name"
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
                    spawn_local(async move {
                        if let Ok(ApiResult { result, .. }) = crate::server::admin::test_ldap(crate::server::db::structs::LdapArgs {
                                url,
                                bind_dn,
                                bind_pw,
                                base_dn
                            })
                            .await && result == ResultStatus::Success
                        {
                            connect_success.set(true);
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
                    // spawn_local(async move {
                    //     if let Ok(ApiResult { result, .. }) = crate::server::admin::challenge(crate::server::admin::ChallengeAction::Create {
                    //             event_id,
                    //             name,
                    //             description,
                    //             category,
                    //             difficulty,
                    //             points,
                    //             flag,
                    //             visible_to_groups,
                    //             attachments,
                    //             illustration
                    //         })
                    //         .await && result == ResultStatus::Success
                    //     {
                    //         connect_success.set(true);
                    //     }
                    // });
                }
            >
                "Apply"
            </button>
        </div>
    }
}
