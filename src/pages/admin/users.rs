use crate::{components::{admin::user::User, utils::HidePasswordButton}, pages::admin::Actions, server::{admin::get_all_users, db::{enums::UserRole, structs::DbUser}, enums::ResultStatus, structs::ApiResult}};
use leptos::{prelude::*, task::spawn_local};

/// Default Home Page
#[component]
pub fn Users() -> impl IntoView {
    let section = RwSignal::new(Actions::None);
    let refresh = RwSignal::new(0);
    let creating = RwSignal::new(false);
    let password_hidden = RwSignal::new(true);
    let confirm_password_hidden = RwSignal::new(true);
    
    let username_signal = RwSignal::new("".to_string());
    let email_signal = RwSignal::new("".to_string());
    let password_signal = RwSignal::new("".to_string());
    let confirm_password_signal = RwSignal::new("".to_string());
    let roles_signal = RwSignal::new(vec![UserRole::Admin, UserRole::Competitor]);
    let role_signal = RwSignal::new("".to_string());

    let users_resource = Resource::new(move || refresh.get(), move |_| async move {
        get_all_users().await.unwrap_or_default()
    });

    view! {
        <div class=r#"flex gap-2 mb-4"#>
            <button
                class=r#"py-1 px-3 text-sm rounded-md border border-gray-300 hover:bg-gray-50"#
                on:click=move |_| {
                    if creating.get() {
                        creating.set(false);
                        section.set(Actions::None);
                    } else {
                        creating.set(true);
                        section.set(Actions::Create);
                    }
                }
            >
                "Create"
            </button>
        </div>

        <div class=r#"flex flex-col gap-4"#>
            <Show when=move || section.get() == Actions::Create>
                <label class=r#"block mb-1 text-sm font-medium text-gray-700"#>"Name"</label>
                <input
                    class=r#"py-2 px-3 w-full text-sm rounded-md border border-gray-300 
                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                    name="username"
                    value=move || username_signal.get()
                    bind:value=username_signal
                />

                <label class=r#"block mb-1 text-sm font-medium text-gray-700"#>"E-mail"</label>
                <input
                    class=r#"py-2 px-3 w-full text-sm rounded-md border border-gray-300 
                    focus:ring-2 focus:outline-none focus:ring--500"#
                    name="email"
                    value=move || email_signal.get()
                    bind:value=email_signal
                />

                <label class=r#"block mb-1 text-sm font-medium text-gray-700"#>"Password"</label>
                <input
                    class=r#"py-2 px-3 w-full text-sm rounded-md border border-gray-300 
                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                    type=move || if password_hidden.get() { "password" } else { "text" }
                    name="password"
                    value=move || password_signal.get()
                    bind:value=password_signal
                />
                <HidePasswordButton hidden=password_hidden />

                <label class=r#"block mb-1 text-sm font-medium text-gray-700"#>
                    "Confirm Password"
                </label>
                <input
                    class=r#"py-2 px-3 w-full text-sm rounded-md border border-gray-300 
                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                    type=move || if confirm_password_hidden.get() { "password" } else { "text" }
                    name="confirm_password"
                    value=move || confirm_password_signal.get()
                    bind:value=confirm_password_signal
                />
                <HidePasswordButton hidden=confirm_password_hidden />

                <label class=r#"block mb-1 text-sm font-medium text-gray-700"#>"Role"</label>
                <select
                    class=r#"py-2 px-3 w-full text-sm bg-white rounded-md border border-gray-300 
                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                    name="event_id"
                    bind:value=role_signal
                >
                    <option value="">"-- Select Role --"</option>
                    <For
                        each=move || roles_signal.get()
                        key=|r: &UserRole| r.to_string()
                        let(role: UserRole)
                    >
                        <option value=role.to_string()>{role.to_string()}</option>
                    </For>
                </select>
                <Transition fallback=|| {
                    view! { "..." }
                }>
                    {move || {
                        if password_signal.get() != confirm_password_signal.get() {
                            "Confirmation must match"
                        } else {
                            ""
                        }
                    }}
                </Transition>

                <div class=r#"flex gap-3 mt-2"#>
                    <button
                        type="button"
                        class=r#"py-2 px-4 text-sm rounded-md border border-gray-300 hover:bg-gray-50"#
                        on:click=move |_| { section.set(Actions::None) }
                    >
                        "Cancel"
                    </button>
                    <button
                        type="button"
                        class=r#"inline-flex items-center py-2 px-4 ml-auto text-sm font-semibold 
                        text-white rounded-md shadow-sm focus:ring-2 focus:outline-none 
                        bg-yale-blue-600 hover:bg-yale-blue-500 focus:ring-yale-blue-500"#
                        on:click=move |_| {
                            let username = username_signal.get().clone();
                            let email = email_signal.get().clone();
                            let password = password_signal.get().clone();
                            let confirm_password = confirm_password_signal.get().clone();
                            let role = role_signal.get().clone().into();
                            spawn_local(async move {
                                tracing::debug!("creating user...");
                                if let Ok(ApiResult { result, .. }) = crate::server::admin::user(crate::server::admin::UserAction::Create {
                                        username,
                                        email,
                                        password,
                                        confirm_password,
                                        role,
                                    })
                                    .await && result == ResultStatus::Success
                                {
                                    refresh.update(|n| *n += 1);
                                }
                            });
                        }
                    >
                        "Create"
                    </button>
                </div>
            </Show>
        </div>

        <Transition fallback=move || view! { <div>"Loading..."</div> }>
            <div class=r#"flex grid-cols-4 gap-4 p-4 m-4"#>
                <For
                    each=move || users_resource.get().unwrap_or_default()
                    key=|user: &DbUser| user.id.clone()
                    let(user)
                >
                    <User user refresh />
                </For>
            </div>
        </Transition>
    }
}
