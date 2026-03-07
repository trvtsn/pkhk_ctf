use crate::{components::{toast::{ToastMessageType, push_new_toast}, utils::{HidePasswordButton, FileTooltip}}, server::{admin::upload_avatar, db::{enums::UserRole, structs::{DbUser, UserAvatar}}, enums::ResultStatus, structs::ApiResult}};
use crate::utils::{action_btn_text, build_single_file_form_data, collect_selected_options, csv_contains};
use leptos::{prelude::*, task::spawn_local, wasm_bindgen::JsCast, web_sys::{Event, HtmlSelectElement}};

#[component]
pub fn User(
    user: DbUser,
    user_avatars: RwSignal<Vec<UserAvatar>>,
    groups: RwSignal<Vec<String>>,
    refresh: RwSignal<i32>
) -> impl IntoView {
    let avatar_ref = NodeRef::new();
    let group_add_new_selected = RwSignal::new(false);
    
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
    let groups_signal = RwSignal::new(user.groups.clone());

    let username_edit = RwSignal::new(user.username);
    let email_edit = RwSignal::new(user.email);
    let new_password_edit = RwSignal::new("".to_string());
    let confirm_new_password_edit = RwSignal::new("".to_string());
    let points_edit = RwSignal::new(user.points);
    let role_edit = RwSignal::new(user.role.to_string());
    let avatar_edit = RwSignal::new(None);
    let groups_edit = RwSignal::new(user.groups);

    let editing = RwSignal::new(false);
    let editing_password = RwSignal::new(false);
    let deleting = RwSignal::new(false);
    let new_password_hidden = RwSignal::new(true);
    let confirm_new_password_hidden = RwSignal::new(true);

    let user_avatar = Memo::new(move |_| {
        let user_id = id_signal.get();
        let user_avatars = user_avatars.get();
        user_avatars.into_iter().find(|u| u.clone().user_id.unwrap_or_default() == user_id)
    });
    let delete_submit_btn_text = action_btn_text(move || deleting.get(), "Confirm Delete", "Delete");
    let edit_submit_btn_text = action_btn_text(move || editing.get(), "Confirm Edit", "Edit");
    let edit_password_submit_btn_text = action_btn_text(move || editing.get(), "Confirm Edit", "Edit Password");

    let any_changes_made = Memo::new(move |_| {
        if username_signal.get() == username_edit.get() &&
            email_signal.get() == email_edit.get() &&
            new_password_signal.get() == new_password_edit.get() &&
            confirm_new_password_signal.get() == confirm_new_password_edit.get() &&
            points_signal.get() == points_edit.get() &&
            role_signal.get() == role_edit.get() &&
            groups_signal.get() == groups_edit.get() &&
            avatar_edit.get() == user_avatar.get()
        { false } else { true }
    });
    
    view! {
        <div class=r#"content-center p-4 rounded-lg bg-card hover:bg-card-hover text-text break-all"#>
            {move || {
                if let Some(user_avatar) = user_avatar.get() {
                    avatar_edit.set(Some(user_avatar.clone()));
                    view! {
                        <div class="h-48 w-48 flex justify-center m-auto">
                            <img 
                                src=move || format!("/avatar/{}", user_avatar.attachment_id) 
                                class=r#"rounded-[50%] shadow-sm"#
                            />
                        </div>
                    }.into_any()
                } else {
                    "".into_any()
                }
            }}
            <Show when=move || !editing.get()>
                <h3 class=r#"font-bold text-3xl/8"#>{move || username_signal.get()}</h3>
                <p class=r#"text-lg/8"#>
                    <b>"ID: "</b>
                    {move || id_signal.get()}
                </p>
                <p class=r#"text-lg/8"#>
                    <b>"E-mail: "</b>
                    {move || email_signal.get()}
                </p>
                <p class=r#"text-lg/8"#>
                    <b>"Role: "</b>
                    {move || role_signal.get().to_string()}
                </p>
                <p class=r#"text-lg/8"#>
                    <b>"Points: "</b>
                    {move || points_signal.get()}
                </p>
                <p class=r#"text-lg/8"#>
                    <b>"Created: "</b>
                    {move || created_signal.get().format("%Y-%m-%d %H:%M:%S").to_string()}
                </p>
                <p class=r#"text-lg/8"#>
                    <b>"Last active: "</b>
                    {move || last_active_signal.get().format("%Y-%m-%d %H:%M:%S").to_string()}
                </p>
                <p class=r#"text-lg/8"#>
                    <b>"Groups: "</b>
                    {move || groups_signal.get().replace(",", ", ")}
                </p>
            </Show>

            <Show when=move || editing.get()>
                <div class="grid gap-3">
                    <div class="grid">
                        <label class=r#"block mb-1 text-sm font-medium"#>"Name"</label>
                        <input
                            class=r#"bg-background py-2 px-3 w-full text-sm rounded-md border border-input-border 
                            focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                            name="username"
                            value=move || username_signal.get()
                            bind:value=username_edit
                        />
                    </div>

                    <div class="grid">
                        <label class=r#"block mb-1 text-sm font-medium"#>"E-mail"</label>
                        <input
                            class=r#"bg-background py-2 px-3 w-full text-sm rounded-md border border-input-border 
                            focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                            name="email"
                            value=move || email_signal.get()
                            bind:value=email_edit
                        />
                    </div>

                    <div class="grid">
                        <label class=r#"block mb-1 text-sm font-medium"#>"Points"</label>
                        <input
                            class=r#"bg-background py-2 px-3 w-full text-sm rounded-md border border-input-border 
                            focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                            name="points"
                            type="number"
                            value=move || points_signal.get()
                            on:change=move |ev: Event| {
                                let value = event_target_value(&ev);
                                points_edit.set(value.parse::<u32>().unwrap_or_default());
                            }
                        />
                    </div>

                    <div class="grid">
                        <label class=r#"block mb-1 text-sm font-medium"#>"Role"</label>
                        <select
                            class=r#"bg-background py-2 px-3 w-full text-sm rounded-md border border-input-border 
                            focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                            name="event_id"
                            bind:value=role_edit
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
                    </div>

                    <div class="grid">
                        <label class=r#"block mb-1 text-sm font-medium text-text"#>"Group"</label>
                        <select
                            class=r#"bg-background py-2 px-3 w-full text-sm rounded-md border border-input-border 
                            focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                            name="group"
                            multiple=true
                            on:change=move |ev: Event| {
                                let sel = match ev.target() {
                                    Some(target) => target.unchecked_into::<HtmlSelectElement>(),
                                    None => { push_new_toast(ToastMessageType::ErrorOccurred); return }
                                };
                                let picked = collect_selected_options(&sel);
                                if picked.contains(&"__new__".to_string()) {
                                    group_add_new_selected.set(true);
                                } else {
                                    group_add_new_selected.set(false);
                                    groups_edit.set(picked.join(","));
                                }
                            }
                        >
                            <option value="__new__">"-- Add New --"</option>
                            {move || {
                                view! {
                                    <For
                                        each=move || groups.get()
                                        key=|group: &String| group.clone()
                                        children=move |group| {
                                            let selected = csv_contains(&groups_edit.get(), &group);
                                            
                                            view! {
                                                <option 
                                                    value=group
                                                    selected=selected
                                                >
                                                    {group.clone()}
                                                </option>
                                            }
                                        }
                                    />
                                }
                            }}
                        </select>
                        <input
                            class=r#"bg-background py-2 px-3 mt-2 w-full text-sm rounded-md border border-input-border 
                            focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                            hidden=move || !group_add_new_selected.get()
                            type="text"
                            id="action_create_group_input"
                            value=""
                            on:change=move |ev: Event| {
                                let value = event_target_value(&ev);
                                groups_edit.set(value);
                            }
                        />
                    </div>

                    <div class="grid">
                        <label class=r#"block mb-1 text-sm font-medium"#>"Avatar"</label>
                        <div class="grid gap-2">
                            {move || {
                                let user_avatar = avatar_edit.get();
                                if let Some(user_avatar) = user_avatar {
                                    view! {
                                        <FileTooltip
                                            file_name=user_avatar.file_name.clone()
                                            id=user_avatar.attachment_id.clone()
                                            on_download=format!("/file/{}", user_avatar.attachment_id)
                                            on_remove=Callback::new(move |_| avatar_edit.set(None))
                                        />
                                    }.into_any()
                                } else {
                                    "".into_any()
                                }
                            }}
                            <input
                                class=r#"bg-background w-full text-sm p-3 rounded-lg shadow-sm"#
                                type="file"
                                name="avatar"
                                node_ref=avatar_ref
                            />
                        </div>
                    </div>
                </div>
            </Show>

            <Show when=move || editing_password.get()>
                <label class=r#"block mb-1 text-sm font-medium"#>
                    "New Password"
                </label>
                <div class="flex gap-2">
                    <input
                        class=r#"bg-background py-2 px-3 w-full text-sm rounded-md border border-input-border 
                        focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                        type=move || if new_password_hidden.get() { "password" } else { "text" }
                        name="new_password"
                        bind:value=new_password_edit
                    />
                    <HidePasswordButton hidden=new_password_hidden />
                </div>

                <label class=r#"block mb-1 text-sm font-medium"#>
                    "Confirm New Password"
                </label>
                <div class="flex gap-2">
                    <input
                        class=r#"bg-background py-2 px-3 w-full text-sm rounded-md border border-input-border focus:ring-2 
                        focus:outline-none focus:ring-yale-blue-500"#
                        type=move || if confirm_new_password_hidden.get() { "password" } else { "text" }
                        name="confirm_new_password"
                        bind:value=confirm_new_password_edit
                    />
                    <HidePasswordButton hidden=confirm_new_password_hidden />
                    {move || {
                        if new_password_edit.get() != confirm_new_password_edit.get() {
                            "Confirmation must match"
                        } else {
                            ""
                        }
                    }}
                </div>
            </Show>

            // dont show edit and delete buttons for admin users
            <Show when=move || role_signal.get() != UserRole::Admin.to_string()>
                <div class=r#"flex flex-row-reverse gap-3 mt-2"#>
                    <Show when=move || editing.get() || editing_password.get() || deleting.get()>
                        <button
                            class=r#"py-2 px-4 text-sm rounded-md border border-input-border hover:bg-background-hover"#
                            on:click=move |_| {
                                editing.set(false);
                                deleting.set(false);
                                editing_password.set(false);
                            }
                        >
                            "Cancel"
                        </button>
                    </Show>

                    <button
                        type="button"
                        hidden=move || deleting.get() || editing_password.get()
                        class=r#"inline-flex gap-2 items-center py-2 px-4 text-sm font-medium text-white 
                        rounded-lg transition focus:ring-2 focus:outline-none active:scale-95 
                        bg-yale-blue-600 hover:bg-yale-blue-700 focus:ring-yale-blue-400"#
                        on:click=move |_| {
                            let user_id = id_signal.get_untracked();
                            let username = username_edit.get_untracked();
                            let email = email_edit.get_untracked();
                            let password = new_password_edit.get_untracked();
                            let confirm_password = confirm_new_password_edit.get_untracked();
                            let points = points_edit.get_untracked();
                            let role = role_edit.get_untracked();
                            let groups = groups_edit.get_untracked();

                            let avatar_ref = avatar_ref.get_untracked();

                            if editing.get_untracked() {
                                spawn_local(async move {
                                    tracing::debug!("editing user: {}", user_id.clone());

                                    if let Some(fd) = build_single_file_form_data(avatar_ref) {
                                        if let Ok(api_result) = upload_avatar(fd.into()).await {
                                            avatar_edit.set(Some(api_result.details));
                                        }
                                    }

                                    // using .get_untracked as we're inside an on:click event handler and don't require an active subscription to these values
                                    // consider changing the .get()'s of the initial values above
                                    let avatar = avatar_edit.get_untracked();

                                    if !any_changes_made.get_untracked() {
                                        editing.set(false);
                                        push_new_toast(ToastMessageType::NoChangesMade);
                                    } else {
                                        if let Ok(ApiResult { result, .. }) = crate::server::admin::user(crate::server::admin::UserAction::Edit {
                                                id: user_id,
                                                username: username.clone(),
                                                email: email.clone(),
                                                password: password.clone(),
                                                confirm_password: confirm_password.clone(),
                                                points,
                                                role: role.clone().into(),
                                                avatar,
                                                groups
                                            })
                                            .await && result == ResultStatus::Success
                                        {
                                            push_new_toast(ToastMessageType::UserEdited);
                                            refresh.update(|n| *n += 1);
                                            username_signal.set(username);
                                            email_signal.set(email);
                                            new_password_signal.set(password);
                                            confirm_new_password_signal.set(confirm_password);
                                            points_signal.set(points);
                                            role_signal.set(role);
                                        } else {
                                            push_new_toast(ToastMessageType::UserEditFail);
                                        }
                                    }
                                });
                                editing.set(false)
                            } else {
                                editing.set(true)
                            }
                        }
                    >
                        {move || edit_submit_btn_text.get()}
                    </button>

                    <button
                        type="button"
                        hidden=move || deleting.get() || editing.get()
                        class=r#"inline-flex gap-2 items-center py-2 px-4 text-sm font-medium text-white 
                        rounded-lg transition focus:ring-2 focus:outline-none active:scale-95 
                        bg-yale-blue-600 hover:bg-yale-blue-700 focus:ring-yale-blue-400"#
                        on:click=move |_| {
                            if editing_password.get_untracked() {
                                spawn_local(async move {
                                    tracing::debug!(
                                        "editing user password: {}", id_signal.get_untracked()
                                    );
                                    if let Ok(ApiResult { result, .. }) = crate::server::admin::user(crate::server::admin::UserAction::EditPassword {
                                            id: id_signal.get_untracked(),
                                            password: new_password_edit.get_untracked(),
                                            confirm_password: confirm_new_password_edit.get_untracked(),
                                        })
                                        .await && result == ResultStatus::Success
                                    {
                                        push_new_toast(ToastMessageType::UserPasswordChanged);
                                        refresh.update(|n| *n += 1);
                                    } else {
                                        push_new_toast(ToastMessageType::UserPasswordChangeFail);
                                    }
                                });
                                editing_password.set(false)
                            } else {
                                editing_password.set(true)
                            }
                        }
                    >
                        {move || edit_password_submit_btn_text.get()}
                    </button>

                    <button
                        hidden=move || editing.get() || editing_password.get()
                        class=r#"inline-flex items-center py-2 px-4 ml-auto text-sm font-semibold text-white 
                        bg-red-600 rounded-md shadow-sm hover:bg-red-500 focus:ring-2 focus:outline-none 
                        focus:ring-yale-blue-500"#
                        on:click=move |_| {
                            let id = id_signal.get_untracked();
                            if deleting.get_untracked() {
                                spawn_local(async move {
                                    tracing::debug!("deleting user: {id}");
                                    if let Ok(ApiResult { result, .. }) = crate::server::admin::user(crate::server::admin::UserAction::Delete {
                                            id,
                                        })
                                        .await && result == ResultStatus::Success
                                    {
                                        push_new_toast(ToastMessageType::UserDeleted);
                                        refresh.update(|n| *n += 1);
                                    } else {
                                        push_new_toast(ToastMessageType::UserDeleteFail);
                                    }
                                });
                                deleting.set(false);
                            } else {
                                deleting.set(true);
                            }
                        }
                    >
                        {move || delete_submit_btn_text.get()}
                    </button>
                </div>
            </Show>
        </div>
    }
}
