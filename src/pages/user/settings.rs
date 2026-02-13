use crate::{app::RefreshUser, components::{navbar::NavBar, utils::HidePasswordButton}, server::{EditPassword, edit_avatar, edit_username}};
use leptos::{prelude::*, task::spawn_local, wasm_bindgen::JsCast, web_sys::{Event, FormData, HtmlInputElement}};
use leptos_use::{ColorMode};

/// Default Home Page
#[component]
pub fn Settings() -> impl IntoView {
    let refresh_user = expect_context::<RwSignal<RefreshUser>>();

    let new_username = RwSignal::new("".to_string());
    let edit_avatar_action = Action::new_local(|data: &FormData| {
        edit_avatar(data.clone().into())
    });
    let edit_password = ServerAction::<EditPassword>::new();
    let color_mode = use_context::<Signal<ColorMode>>().unwrap();
    let set_color_mode = use_context::<WriteSignal<ColorMode>>().unwrap();
    let changing_password = RwSignal::new(false);

    let edit_avatar_action_text = Memo::new(move |_| {
        if edit_avatar_action.pending().get() { "Uploading..." } else { "" }
    });

    let old_password_hidden = RwSignal::new(true);
    let new_password_hidden = RwSignal::new(true);
    let confirm_new_password_hidden = RwSignal::new(true);

    view! {
        <NavBar />
        <div class=r#"grid justify-center p-4 h-full bg-background text-text min-h-screen"#>
            <div class="h-1/2 grid gap-2">
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
                        class=r#"py-2 px-3 w-full text-sm rounded-md border border-gray-300 
                        focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                        type="text"
                        bind:value=new_username
                    />
                    <button
                        class=r#"items-center py-2 px-4 ml-auto text-sm font-semibold text-white 
                        rounded-md shadow-sm focus:ring-2 focus:outline-none bg-yale-blue-600 
                        hover:bg-yale-blue-500 focus:ring-yale-blue-500"#

                        on:click=move |_| {
                            let new_username = new_username.get();
                            spawn_local(async move {
                                if edit_username(new_username).await.is_ok() {
                                    refresh_user.update(|r| r.iteration += 1);
                                }
                            });
                        }
                    >
                        "Change Username"
                    </button>
                </div>

                <div class="grid gap-2 items-center">
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
                        <ActionForm action=edit_password>
                            <label>"Old Password"</label>
                            <div class="flex gap-2">
                                <input
                                    class=r#"py-2 px-3 w-full text-sm rounded-md border border-gray-300 
                                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                                    type=move || if old_password_hidden.get() { "password" } else { "text" }
                                    name="old_password"
                                />
                                <HidePasswordButton hidden=old_password_hidden />
                            </div>

                            <label>"New Password"</label>
                            <div class="flex gap-2">
                                <input
                                    class=r#"py-2 px-3 w-full text-sm rounded-md border border-gray-300 
                                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                                    type=move || if new_password_hidden.get() { "password" } else { "text" }
                                    name="new_password"
                                />
                                <HidePasswordButton hidden=new_password_hidden />
                            </div>

                            <label>"Confirm New Password"</label>
                            <div class="flex gap-2">
                                <input
                                    class=r#"py-2 px-3 w-full text-sm rounded-md border border-gray-300 
                                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                                    type=move || {
                                        if confirm_new_password_hidden.get() { "password" } else { "text" }
                                    }
                                    name="confirm_new_password"
                                />
                                <HidePasswordButton hidden=confirm_new_password_hidden />
                            </div>

                            <div class=r#"flex gap-3 mt-2 pt-2"#>
                                <button
                                    class=r#"py-2 px-4 text-sm rounded-md border border-gray-300 hover:bg-background-hover"#
                                    on:click=move |_| { changing_password.set(false) }
                                >
                                    "Cancel"
                                </button>
                                <input
                                    class=r#"items-center py-2 px-4 ml-auto text-sm font-semibold text-white 
                                    rounded-md shadow-sm focus:ring-2 focus:outline-none bg-yale-blue-600 
                                    hover:bg-yale-blue-500 focus:ring-yale-blue-500"#
                                    type="submit"
                                    value="Submit"
                                />
                            </div>
                        </ActionForm>
                    </div>
                </Show>

                <div class="flex gap-2 items-center">
                    <label>
                        <b>"Change Avatar (Max 16 MiB)"</b>
                    </label>
                    <input
                        class=r#"p-2 rounded-lg shadow-sm"#
                        type="file"
                        name="file"
                        on:change=move |ev: Event| {
                            let input = ev.target().unwrap().unchecked_into::<HtmlInputElement>();
                            if let Some(files) = input.files() && files.length() > 0 {
                                let file = files.get(0).unwrap();
                                let fd = FormData::new().unwrap();
                                fd.append_with_blob_and_filename("file", &file, &file.name()).unwrap();
                                edit_avatar_action.dispatch_local(fd);
                            }
                        }
                    />
                    <p>{move || edit_avatar_action_text.get()}</p>
                </div>
            </div>
        </div>
    }
}
