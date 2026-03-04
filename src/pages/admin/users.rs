use crate::{components::{admin::user::User, toast::{ToastMessageType, push_new_toast}, utils::{ComponentSize, HidePasswordButton, Spinner}}, pages::admin::Actions, server::{admin::{get_all_user_groups, get_all_users, upload_avatar}, db::{enums::UserRole, structs::{DbUser, UserAvatar}}, enums::ResultStatus, get_all_user_avatar_ids, structs::ApiResult}};
use icondata as i;
use leptos::{prelude::*, task::spawn_local, wasm_bindgen::JsCast, web_sys::{Event, FormData, HtmlInputElement, HtmlSelectElement, HtmlOptionElement}};
use leptos_icons::Icon;

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
                <div class="grid">
                    <label class=r#"block mb-1 text-sm font-medium text-text"#>"Name"</label>
                    <input
                        class=r#"bg-background py-2 px-3 w-full text-sm rounded-md border border-input-border 
                        focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                        name="username"
                        value=move || username_signal.get()
                        bind:value=username_signal
                    />
                </div>

                <div class="grid">
                    <label class=r#"block mb-1 text-sm font-medium text-text"#>"E-mail"</label>
                    <input
                        class=r#"bg-background py-2 px-3 w-full text-sm rounded-md border border-input-border 
                        focus:ring-2 focus:outline-none focus:ring--500"#
                        name="email"
                        value=move || email_signal.get()
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
                            value=move || password_signal.get()
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
                            value=move || confirm_password_signal.get()
                            bind:value=confirm_password_signal
                        />
                        <HidePasswordButton hidden=confirm_password_hidden />
                    </div>
                </div>

                <div class="grid">
                    <label class=r#"block mb-1 text-sm font-medium text-text"#>"Role"</label>
                    <select
                        class=r#"bg-background py-2 px-3 w-full text-sm rounded-md border border-input-border 
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
                    {move || {
                        if password_signal.get() != confirm_password_signal.get() {
                            "Confirmation must match"
                        } else {
                            ""
                        }
                    }}
                </div>

                <div class="grid">
                    <label class=r#"block mb-1 text-sm font-medium text-text"#>"Group"</label>
                    <select
                        class=r#"bg-background py-2 px-3 w-full text-sm rounded-md border border-input-border 
                        focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                        name="group"
                        multiple=true
                        on:change=move |ev: Event| {
                            let sel = ev.target().unwrap().unchecked_into::<HtmlSelectElement>();
                            let doc = leptos::web_sys::window().unwrap().document().unwrap();
                            let new_input = doc
                                .get_element_by_id("action_create_group_input")
                                .unwrap()
                                .unchecked_into::<HtmlInputElement>();
                            if sel.value() == "__new__" {
                                let _ = sel.remove_attribute("name");
                                let _ = new_input.set_attribute("name", "group");
                                group_add_new_selected.set(true);
                            } else {
                                let _ = sel.set_attribute("name", "group");
                                let _ = new_input.remove_attribute("name");
                                group_add_new_selected.set(false);
                            }

                            let selected = sel.selected_options();
                            let mut picked: Vec<String> = Vec::new();

                            for i in 0..selected.length() {
                                if let Some(item) = selected.item(i) {
                                    if let Ok(opt) = item.dyn_into::<HtmlOptionElement>() {
                                        picked.push(opt.value());
                                    }
                                }
                            }

                            groups_signal.set(picked.join(","));
                        }
                    >
                        <option value="">"-- Select Group --"</option>
                        <Suspense fallback=move || {
                            view! { <Spinner component_size=ComponentSize::Small /> }
                        }>
                            {move || {
                                let user_groups = user_groups_resource.get().unwrap_or_default();
                                view! {
                                    <For
                                        each=move || user_groups.clone()
                                        key=|group: &String| group.clone()
                                        let(group)
                                    >
                                        <option value={group.clone()}>{group.clone()}</option>
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
                                let show_tooltip = RwSignal::new(false);
                                let id = user_avatar.attachment_id.clone();
                                view! {
                                    <div class="flex gap-2 items-center">
                                        <span
                                            class="relative inline-block"
                                            on:mouseenter=move |_| show_tooltip.set(true)
                                            on:mouseleave=move |_| show_tooltip.set(false)
                                            on:focus=move |_| show_tooltip.set(true)
                                            on:blur=move |_| show_tooltip.set(false)
                                            tabindex="0"
                                        >
                                            {move || user_avatar.file_name.clone()}
                                            <Show when=move || show_tooltip.get()>
                                                <div
                                                    role="tooltip"
                                                    class=r#"absolute left-1/2 bottom-full -translate-x-1/2 whitespace-nowrap 
                                                        rounded p-1 text-xs bg-card-hover shadow-sm z-1"#
                                                >
                                                    {format!("ID: {}", id)}
                                                </div>
                                            </Show>
                                        </span>
                                        <a
                                            download
                                            href=move || format!("/file/{}", user_avatar.attachment_id.clone())
                                        >
                                            <Icon icon=i::LuDownload />
                                        </a>
                                        <button 
                                            class="cursor-pointer"
                                            on:click=move |_| {
                                                avatar_signal.set(None);
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
                            let groups = groups_signal.get();
                            spawn_local(async move {
                                tracing::debug!("creating user...");
                                
                                if let Some(avatar_el) = avatar_ref.get() {
                                    if let Some(files) = avatar_el.files() {
                                        if files.length() > 0 {
                                            let file = files.get(0).unwrap();
                                            let fd = FormData::new().unwrap();
                                            fd.append_with_blob_and_filename("file", &file, &file.name()).unwrap();

                                            if let Ok(api_result) = upload_avatar(fd.into()).await {
                                                avatar_signal.set(Some(api_result.details.clone()));
                                            }
                                        }
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
