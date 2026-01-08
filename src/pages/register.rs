use crate::{components::navbar::NavBar, server::{Register, get_user, user_exists}};
use leptos::{prelude::*, web_sys::{HtmlInputElement, Event}, wasm_bindgen::JsCast};
use leptos_router::hooks::use_navigate;

/// Default Home Page
#[component]
pub fn Register() -> impl IntoView {
    let email = RwSignal::new("".to_string());
    let password = RwSignal::new("".to_string());
    let loading = RwSignal::new(false);

    let register: ServerAction<Register> = ServerAction::new();

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

    let logged_in = Resource::new(move || register.version().get(), move |ver| async move {
        if let Some(user) = get_user().await.ok().and_then(|u| u) {
            if ver > 0 { // only redirect if a *new* registration happened
                let nav = use_navigate();
                nav("/", Default::default());
            }
            Some(user)          
        } else {
            None
        }
    });

    // Return true if the username provided is already taken, false if it is available
    let name_taken = Resource::new(move || email.get(), move |email| async move {user_exists(email).await});

    // Show a piece of text to inform the user of whether or not their chosen username is already
    // taken. A Transition is used here instead of a Suspense so that it can switch back and forth
    // every time the name is edited. Suspend::new is a new way to turn async stuff into events in
    // the reactive system. 
    let available_ui = move || view! {
        <Transition fallback=|| {
            view! { "..." }
        }>
            {move || Suspend::new(async move {
                match name_taken.await {
                    Ok(true) => view! { " Sorry, that one's taken. " }.into_any(),
                    _ => view! { " Available! " }.into_any(),
                }
            })}
        </Transition>
    };

    // Inform the user if they are logged in already, and who they are (people forget these things
    // from time to time) This uses the Either component, which has a ton of relatives for
    // different numbers of options. If you have 7 things, for example, try the Either7 version.
    let login_status = move || Suspend::new(async move {
        match logged_in.await {
            Some(user) => view! { <p>"Logged in as " {user.username}</p> }.into_any(),
            None => view! { <p>"Not logged in yet!"</p> }.into_any()
        } 
    });

    view! {
        <NavBar />
        <div class="container p-8 inline justify-center">
            <h3 class="text-4xl text-center">"Register"</h3>
            <ActionForm action=register>
                <label>
                    <b>"Email"</b>
                    <input class="bg-white border" type="email" name="email" on:blur=move |ev| {
                        let value = event_target_value(&ev);
                        email.set(value);
                    }/>
                    {available_ui}
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
                    value="Register"
                />
            </ActionForm>
            <Transition fallback=|| view! { "Checking login status..." }>{login_status}</Transition>
        </div>
    }
}
