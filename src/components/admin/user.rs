use crate::{components::utils::HidePasswordButton, server::{db::{enums::UserRole, structs::DbUser}, enums::ResultStatus, structs::ApiResult}};
use leptos::{prelude::*, task::spawn_local, web_sys::Event};

#[component]
pub fn User(
    user: DbUser,
    refresh: RwSignal<i32>
) -> impl IntoView {
    let id_signal = RwSignal::new(user.id.clone());
    let username_signal = RwSignal::new(user.username.clone());
    let email_signal = RwSignal::new(user.email.clone());
    let created_signal = RwSignal::new(user.created_at);
    let last_active_signal = RwSignal::new(user.last_active_at);
    let new_password_signal = RwSignal::new("".to_string());
    let confirm_new_password_signal = RwSignal::new("".to_string());
    let points_signal = RwSignal::new(user.points);
    let roles_signal = RwSignal::new(vec![UserRole::Admin, UserRole::Competitor]);
    let role_signal = RwSignal::new(user.role.to_string());

    let username_edit = RwSignal::new(user.username.clone());
    let email_edit = RwSignal::new(user.email.clone());
    let new_password_edit = RwSignal::new("".to_string());
    let confirm_new_password_edit = RwSignal::new("".to_string());
    let points_edit = RwSignal::new(user.points);
    let role_edit = RwSignal::new(user.role.to_string());

    let editing = RwSignal::new(false);
    let editing_password = RwSignal::new(false);
    let deleting = RwSignal::new(false);
    let new_password_hidden = RwSignal::new(true);
    let confirm_new_password_hidden = RwSignal::new(true);

    let delete_submit_btn_text = Memo::new(move |_| {
        if deleting.get() { "Confirm Delete".to_string() } else { "Delete".to_string() }
    });
    let edit_submit_btn_text = Memo::new(move |_| {
        if editing.get() { "Confirm Edit".to_string() } else { "Edit".to_string() }
    });
    let edit_password_submit_btn_text = Memo::new(move |_| {
        if editing.get() { "Confirm Edit".to_string() } else { "Edit Password".to_string() }
    });
    
    view! {
        <div class="bg-yale-blue-50 hover:bg-yale-blue-100 rounded-lg p-4 content-center">
            <h3 class="text-3xl/8 font-bold">{move || username_signal.get().clone()}</h3>
            <p class="text-lg/8"><b>"ID: "</b> {move || id_signal.get().clone()}</p>
            <p class="text-lg/8"><b>"E-mail: "</b> {move || email_signal.get().clone()}</p>
            <p class="text-lg/8"><b>"Role: "</b> {move || role_signal.get().to_string()}</p>
            <p class="text-lg/8"><b>"Points: "</b> {move || points_signal.get()}</p>
            <p class="text-lg/8"><b>"Created: "</b> {move || created_signal.get().to_string()}</p>
            <p class="text-lg/8"><b>"Last active: "</b> {move || last_active_signal.get().to_string()}</p>

            <Show when=move || editing.get()>
                <label class="block text-sm font-medium text-gray-700 mb-1">"Name"</label>
                <input 
                    class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" 
                    name="username" 
                    value=move || username_signal.get() 
                    bind:value=username_edit
                />

                <label class="block text-sm font-medium text-gray-700 mb-1">"E-mail"</label>
                <input 
                    class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" 
                    name="email" 
                    value=move || email_signal.get() 
                    bind:value=email_edit
                />

                <label class="block text-sm font-medium text-gray-700 mb-1">"Points"</label>
                <input 
                    class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" 
                    name="points" 
                    type="number"
                    value=move || points_signal.get() 
                    on:change=move |ev: Event| {
                        let value = event_target_value(&ev);
                        points_edit.set(value.parse::<u32>().unwrap_or_default());
                    }
                />

                <label class="block text-sm font-medium text-gray-700 mb-1">"Role"</label>
                <select 
                    class="w-full rounded-md border border-gray-300 bg-white px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" 
                    name="event_id" 
                    bind:value=role_edit
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
            </Show>

            <Show when=move || editing_password.get()>
                <label class="block text-sm font-medium text-gray-700 mb-1">"New Password"</label>
                <input 
                    class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" 
                    type=move || if new_password_hidden.get() { "password" } else { "text" }
                    name="new_password" 
                    bind:value=new_password_edit
                /><HidePasswordButton hidden=new_password_hidden/>
                
                <label class="block text-sm font-medium text-gray-700 mb-1">"Confirm New Password"</label>
                <input 
                    class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" 
                    type=move || if confirm_new_password_hidden.get() { "password" } else { "text" }
                    name="confirm_new_password" 
                    bind:value=confirm_new_password_edit
                /><HidePasswordButton hidden=confirm_new_password_hidden/>
                <Transition fallback=|| {view! { "..." }}>
                    {move || if new_password_signal.get() != confirm_new_password_signal.get() { "Confirmation must match" } else { "" }}
                </Transition>
            </Show>

            // dont show edit and delete buttons for admin users
            <Show when=move || role_signal.get() != UserRole::Admin.to_string()>
                <div class="flex flex-row-reverse gap-3 mt-2">
                    <Show when=move || editing.get() || editing_password.get() || deleting.get()>
                        <button 
                            class="px-4 py-2 rounded-md border border-gray-300 text-sm hover:bg-gray-50" 
                            on:click=move |_| {editing.set(false); deleting.set(false); editing_password.set(false);}
                        >"Cancel"</button>
                    </Show>

                    <button 
                        type="button" 
                        hidden=move || deleting.get() || editing_password.get()
                        class=r#"inline-flex items-center gap-2 rounded-lg text-white px-4 py-2 text-sm font-medium focus:outline-none 
                        focus:ring-2 active:scale-95 transition bg-indigo-600 hover:bg-indigo-700 focus:ring-indigo-400"#
                        on:click=move |_| {
                            let user_id = id_signal.get(); 
                            let username = username_edit.get();
                            let email = email_edit.get();
                            let password = new_password_edit.get();
                            let confirm_password = confirm_new_password_edit.get();
                            let points = points_edit.get();
                            let role = role_edit.get();
                            if editing.get() {
                                spawn_local(async move {
                                    tracing::debug!("editing user: {}", user_id.clone());
                                    if let Ok(ApiResult { result, .. }) = crate::server::admin::user(
                                        crate::server::admin::UserAction::Edit { 
                                            id: user_id, 
                                            username: username.clone(), 
                                            email: email.clone(),
                                            password: password.clone(),
                                            confirm_password: confirm_password.clone(),
                                            points,
                                            role: role.clone().into()
                                        }
                                    ).await && result == ResultStatus::Success {
                                        refresh.update(|n| *n += 1);

                                        username_signal.set(username);
                                        email_signal.set(email);
                                        new_password_signal.set(password);
                                        confirm_new_password_signal.set(confirm_password);
                                        points_signal.set(points);
                                        role_signal.set(role);
                                    }
                                });
                                editing.set(false)
                            } else {
                                editing.set(true)
                            }
                    }>{ move || edit_submit_btn_text.get() }</button>

                    <button 
                        type="button" 
                        hidden=move || deleting.get() || editing.get()
                        class="inline-flex items-center gap-2 rounded-lg text-white px-4 py-2 text-sm font-medium focus:outline-none focus:ring-2 active:scale-95 transition bg-indigo-600 hover:bg-indigo-700 focus:ring-indigo-400"
                        on:click=move |_| {
                            if editing_password.get() {
                                spawn_local(async move {
                                    tracing::debug!("editing user password: {}", id_signal.get().clone());
                                    if let Ok(ApiResult { result, .. }) = crate::server::admin::user(
                                        crate::server::admin::UserAction::EditPassword { 
                                            id: id_signal.get().clone(), 
                                            password: new_password_edit.get().clone(),
                                            confirm_password: confirm_new_password_edit.get().clone()
                                        }
                                    ).await && result == ResultStatus::Success {
                                        refresh.update(|n| *n += 1);
                                    }
                                });
                                editing_password.set(false)
                            } else {
                                editing_password.set(true)
                            }
                    }>{ move || edit_password_submit_btn_text.get() }</button>

                    <button
                        hidden=move || editing.get() || editing_password.get()
                        class="ml-auto inline-flex items-center px-4 py-2 rounded-md bg-red-600 text-white text-sm font-semibold shadow-sm hover:bg-red-500 focus:outline-none focus:ring-2 focus:ring-indigo-500"
                        on:click=move |_| {
                            let id = id_signal.get().clone();
                            if deleting.get() {
                                spawn_local(async move {
                                    tracing::debug!("deleting user: {id}");
                                    if let Ok(ApiResult { result, .. }) = crate::server::admin::user(
                                        crate::server::admin::UserAction::Delete { id } 
                                    ).await && result == ResultStatus::Success {
                                        refresh.update(|n| *n += 1);
                                    }
                                });
                                deleting.set(false);
                            } else {
                                deleting.set(true);
                            }
                        }
                    >
                        { move || delete_submit_btn_text.get() }
                    </button>
                </div>
            </Show>
        </div>
    }
}
