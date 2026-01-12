use crate::components::utils::TruncatedDesc;
use crate::server::db::structs::ChallengeWithAttachments;
use crate::server::{check_flag, db::structs::{AttachmentWithoutBlob, Challenge}, enums::ResultStatus, structs::ApiResult};
use leptos::{prelude::*, task::spawn_local};
use leptos_use::{use_timeout_fn, UseTimeoutFnReturn};
// use thaw::*;

#[component]
pub fn Challenge(
    cwa: ChallengeWithAttachments,
    solved_challenges: RwSignal<Vec<String>>
) -> impl IntoView {
    let ChallengeWithAttachments { challenge, attachments } = cwa;
    let challenge_signal = RwSignal::new(challenge.clone());
    let description_signal = RwSignal::new(challenge.description.clone());
    let flag_signal = RwSignal::new("".to_string());

    let solved = RwSignal::new(false);
    let incorrect = RwSignal::new(false);

    let button_classes = Memo::new(move |_| {
        let base = "inline-flex items-center gap-2 rounded-lg text-white px-4 py-2 text-sm font-medium focus:outline-none focus:ring-2 active:scale-95 transition";
        if solved.get() {
            format!("{base} {}", "bg-green-600 hover:bg-green-700 focus:ring-green-400")
        } else if incorrect.get() {
            format!("{base} {}", "bg-red-600 hover:bg-red-700 focus:ring-red-400")
        } else {
            format!(
                "{base} {}",
                "bg-indigo-600 hover:bg-indigo-700 focus:ring-indigo-400"
            )
        }
    });
    let attachment_path = RwSignal::new("".to_string());

    let UseTimeoutFnReturn { start, stop, .. } =
        use_timeout_fn(move |_: ()| {
            // runs after the delay on the client
            incorrect.set(false);
        }, 3000.0);

    let open = RwSignal::new(false);

    let btn_text = Memo::new(move |_| {
        if solved_challenges.get().contains(&challenge_signal.get().id) { "Solved".to_string() } else { "Submit".to_string() }
    });

    view! {
        <div
            class="bg-yale-blue-50 hover:bg-yale-blue-100 rounded-lg p-4 content-center"
            on:click=move |_| {
                open.set(true);
            }
        >
            <h3 class="text-3xl/8">{move || challenge_signal.get().name.clone()}</h3>
            <p class="text-lg/8">
                <TruncatedDesc description_signal/>
            </p>
            <Difficulty rating=challenge_signal.get().difficulty />
            <p class="text-lg/8"><b>"Points: "</b> {challenge_signal.get().points}</p>
            <br />
                
            <label for="flag"><b>"Flag: "</b></label>
            <input class="border-black border-1 rounded-sm bg-white m-1" bind:value=flag_signal/>
            <button
                class=move || button_classes.get()
                disabled=move || solved.get() || incorrect.get()
                on:click=move |_| {
                    let start = start.clone();
                    let stop = stop.clone();
                    spawn_local(async move {
                        if let Ok(ApiResult { result, details }) = check_flag(flag_signal.get(), challenge_signal.get()).await {
                            // change button appearance to red, incorrect
                            if result == ResultStatus::Fail && details == "incorrect solution" {
                                incorrect.set(true);
                                // cancel previous pending timeout (if any)
                                stop();
                                start(());
                            } else if result == ResultStatus::Success {
                                solved.set(true);
                            }
                        }
                    });
                }
            >
                { move || btn_text.get() }
            </button>

            <For
                each=move || attachments.clone()
                key=|a: &AttachmentWithoutBlob| a.id.clone()
                let(a)
            >
                {attachment_path.set(format!("/file/{}", a.id))}
                <a download href=move || attachment_path.get() class="underline text-blue-600">{a.file_name}</a>
            </For>
        </div>
    }
}

#[component]
pub fn Difficulty(#[prop(default = 3)] rating: i8) -> impl IntoView {
    let rating = rating.clamp(1, 5);

    view! {
        <div class="difficulty" role="img" aria-label=format!("Difficulty: {} of 5", rating)>
            <span class="label">
                <b class="text-lg/8">"Difficulty: "</b>
                {"⭐".repeat(rating as usize)}
            </span>
        </div>
    }
}
