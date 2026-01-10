use crate::{components::navbar::NavBar, server::{EditUsername, EditPassword, edit_avatar}};
use leptos::{prelude::*, web_sys::{FormData, Event, HtmlInputElement}, wasm_bindgen::JsCast};
use leptos_use::{ColorMode};

/// Default Home Page
#[component]
pub fn Settings() -> impl IntoView {
    let edit_avatar_action = Action::new_local(|data: &FormData| {
        edit_avatar(data.clone().into())
    });
    let edit_username = ServerAction::<EditUsername>::new();
    let edit_password = ServerAction::<EditPassword>::new();
    let set_mode = use_context::<WriteSignal<ColorMode>>().unwrap();

    view! {
        <NavBar />
        <div class="grid justify-center p-4">
            <label>
                "Dark Mode"
                <input type="checkbox" on:input=move |ev| {
                    let is_checked = event_target_checked(&ev);
                    if is_checked { set_mode.set(ColorMode::Dark) } else { set_mode.set(ColorMode::Light) };
                } />
            </label>
            <ActionForm action=edit_username>
                <label>
                    "Change Username" 
                    <input class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" type="text" name="username" />
                    <input class="ml-auto inline-flex items-center px-4 py-2 rounded-md bg-indigo-600 text-white text-sm font-semibold shadow-sm hover:bg-indigo-500 focus:outline-none focus:ring-2 focus:ring-indigo-500" type="submit" value="Submit" />
                </label>
            </ActionForm>
            <ActionForm action=edit_password>
                <label>
                    "Change Password" 
                    <label>
                        "Old Password"
                        <input class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" type="password" name="old_password" />
                    </label>
                    <label>
                        "New Password"
                        <input class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" type="password" name="new_password" />
                    </label>
                    <label>
                        "Confirm New Password"
                        <input class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" type="password" name="confirm_new_password" />
                    </label>
                    <input class="ml-auto inline-flex items-center px-4 py-2 rounded-md bg-indigo-600 text-white text-sm font-semibold shadow-sm hover:bg-indigo-500 focus:outline-none focus:ring-2 focus:ring-indigo-500" type="submit" value="Submit" /> // check if new_password == confirm_new_password
                </label>
            </ActionForm>
            <label>
                <b>"Change Avatar (Max 16 MiB)"</b>
                <input class="bg-white shadow-sm rounded-lg p-2" type="file" name="file"
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
                <p>
                    { move || {
                        if edit_avatar_action.pending().get() {
                            "Uploading...".to_string()
                        // } else if let Some(Ok(val)) = edit_avatar_action.value().get() {
                        //     format!("Uploaded: {}", val.details)
                        // } else {
                        //     "Choose a file".to_string()
                        // }
                        } else {
                            "".to_string()
                        }
                    }}
                </p>
            </label>
        </div>
    }
}
