use std::time::Duration;

use crate::app::RefreshUser;
use crate::components::toast::{ToastMessageType, push_new_toast};
use crate::components::utils::TruncatedDesc;
use crate::server::db::structs::{Challenge, ChallengeWithAttachments};
use crate::server::{check_flag, db::structs::AttachmentWithoutBlob, enums::ResultStatus, structs::ApiResult};
use icondata as i;
use leptos::prelude::*;
use leptos_icons::Icon;

#[component]
pub fn Challenge(
    cwa: RwSignal<ChallengeWithAttachments>,
    solved_challenges: RwSignal<Vec<String>>,
    overlay_triggered: RwSignal<bool>,
    cwa_popup: RwSignal<Option<ChallengeWithAttachments>>,
    refresh_solved_challenges: RwSignal<i32>
) -> impl IntoView {
    let flag_signal = RwSignal::new("".to_string());
    let solved = RwSignal::new(false);
    let incorrect = RwSignal::new(false);

    let refresh_user = expect_context::<RwSignal<RefreshUser>>();

    let button_classes = Memo::new(move |_| {
        let base = r#"inline-flex items-center gap-2 rounded-lg text-white px-4 py-2 
            text-sm font-medium focus:outline-none focus:ring-2 active:scale-95 transition"#;
        if solved.get() {
            format!("{base} {}", "bg-green-600 hover:bg-green-700 focus:ring-green-400")
        } else if incorrect.get() {
            format!("{base} {}", "bg-red-600 hover:bg-red-700 focus:ring-red-400")
        } else {
            format!(
                "{base} {}",
                "bg-yale-blue-600 hover:bg-yale-blue-700 focus:ring-yale-blue-400"
            )
        }
    });

    let submit_btn_text = Memo::new(move |_| {
        if solved_challenges.get().contains(&cwa.get().challenge.id) { 
            solved.set(true);
            "Solved" 
        } else if incorrect.get() { 
            "Incorrect"
        } else {
            "Submit"
        }
    });

    Effect::new(move |_| {
        if incorrect.get() {
            set_timeout(move || incorrect.set(false), Duration::from_secs(2));
        }
    });

    let check_flag_action = Action::new(move |(flag, challenge): &(String, Challenge)| {
        let flag = flag.clone();
        let challenge = challenge.clone();
        let challenge_points = challenge.clone().points;
        async move {
            if let Ok(ApiResult { result, details }) = check_flag(flag, challenge).await {
                if result == ResultStatus::Fail && details == "incorrect solution" {
                    incorrect.set(true);
                } else if result == ResultStatus::Success {
                    push_new_toast(ToastMessageType::Custom(format!("Solved challenge +{challenge_points}p")));
                    refresh_user.update(|r| r.iteration += 1);
                    refresh_solved_challenges.update(|r| *r += 1);
                }
            }
        }
    });

    view! {
        <div
            class=r#"content-center p-4 rounded-lg bg-card hover:bg-card-hover break-all"#
        >
            <div class="flex items-center justify-between mb-4">
                <h3 class=r#"font-bold text-3xl/8"#>{move || cwa.get().challenge.name}</h3>
                <button 
                    class="cursor-pointer"
                    on:click=move |_| {
                        overlay_triggered.set(true);
                        cwa_popup.set(Some(cwa.get()));
                    }
                >
                    <Icon icon=i::LuMaximize2 />
                </button>
            </div>

            {move || {
                if let Some(illustration) = cwa.get().illustration { 
                    view! {
                        <div class="flex justify-center m-auto mb-4">
                            <img 
                                src=move || format!("/image/{}", illustration.id) 
                                class=r#"shadow-sm"#
                            />
                        </div>
                    }.into_any()
                } else {
                    "".into_any()
                }
            }}

            <p class=r#"text-lg/8 mt-2 whitespace-pre-wrap"#>
                {move || {
                    let description = RwSignal::new(cwa.get().challenge.description);
                    view! { <TruncatedDesc description /> }
                }}
            </p>

            {move || {
                let difficulty = cwa.get().challenge.difficulty;
                view! { <Difficulty difficulty /> }
            }}

            <p class=r#"text-lg/8"#>
                <b>"Points: "</b>
                {move || cwa.get().challenge.points}
            </p>

            <div class="flex gap-2 items-center">
                <label
                    hidden=move || solved.get()
                    for="flag"
                >
                    <b>"Flag: "</b>
                </label>
                <input
                    hidden=move || solved.get()
                    class=r#"m-1 bg-white rounded-sm text-black"#
                    bind:value=flag_signal
                />
                <button
                    class=move || button_classes.get()
                    disabled=move || solved.get() || incorrect.get()
                    on:click=move |_| {
                        let flag = flag_signal.get();
                        let challenge = cwa.get().challenge;
                        check_flag_action.dispatch((flag, challenge));
                    }
                >
                    {move || submit_btn_text.get()}
                </button>
            </div>

            <div class="grid gap-2 pt-4">
                <div class="flex gap-2 items-center">
                    <For
                        each=move || cwa.get().attachments
                        key=|a: &AttachmentWithoutBlob| a.id.clone()
                        let(a)
                    >
                        {move || a.file_name.clone()}
                        <a
                            download
                            href=move || format!("/file/{}", a.id)
                        >
                            <Icon icon=i::LuDownload />
                        </a>
                    </For>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn Difficulty(difficulty: i8) -> impl IntoView {
    view! {
        {move || {
            view! {
                <div
                    class=r#"difficulty"#
                    role="img"
                    aria-label=format!("Difficulty: {} of 5", difficulty)
                >
                    <span class=r#"label"#>
                        <b class=r#"text-lg/8"#>"Difficulty: "</b>
                        {"⭐".repeat(difficulty as usize)}
                    </span>
                </div>
            }
        }}
    }
}
