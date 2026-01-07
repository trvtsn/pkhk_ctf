use crate::{components::navbar::NavBar, server::LoginUser};
use leptos::prelude::*;
use leptos_router::components::Redirect;

/// Default Home Page
#[component]
pub fn Login() -> impl IntoView {
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

    view! {
        <NavBar />
        <div class="container p-8 inline justify-center">
            <h3 class="text-4xl text-center">"Login"</h3>
            <ActionForm action=login>
                <label>
                    <b>"Email"</b>
                    <input class="bg-white border" type="email" name="email" bind:value=email />
                </label>
                <label>
                    <b>"Password"</b>
                    <input class="bg-white border" type="password" name="password" bind:value=password />
                </label>
                //<button loading=loading on_click=move |_| { loading.set(true) }>
                <input
                    type="submit"
                    class=r#"flex w-full justify-center rounded-md bg-indigo-600 px-3 py-1.5 text-sm font-semibold
                            leading-6 text-white shadow-sm hover:bg-indigo-500 focus-visible:outline 
                            focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600"#
                    value="Login"
                />
            </ActionForm>
        </div>
    }
}
