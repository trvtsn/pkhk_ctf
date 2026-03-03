use crate::{app::RefreshUser, components::{navbar::NavBar, toast::{ToastAppear, ToastMessageType}, utils::HidePasswordButton}, server::{edit_avatar, edit_password, edit_username, enums::ResultStatus}};
use leptos::{prelude::*, task::spawn_local, wasm_bindgen::JsCast, web_sys::{Event, FormData, HtmlFormElement, HtmlInputElement}};
use leptos_use::{ColorMode};

/// Default Home Page
#[component]
pub fn Settings() -> impl IntoView {
    let toast_message_type = expect_context::<RwSignal<ToastMessageType>>();
    let toast_appear = expect_context::<RwSignal<ToastAppear>>();

    let refresh_user = expect_context::<RwSignal<RefreshUser>>();

    let old_password = RwSignal::new("".to_string());
    let new_password = RwSignal::new("".to_string());
    let confirm_new_password = RwSignal::new("".to_string());
    let new_password_confirm_matches = RwSignal::new(false);
    let new_username = RwSignal::new("".to_string());
    let edit_avatar_action = Action::new_local(move |data: &FormData| {
        let data = data.clone();
        async move {
            if let Ok(_) = edit_avatar(data.clone().into()).await {
                toast_appear.set(true);
                toast_message_type.set(ToastMessageType::AvatarEdited);
            } else {
                toast_appear.set(true);
                toast_message_type.set(ToastMessageType::AvatarEditFail);
            }
        }
    });
    let edit_password = Action::new_local(move |(old_password, new_password, confirm_new_password): &(String, String, String)| {
        let old_password = old_password.clone();
        let new_password = new_password.clone();
        let confirm_new_password = confirm_new_password.clone();
        async move {
            if let Ok(result) = edit_password(old_password, new_password, confirm_new_password).await
            && result.result != ResultStatus::Fail {
                toast_appear.set(true);
                toast_message_type.set(ToastMessageType::UserPasswordChanged);
            } else {
                toast_appear.set(true);
                toast_message_type.set(ToastMessageType::UserPasswordChangeFail);
            }
        }
    });
    let color_mode = use_context::<Signal<ColorMode>>().unwrap();
    let set_color_mode = use_context::<WriteSignal<ColorMode>>().unwrap();
    let changing_password = RwSignal::new(false);

    let edit_avatar_action_text = Memo::new(move |_| {
        if edit_avatar_action.pending().get() { "Uploading..." } else { "" }
    });

    let old_password_hidden = RwSignal::new(true);
    let new_password_hidden = RwSignal::new(true);
    let confirm_new_password_hidden = RwSignal::new(true);
    let uploaded_avatar = RwSignal::new(false);

    view! {
        <NavBar />
        <div class=r#"p-4 bg-background text-text min-h-screen"#>
            <div class="grid gap-4 justify-center">
                <div class="flex gap-2 items-center">
                    <label>"Dark Mode"</label>
                    <input
                        type="checkbox"
                        checked=move || color_mode.get().to_string() == "dark"
                        on:input=move |ev| {
                            let is_checked = event_target_checked(&ev);
                            if is_checked {
                                set_color_mode.set(ColorMode::Dark)
                            } else {
                                set_color_mode.set(ColorMode::Light)
                            };
                        }
                    />
                </div>

                <div class="flex gap-2 items-center">
                    <label>"Change Username"</label>
                    <input
                        class=r#"py-2 px-3 w-full text-sm rounded-md border border-input-border 
                        focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                        type="text"
                        bind:value=new_username
                    />
                    <button
                        class=r#"items-center py-2 px-4 ml-auto text-sm font-semibold text-white 
                        rounded-md shadow-sm focus:ring-2 focus:outline-none bg-yale-blue-600 
                        hover:bg-yale-blue-500 focus:ring-yale-blue-500"#
                        disabled=move || new_username.get().is_empty()
                        on:click=move |_| {
                            let new_username = new_username.get();
                            if !new_username.is_empty() {
                                spawn_local(async move {
                                    if edit_username(new_username).await.is_ok() {
                                        toast_appear.set(true);
                                        toast_message_type.set(ToastMessageType::UserUsernameChanged);
                                        refresh_user.update(|r| r.iteration += 1);
                                    } else {
                                        toast_appear.set(true);
                                        toast_message_type.set(ToastMessageType::UserUsernameChangeFail);
                                    }
                                });
                            }
                        }
                    >
                        "Change Username"
                    </button>
                </div>

                <div class="grid gap-2 items-center justify-start">
                    <button
                        class=r#"py-2 px-4 ml-auto text-sm font-semibold text-white
                        rounded-md shadow-sm focus:ring-2 focus:outline-none bg-yale-blue-600 
                        hover:bg-yale-blue-500 focus:ring-yale-blue-500"#
                        on:click=move |_| {
                            if changing_password.get() {
                                changing_password.set(false)
                            } else {
                                changing_password.set(true)
                            }
                        }
                    >
                        "Change Password"
                    </button>
                </div>

                <Show when=move || changing_password.get()>
                    <div class="grid gap-2">
                        <form 
                            class="flex flex-col gap-4"
                            on:submit=move |ev| {
                                ev.prevent_default();
                                let old_password = old_password.get();
                                let new_password = new_password.get();
                                let confirm_new_password = confirm_new_password.get();
                                edit_password.dispatch((old_password, new_password, confirm_new_password));
                            }
                        >
                            <label>"Old Password"</label>
                            <div class="flex gap-2">
                                <input
                                    class=r#"py-2 px-3 w-full text-sm rounded-md border border-input-border 
                                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                                    type=move || if old_password_hidden.get() { "password" } else { "text" }
                                    name="old_password"
                                    bind:value=old_password
                                />
                                <HidePasswordButton hidden=old_password_hidden />
                            </div>

                            <label>"New Password"</label>
                            <div class="flex gap-2">
                                <input
                                    class=r#"py-2 px-3 w-full text-sm rounded-md border border-input-border 
                                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                                    type=move || if new_password_hidden.get() { "password" } else { "text" }
                                    name="new_password"
                                    bind:value=new_password
                                />
                                <HidePasswordButton hidden=new_password_hidden />
                            </div>

                            <label>"Confirm New Password"</label>
                            <div class="flex gap-2">
                                <input
                                    class=r#"py-2 px-3 w-full text-sm rounded-md border border-input-border 
                                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                                    type=move || {
                                        if confirm_new_password_hidden.get() { "password" } else { "text" }
                                    }
                                    name="confirm_new_password"
                                    bind:value=confirm_new_password
                                />
                                <HidePasswordButton hidden=confirm_new_password_hidden />
                            </div>

                            {move || if new_password.get() != confirm_new_password.get() {
                                "Confirmation must match"
                            } else {
                                new_password_confirm_matches.set(true);
                                ""
                            }}

                            <div class=r#"flex gap-3 mt-2 pt-2"#>
                                <button
                                    class=r#"py-2 px-4 text-sm rounded-md border border-input-border hover:bg-background-hover"#
                                    on:click=move |_| { changing_password.set(false) }
                                >
                                    "Cancel"
                                </button>
                                <input
                                    class=r#"items-center py-2 px-4 ml-auto text-sm font-semibold text-white 
                                    rounded-md shadow-sm focus:ring-2 focus:outline-none bg-yale-blue-600 
                                    hover:bg-yale-blue-500 focus:ring-yale-blue-500"#
                                    type="submit"
                                    disabled=move || !new_password_confirm_matches.get()
                                    value="Submit"
                                />
                            </div>
                        </form>
                    </div>
                </Show>

                <div class="flex gap-2 items-center">
                    <label>
                        <b>"Change Avatar (Max 16 MiB)"</b>
                    </label>
                    <form 
                        class="flex gap-2"
                        on:submit=move |ev| {
                            ev.prevent_default();
                            if !uploaded_avatar.get() {
                                return;
                            } else {
                                let target = ev.target().unwrap().unchecked_into::<HtmlFormElement>();
                                let fd = FormData::new_with_form(&target).unwrap();
                                edit_avatar_action.dispatch_local(fd);
                            }
                        }
                    >
                        <input
                            class=r#"p-2 rounded-lg shadow-sm bg-background-secondary"#
                            type="file"
                            name="file"
                            required
                            on:change=move |ev: Event| {
                                let input = ev.target().unwrap().unchecked_into::<HtmlInputElement>();
                                if input.files().is_some() {
                                    uploaded_avatar.set(true);
                                }
                            }
                        />
                        <input
                            class=r#"inline-flex items-center py-2 px-4 ml-auto text-sm font-semibold text-white 
                            rounded-md shadow-sm focus:ring-2 focus:outline-none bg-yale-blue-600 
                            hover:bg-yale-blue-500 focus:ring-yale-blue-500"#
                            disabled=move || !uploaded_avatar.get()
                            type="submit"
                            value="Submit"
                        />
                    </form>
                    <p>{move || edit_avatar_action_text.get()}</p>
                </div>
            </div>
        </div>
    }
}
