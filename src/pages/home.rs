use crate::components::navbar::NavBar;
use leptos::prelude::*;
//use thaw::*;

/// Default Home Page
#[component]
pub fn Home() -> impl IntoView {
    view! {
        // <div class="min-h-screen flex flex-col animated-background bg-linear-to-bl from-yale-blue-800 via-lavender-blush-600 to-tomato-jam-600">
        <div class="min-h-screen flex flex-col bg-linear-to-bl from-yale-blue-800 via-lavender-blush-600 to-tomato-jam-600">
            <NavBar />
            <div class="flex-1 flex items-center justify-center h-screen text-center overflow-auto">
                // <Image src="/public/blurry-gradient-haikei.svg" />
                <div class="p-8">
                    <h1 class="text-9xl">"Capture The Flag"</h1>
                    <br />
                    <h3 class="text-4xl">"PKHK"</h3>
                </div>
            </div>
        </div>
    }
}
