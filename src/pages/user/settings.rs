use cfg_if::cfg_if;
use crate::{components::navbar::NavBar, server::{EditAvatar, EditUsername}};
use leptos::prelude::*;
use leptos_use::{ColorMode};

/// Default Home Page
#[component]
pub fn Settings() -> impl IntoView {
    let edit_username = ServerAction::<EditUsername>::new();
    // let edit_avatar = ServerAction::<EditAvatar>::new();
    let set_mode = use_context::<WriteSignal<ColorMode>>().unwrap();

    view! {
        <NavBar />
        <div class="container">
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
                    <input class="bg-white border" type="text" name="username" />
                    <input class="bg-white border" type="submit" value="Submit" />
                </label>
            </ActionForm>
            // <ActionForm action=edit_avatar>
            //     <label>
            //         "Change Avatar" 
            //         <input class="bg-white border" type="file" name="avatar"/>
            //         <input class="bg-white border" type="submit" value="Submit" />
            //     </label>
            // </ActionForm>
        </div>
    }
}
