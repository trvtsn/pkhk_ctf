use crate::{components::{admin::user::User, utils::HidePasswordButton}, pages::admin::Actions, server::{admin::{get_all_user_groups, get_all_users, upload_avatar}, db::{enums::UserRole, structs::{AttachmentWithoutBlob, DbUser}}, enums::ResultStatus, structs::ApiResult}};
use leptos::{prelude::*, task::spawn_local, wasm_bindgen::JsCast, web_sys::{Event, FormData, HtmlInputElement, HtmlSelectElement, HtmlOptionElement}};

/// Default Home Page
#[component]
pub fn Users() -> impl IntoView {
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
    let avatar_signal = RwSignal::<Option<AttachmentWithoutBlob>>::new(None);
    let group_signal = RwSignal::<String>::new("".to_string());

    let users_resource = Resource::new(move || refresh.get(), move |_| async move {
        get_all_users().await.unwrap_or_default()
    });

    let groups_resource = Resource::new(move || refresh.get(), move |_| async move {
        get_all_user_groups().await.unwrap_or_default()
    });

    let avatar_upload_action = Action::new_local(|data: &FormData| {
        upload_avatar(data.clone().into())
    });

    Effect::new(move |_| {
        if let Some(Ok(api_result)) = avatar_upload_action.value().get() {
            avatar_signal.set(Some(api_result.details.clone()));
        }
    });

    let uploading_avatar_text = Memo::new(move |_| {
        if avatar_upload_action.pending().get() {
            "Uploading...".to_string()
        // } else if let Some(Ok(val)) = upload_action.value().get() {
        //     format!("Uploaded: {}", val.details.file_name)
        // } else {
        } else {
            "".to_string()
        }
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
                <label class=r#"block mb-1 text-sm font-medium text-text"#>"Name"</label>
                <input
                    class=r#"py-2 px-3 w-full text-sm rounded-md border border-gray-300 
                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                    name="username"
                    value=move || username_signal.get()
                    bind:value=username_signal
                />

                <label class=r#"block mb-1 text-sm font-medium text-text"#>"E-mail"</label>
                <input
                    class=r#"py-2 px-3 w-full text-sm rounded-md border border-gray-300 
                    focus:ring-2 focus:outline-none focus:ring--500"#
                    name="email"
                    value=move || email_signal.get()
                    bind:value=email_signal
                />

                <label class=r#"block mb-1 text-sm font-medium text-text"#>"Password"</label>
                <input
                    class=r#"py-2 px-3 w-full text-sm rounded-md border border-gray-300 
                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                    type=move || if password_hidden.get() { "password" } else { "text" }
                    name="password"
                    value=move || password_signal.get()
                    bind:value=password_signal
                />
                <HidePasswordButton hidden=password_hidden />

                <label class=r#"block mb-1 text-sm font-medium text-text"#>
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

                <label class=r#"block mb-1 text-sm font-medium text-text"#>"Role"</label>
                <select
                    class=r#"py-2 px-3 w-full text-sm rounded-md border border-gray-300 
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

                <label class=r#"block mb-1 text-sm font-medium text-text"#>"Group"</label>
                <select
                    class=r#"py-2 px-3 w-full text-sm rounded-md border border-gray-300 
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

                        group_signal.set(picked.join(","));
                    }
                >
                    <option value="">"-- Select Group --"</option>
                    <Suspense fallback=move || {
                        view! { <div>"Loading..."</div> }
                    }>
                        {move || {
                            let groups = groups_resource.get().unwrap_or_default();
                            view! {
                                <For
                                    each=move || groups.clone()
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
                    class=r#"py-2 px-3 mt-2 w-full text-sm rounded-md border border-gray-300 
                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                    hidden=move || !group_add_new_selected.get()
                    type="text"
                    id="action_create_group_input"
                    value=""
                    bind:value=group_signal
                />

                <input
                    class=r#"py-2 px-3 mt-2 w-full text-sm rounded-md border border-gray-300 
                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                    hidden=move || !group_add_new_selected.get()
                    type="text"
                    id="action_create_group_input"
                    bind:value=group_signal
                />

                <label class=r#"block mb-1 text-sm font-medium text-text"#>"Avatar"</label>
                <input
                    class=r#"w-full text-sm"#
                    type="file"
                    name="illustration"
                    on:change=move |ev: Event| {
                        let input = ev.target().unwrap().unchecked_into::<HtmlInputElement>();
                        if let Some(files) = input.files() && files.length() > 0 {
                            let file = files.get(0).unwrap();
                            let fd = FormData::new().unwrap();
                            fd.append_with_blob_and_filename("file", &file, &file.name()).unwrap();
                            avatar_upload_action.dispatch_local(fd);
                        }
                    }
                />
                <p>{uploading_avatar_text.get()}</p>

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
                            let avatar = avatar_signal.get();
                            let group = group_signal.get();
                            spawn_local(async move {
                                tracing::debug!("creating user...");
                                if let Ok(ApiResult { result, .. }) = crate::server::admin::user(crate::server::admin::UserAction::Create {
                                        username,
                                        email,
                                        password,
                                        confirm_password,
                                        role,
                                        avatar,
                                        group
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
            <div class=r#"grid grid-cols-4 gap-4 p-4 m-4"#>
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
