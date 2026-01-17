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
    let set_mode = use_context::<WriteSignal<ColorMode>>().unwrap();
    let changing_password = RwSignal::new(false);

    let edit_avatar_action_text = Memo::new(move |_| {
        if edit_avatar_action.pending().get() { "Uploading..." } else { "" }
    });

    let password_hidden = RwSignal::new(true);

    let password_input_type = Memo::new(move |_| {
        if password_hidden.get() {
            "password"
        } else {
            "text"
        }
    });

    view! {
        <NavBar />
        <div class="grid justify-center p-4">
            <label>"Dark Mode"</label>
            <input type="checkbox" on:input=move |ev| {
                let is_checked = event_target_checked(&ev);
                if is_checked { set_mode.set(ColorMode::Dark) } else { set_mode.set(ColorMode::Light) };
            }/>
            
            <label>"Change Username"</label>
            <input 
                class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" 
                type="text" 
                bind:value=new_username
            />
            <button
                class=r#"ml-auto inline-flex items-center px-4 py-2 rounded-md bg-indigo-600 text-white text-sm 
                font-semibold shadow-sm hover:bg-indigo-500 focus:outline-none focus:ring-2 focus:ring-indigo-500"# 
                on:click=move |_| {
                    spawn_local(async move {
                        if edit_username(new_username.get()).await.is_ok() {
                            let iteration = refresh_user.get().iteration + 1;
                            refresh_user.set(RefreshUser { iteration });
                        }
                    });
                }
            >"Change Username"</button>

            <button
                class=r#"ml-auto inline-flex items-center px-4 py-2 rounded-md bg-indigo-600 text-white text-sm 
                font-semibold shadow-sm hover:bg-indigo-500 focus:outline-none focus:ring-2 focus:ring-indigo-500"#
                on:click=move |_| {
                    if changing_password.get() {changing_password.set(false)} else {changing_password.set(true)}

                }
            >"Change Password"</button>

            <Show when=move || changing_password.get()>
                <ActionForm action=edit_password>
                    <label>"Old Password"</label>
                    <input 
                        class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" 
                        type=move || password_input_type.get()
                        name="old_password" 
                    /><HidePasswordButton hidden=password_hidden/>
                    
                    <label>"New Password"</label>
                    <input 
                        class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" 
                        type=move || password_input_type.get()
                        name="new_password" 
                    /><HidePasswordButton hidden=password_hidden/>
                    
                    <label>"Confirm New Password"</label>
                    <input 
                        class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" 
                        type=move || password_input_type.get()
                        name="confirm_new_password" 
                    /><HidePasswordButton hidden=password_hidden/>
                    
                    <button 
                        class="px-4 py-2 rounded-md border border-gray-300 text-sm hover:bg-gray-50" 
                        on:click=move |_| {changing_password.set(false)}
                    >"Cancel"</button>
                    <input 
                        class=r#"ml-auto inline-flex items-center px-4 py-2 rounded-md bg-indigo-600 text-white text-sm 
                        font-semibold shadow-sm hover:bg-indigo-500 focus:outline-none focus:ring-2 focus:ring-indigo-500"# 
                        type="submit" 
                        value="Submit" 
                    />
                </ActionForm>
            </Show>
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
                <p>{move || edit_avatar_action_text.get()}</p>
            </label>
        </div>
    }
}
