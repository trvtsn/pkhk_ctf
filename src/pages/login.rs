use crate::{components::navbar::NavBar, server::{LoginUser, get_user}};
use leptos::prelude::*;
use leptos_router::{components::Redirect, hooks::{use_navigate, use_query_map}};

/// Default Home Page
#[component]
pub fn Login() -> impl IntoView {
    let qmap = use_query_map();
    let email = RwSignal::new("".to_string());
    let password = RwSignal::new("".to_string());
    let loading = RwSignal::new(false);

    let login: ServerAction<LoginUser> = ServerAction::new();

    let button_classes = Memo::new(move |_| {
        let base = "border-2 border-black p-2 text-black rounded";
        if loading.get() {
            format!("{base} {}", "animate-spin")
        } else {
            format!(
                "{base} {}",
                "bg-lavender-blush-100 hover:bg-lavender-blush-200"
            )
        }
    });

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
        // use urlencoding::decode;
        //
        // if let Some(next) = qmap.get().get("c") {
        //     if let Some(Some(_)) = logged_in_user.get() {
        //         let nav = use_navigate();
        //
        //         if let Ok(next) = decode(&next) {
        //             //nav(&next, Default::default());
        //             nav("/", Default::default());
        //         }
        //     }
        // }

        if let Some(Some(_)) = logged_in_user.get() {
            let nav = use_navigate();
            nav("/", Default::default());
        }
    });

    view! {
        <NavBar />
        <div class="p-8 justify-center grid grid-col">
            <h3 class="text-4xl text-center">"Login"</h3>
            <ActionForm action=login>
                <label class="block text-sm font-medium text-gray-700 mb-1">"Email"</label>
                <input class="w-full rounded-md border border-gray-300 bg-white px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" type="email" name="email" bind:value=email />
                
                <label class="block text-sm font-medium text-gray-700 mb-1">"Password"</label>
                <input class="w-full rounded-md border border-gray-300 bg-white px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" type="password" name="password" bind:value=password />
                
                <input
                    type="submit"
                    class="px-4 py-2 rounded-md border border-gray-300 text-sm hover:bg-gray-50"
                    value="Login"
                />
            </ActionForm>
        </div>
    }
}
