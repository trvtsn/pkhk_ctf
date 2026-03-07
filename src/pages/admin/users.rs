use crate::{components::{admin::user::User, toast::{ToastMessageType, push_new_toast}, utils::{ComponentSize, FileTooltip, HidePasswordButton, Spinner}}, pages::admin::Actions, server::{admin::{get_all_user_groups, get_all_users, upload_avatar}, db::{enums::UserRole, structs::{DbUser, UserAvatar}}, enums::ResultStatus, get_all_user_avatar_ids, structs::ApiResult}};
use crate::utils::{build_single_file_form_data, collect_selected_options};
use leptos::{prelude::*, task::spawn_local, wasm_bindgen::JsCast, web_sys::{Event, HtmlSelectElement}};

/// Default Home Page
#[component]
pub fn Users() -> impl IntoView {
    let avatar_ref = NodeRef::new();

    let section = RwSignal::new(Actions::None);
    let refresh = RwSignal::new(0);
    let creating = RwSignal::new(false);
    let password_hidden = RwSignal::new(true);
    let confirm_password_hidden = RwSignal::new(true);
    let group_add_new_selected = RwSignal::new(false);
    
    let username_signal = RwSignal::new("".to_string());
    let email_signal = RwSignal::new("".to_string());
    let password_signal = RwSignal::new("".to_string());
    let confirm_password_signal = RwSignal::new("".to_string());
    let roles_signal = RwSignal::new(vec![UserRole::Admin, UserRole::Competitor]);
    let role_signal = RwSignal::new("".to_string());
    let avatar_signal = RwSignal::<Option<UserAvatar>>::new(None);
    let groups_signal = RwSignal::<String>::new("".to_string());

    let avatars_signal = RwSignal::new(vec![]);
    let avatars_resource = Resource::new(move || refresh.get(), move |_| async move {
        get_all_user_avatar_ids().await.unwrap_or_default()
    });

    let users_resource = Resource::new(move || refresh.get(), move |_| async move {
        get_all_users().await.unwrap_or_default()
    });

    let user_groups_signal = RwSignal::new(vec![]);
    let user_groups_resource = Resource::new(move || refresh.get(), move |_| async move {
        get_all_user_groups().await.unwrap_or_default()
    });

    view! {
        <div class=r#"flex gap-2 mb-4"#>
            <button
                class=r#"py-1 px-3 text-sm rounded-md border border-input-border hover:bg-background-hover"#
                on:click=move |_| {
                    if creating.get_untracked() {
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
                <div class="grid">
                    <label class=r#"block mb-1 text-sm font-medium text-text"#>"Name"</label>
                    <input
                        class=r#"bg-background py-2 px-3 w-full text-sm rounded-md border border-input-border 
                        focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                        name="username"
                        bind:value=username_signal
                    />
                </div>

                <div class="grid">
                    <label class=r#"block mb-1 text-sm font-medium text-text"#>"E-mail"</label>
                    <input
                        class=r#"bg-background py-2 px-3 w-full text-sm rounded-md border border-input-border 
                        focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                        name="email"
                        bind:value=email_signal
                    />
                </div>

                <div class="grid">
                    <label class=r#"block mb-1 text-sm font-medium text-text"#>"Password"</label>
                    <div class="flex gap-2">
                        <input
                            class=r#"bg-background py-2 px-3 w-full text-sm rounded-md border border-input-border 
                            focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                            type=move || if password_hidden.get() { "password" } else { "text" }
                            name="password"
                            bind:value=password_signal
                        />
                        <HidePasswordButton hidden=password_hidden />
                    </div>
                </div>

                <div class="grid">
                    <label class=r#"block mb-1 text-sm font-medium text-text"#>
                        "Confirm Password"
                    </label>
                    <div class="flex gap-2">
                        <input
                            class=r#"bg-background py-2 px-3 w-full text-sm rounded-md border border-input-border 
                            focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                            type=move || if confirm_password_hidden.get() { "password" } else { "text" }
                            name="confirm_password"
                            bind:value=confirm_password_signal
                        />
                        <HidePasswordButton hidden=confirm_password_hidden />
                    </div>
                    {move || {
                        if password_signal.get() != confirm_password_signal.get() {
                            "Confirmation must match"
                        } else {
                            ""
                        }
                    }}
                </div>

                <div class="grid">
                    <label class=r#"block mb-1 text-sm font-medium text-text"#>"Role"</label>
                    <select
                        class=r#"bg-background py-2 px-3 w-full text-sm rounded-md border border-input-border 
                        focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                        name="role"
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
                                groups_signal.set(picked.join(","));
                            }
                        }
                    >
                        <option value="">"-- Select Group --"</option>
                        <Suspense fallback=move || {
                            view! { <Spinner component_size=ComponentSize::Small /> }
                        }>
                            {move || {
                                view! {
                                    <For
                                        each=move || user_groups_signal.get()
                                        key=|group: &String| group.clone()
                                        let(group)
                                    >
                                        <option value=group>
                                            {group.clone()}
                                        </option>
                                    </For>
                                }
                            }}
                        </Suspense>
                        <option value="__new__">"-- Add New --"</option>
                    </select>
                    <input
                        class=r#"py-2 px-3 mt-2 w-full text-sm rounded-md border border-input-border 
                        focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                        hidden=move || !group_add_new_selected.get()
                        type="text"
                        id="action_create_group_input"
                        value=""
                        on:change=move |ev: Event| {
                            let value = event_target_value(&ev);
                            groups_signal.set(value);
                        }
                    />
                </div>

                <div class="grid">
                    <label class=r#"block mb-1 text-sm font-medium text-text"#>"Avatar"</label>
                    <div class="grid gap-2">
                        {move || {
                            let user_avatar = avatar_signal.get();
                            if let Some(user_avatar) = user_avatar {
                                view! {
                                    <FileTooltip
                                        file_name=user_avatar.file_name.clone()
                                        id=user_avatar.attachment_id.clone()
                                        on_download=format!("/file/{}", user_avatar.attachment_id)
                                        on_remove=Callback::new(move |_| avatar_signal.set(None))
                                    />
                                }.into_any()
                            } else {
                                "".into_any()
                            }
                        }}
                    </div>
                    <input
                        class=r#"bg-background w-full text-sm p-3 rounded-lg shadow-sm"#
                        type="file"
                        name="avatar"
                        node_ref=avatar_ref
                    />
                </div>

                <div class=r#"flex gap-3 mt-2"#>
                    <button
                        type="button"
                        class=r#"py-2 px-4 text-sm rounded-md border border-input-border hover:bg-background-hover"#
                        on:click=move |_| section.set(Actions::None)
                    >
                        "Cancel"
                    </button>
                    <button
                        type="button"
                        class=r#"inline-flex items-center py-2 px-4 ml-auto text-sm font-semibold 
                        text-white rounded-md shadow-sm focus:ring-2 focus:outline-none 
                        bg-yale-blue-600 hover:bg-yale-blue-500 focus:ring-yale-blue-500"#
                        on:click=move |_| {
                            let username = username_signal.get_untracked();
                            let email = email_signal.get_untracked();
                            let password = password_signal.get_untracked();
                            let confirm_password = confirm_password_signal.get_untracked();
                            let role = role_signal.get_untracked().into();
                            let groups = groups_signal.get_untracked();

                            let avatar_ref = avatar_ref.get_untracked();

                            spawn_local(async move {
                                tracing::debug!("creating user...");
                                
                                if let Some(fd) = build_single_file_form_data(avatar_ref) {
                                    if let Ok(api_result) = upload_avatar(fd.into()).await {
                                        avatar_signal.set(Some(api_result.details));
                                    }
                                }

                                let avatar = avatar_signal.get_untracked();

                                if let Ok(ApiResult { result, .. }) = crate::server::admin::user(crate::server::admin::UserAction::Create {
                                        username,
                                        email,
                                        password,
                                        confirm_password,
                                        role,
                                        avatar,
                                        groups
                                    })
                                    .await && result == ResultStatus::Success
                                {
                                    push_new_toast(ToastMessageType::UserCreated);
                                    refresh.update(|n| *n += 1);
                                } else {
                                    push_new_toast(ToastMessageType::UserCreateFail);
                                }
                            });
                        }
                    >
                        "Create"
                    </button>
                </div>
            </Show>
        </div>

        <Transition fallback=move || view! { <Spinner component_size=ComponentSize::Big /> }>
            {move || {
                let user_groups = user_groups_resource.get().unwrap_or_default();
                user_groups_signal.set(user_groups);

                let avatars = avatars_resource.get().unwrap_or_default();
                avatars_signal.set(avatars);
                view! {
                    <div class=r#"grid grid-cols-4 gap-4 pt-4 items-start"#>
                        <For
                            each=move || users_resource.get().unwrap_or_default()
                            key=|user: &DbUser| user.id.clone()
                            let(user)
                        >
                            <User 
                                user 
                                user_avatars=avatars_signal
                                groups=user_groups_signal
                                refresh 
                            />
                        </For>
                    </div>
                }
            }}

        </Transition>
    }
}
