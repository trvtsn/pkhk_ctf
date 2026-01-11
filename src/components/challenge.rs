use crate::server::{check_flag, db::structs::{AttachmentWithoutBlob, Challenge}, enums::ResultStatus, structs::ApiResult};
use leptos::{prelude::*, task::spawn_local};
use leptos_use::{use_timeout_fn, UseTimeoutFnReturn};
// use thaw::*;

#[component]
pub fn Challenge(
    id: String,
    name: String,
    description: Option<String>,
    event_id: String,
    category: Option<String>,
    #[prop(default = 3)] difficulty: i8,
    points: u32,
    #[prop(optional)] attachments: Vec<AttachmentWithoutBlob>,
    solved_challenges: RwSignal<Vec<String>>
) -> impl IntoView {
    let full_desc = description.clone().unwrap_or_default();
    let desc_max_len = 200usize;
    let desc_expanded = RwSignal::new(false);
    let needs_truncate = full_desc.chars().count() > desc_max_len;
    let truncated_desc = if needs_truncate {
        full_desc.chars().take(desc_max_len).collect::<String>()
    } else {
        full_desc.clone()
    };
    let flag = RwSignal::new("".to_string());
    // let check_flag: ServerAction<CheckFlag> = ServerAction::<CheckFlag>::new();
    // let input_flag = RwSignal::new("".to_string());
    let solved = RwSignal::new(false);
    let incorrect = RwSignal::new(false);
    // let loading = RwSignal::new(false);
    //
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

    let challenge_id = id.clone();
    let btn_text = Memo::new(move |_| {
        if solved_challenges.get().contains(&challenge_id) { "Solved".to_string() } else { "Submit".to_string() }
    });

    view! {
        <div
            class="bg-yale-blue-50 hover:bg-yale-blue-100 rounded-lg p-4 content-center"
            on:click=move |_| {
                open.set(true);
            }
        >
            <h3 class="text-3xl/8">{name.clone()}</h3>
            <p class="text-lg/8">{
                move || {
                    if desc_expanded.get() || !needs_truncate {
                        full_desc.clone()
                    } else {
                        format!("{}...", truncated_desc)
                    }
                }
            }
            {
                if needs_truncate {
                    view! {
                        <button
                            class="ml-2 text-base underline text-blue-600 cursor-pointer"
                            on:click=move |_| {
                                desc_expanded.set(!desc_expanded.get());
                            }
                        >
                            { move || if desc_expanded.get() { "Show Less" } else { "Show More" } }
                        </button>
                    }.into_any()
                } else {
                    view! {}.into_any()
                }
            }
            </p>

            <Difficulty rating=difficulty />
            <p class="text-lg/8"><b>"Points: "</b> {points}</p>
            <br />
            <label for="flag"><b>"Flag: "</b></label>
            <input
                class="border-black border-1 rounded-sm bg-white m-1"
                on:input=move |ev| {
                    let val = event_target_value(&ev);
                    flag.set(val);
                }
            />
            <button
                class=move || button_classes.get()
                disabled=move || solved.get() || incorrect.get()
                on:click=move |_| {
                    let id = id.clone();
                    let event_id = event_id.clone();
                    let flag = flag.get().clone();
                    let name = name.clone();
                    let description = description.clone();
                    let category = category.clone();
                    let start = start.clone();
                    let stop = stop.clone();
                    spawn_local(async move {
                        if let Ok(ApiResult { result, details }) = check_flag(flag, Challenge { id, event_id, name, description, category, difficulty, points }).await {
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
