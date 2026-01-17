use crate::{components::{navbar::NavBar, utils::HidePasswordButton}, server::{LoginUser, get_user}};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

/// Default Home Page
#[component]
pub fn Login() -> impl IntoView {
    let email = RwSignal::new("".to_string());
    let password = RwSignal::new("".to_string());
    let password_hidden = RwSignal::new(true);

    let login: ServerAction<LoginUser> = ServerAction::new();

    let logged_in_user = Resource::new(
        move || login.version().get(),
        move |_user| async move {
            if let Ok(Some(user)) = get_user().await {
                Some(user)
            } else {
                None
            }
        }
    );

    Effect::new(move || {
        if let Some(Some(_)) = logged_in_user.get() {
            let nav = use_navigate();
            nav("/", Default::default());
        }
    });

    view! {
        <NavBar />
        <div class="p-8 justify-center grid grid-col">
            <h3 class="text-4xl text-center">"Login"</h3>
            <br/>
            <ActionForm action=login>
                <label class="block text-sm font-medium text-gray-700 mb-1">"Email"</label>
                <input 
                    class="w-full rounded-md border border-gray-300 bg-white px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" 
                    type="email" 
                    name="email" 
                    bind:value=email 
                />
                
                <label class="block text-sm font-medium text-gray-700 mb-1">"Password"</label>
                <input 
                    class="w-full rounded-md border border-gray-300 bg-white px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" 
                    type=move || if password_hidden.get() { "password" } else { "text" }
                    name="password" 
                    bind:value=password 
                /><HidePasswordButton hidden=password_hidden/>
                
                <input
                    type="submit"
                    class="px-4 py-2 rounded-md border border-gray-300 text-sm hover:bg-gray-50"
                    value="Login"
                />
            </ActionForm>
        </div>
    }
}
