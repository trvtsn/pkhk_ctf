use crate::{components::navbar::NavBar, server::{EditAvatar, EditUsername}};
use leptos::prelude::*;

/// Default Home Page
#[component]
pub fn Settings() -> impl IntoView {
    let dark_mode = RwSignal::new(false);
    let edit_username = ServerAction::<EditUsername>::new();
    // let edit_avatar = ServerAction::<EditAvatar>::new();

    view! {
        <NavBar />
        <div class="container">
            <label>
                "Dark Mode" 
                <input type="checkbox" />
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
