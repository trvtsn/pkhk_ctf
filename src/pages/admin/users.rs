use crate::{components::{admin::user::User, utils::HidePasswordButton}, pages::admin::Actions, server::{admin::get_all_users, db::{enums::UserRole, structs::DbUser}, enums::ResultStatus, structs::ApiResult}};
use leptos::{prelude::*, task::spawn_local};

/// Default Home Page
#[component]
pub fn Users() -> impl IntoView {
    let section = RwSignal::new(Actions::None);
    let refresh = RwSignal::new(0);
    let creating = RwSignal::new(false);
    let password_hidden = RwSignal::new(true);

    let password_input_type = Memo::new(move |_| {
        if password_hidden.get() {
            "password"
        } else {
            "text"
        }
    });

    let username_signal = RwSignal::new("".to_string());
    let email_signal = RwSignal::new("".to_string());
    let password_signal = RwSignal::new("".to_string());
    let confirm_password_signal = RwSignal::new("".to_string());
    let roles_signal = RwSignal::new(vec![UserRole::Admin, UserRole::Competitor]);
    let role_signal = RwSignal::new("".to_string());

    let users_resource = Resource::new(move || refresh.get(), move |_| async move {
        get_all_users().await.unwrap_or_default()
    });

    let confirm_password_ui = Memo::new(move |_| {
        if password_signal.get() != confirm_password_signal.get() { "Must match with password" } else { "" }
    });

    view! {
        <div class="flex gap-2 mb-4">
            <button class="border border-gray-300 px-3 py-1 rounded-md text-sm hover:bg-gray-50" on:click=move |_| {
                if creating.get() {
                    creating.set(false);
                    section.set(Actions::None);
                } else {
                    creating.set(true);
                    section.set(Actions::Create);
                }
            }>"Create"</button>
        </div>

        <div class="flex flex-col gap-4">
            <Show when=move || section.get() == Actions::Create>
                <label class="block text-sm font-medium text-gray-700 mb-1">"Name"</label>
                <input 
                    class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" 
                    name="username" 
                    value=move || username_signal.get() 
                    bind:value=username_signal
                />
                
                <label class="block text-sm font-medium text-gray-700 mb-1">"E-mail"</label>
                <input 
                    class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" 
                    name="email" 
                    value=move || email_signal.get() 
                    bind:value=email_signal
                />
                
                <label class="block text-sm font-medium text-gray-700 mb-1">"Password"</label>
                <input 
                    class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" 
                    type=move || password_input_type.get()
                    name="password" 
                    value=move || password_signal.get() 
                    bind:value=password_signal
                /><HidePasswordButton hidden=password_hidden/>
                
                <label class="block text-sm font-medium text-gray-700 mb-1">"Confirm Password"</label>
                <input 
                    class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" 
                    type=move || password_input_type.get()
                    name="confirm_password" 
                    value=move || confirm_password_signal.get() 
                    bind:value=confirm_password_signal
                /><HidePasswordButton hidden=password_hidden/>

                <label class="block text-sm font-medium text-gray-700 mb-1">"Role"</label>
                <select 
                    class="w-full rounded-md border border-gray-300 bg-white px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" 
                    name="event_id" 
                    bind:value=role_signal
                >
                    <option value="">"-- Select Role --"</option>
                    <For
                        each=move || roles_signal.get()
                        key=|r: &UserRole| r.to_string()
                        let(role: UserRole)
                    >
                        <option value={role.to_string()}>{role.to_string()}</option>
                    </For>
                </select>
                <Transition fallback=|| view! { "..." }>
                    {confirm_password_ui.get()}
                </Transition>

                <div class="flex gap-3 mt-2">
                    <button 
                        type="button" 
                        class="px-4 py-2 rounded-md border border-gray-300 text-sm hover:bg-gray-50" 
                        on:click=move |_| {section.set(Actions::None)}
                    >"Cancel"</button>
                    <button
                        type="button"
                        class=r#"ml-auto inline-flex items-center px-4 py-2 rounded-md bg-indigo-600 text-white text-sm 
                        font-semibold shadow-sm hover:bg-indigo-500 focus:outline-none focus:ring-2 focus:ring-indigo-500"#
                        on:click=move |_| {
                            let username = username_signal.get().clone();
                            let email = email_signal.get().clone();
                            let password = password_signal.get().clone();
                            let confirm_password = confirm_password_signal.get().clone();
                            let role = role_signal.get().clone().into();
                            spawn_local(async move {
                                tracing::debug!("creating user...");
                                if let Ok(ApiResult { result, .. }) = crate::server::admin::user(
                                    crate::server::admin::UserAction::Create { username, email, password, confirm_password, role }
                                ).await && result == ResultStatus::Success {
                                    refresh.update(|n| *n += 1);
                                }
                            });
                        }
                    >"Create"</button>
                </div>
            </Show>
        </div>

        <Transition fallback=move || view! { <div>"Loading..."</div> }>
            <div class="grid-cols-4 p-4 m-4 flex gap-4">
                <For
                    each=move || users_resource.get().unwrap_or_default()
                    key=|user: &DbUser| user.id.clone()
                    let(user)
                >
                    <User user refresh/>
                </For>
            </div>
        </Transition>
    }
}
