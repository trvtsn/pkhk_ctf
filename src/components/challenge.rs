use crate::server::{CheckFlag, check_flag, db::structs::{Attachment, Challenge}, enums::ResultStatus, structs::ApiResult};
use leptos::{prelude::*, task::spawn_local};
use leptos_use::UseTimeoutFnReturn;
use leptos_use::use_timeout_fn;
// use thaw::*;

#[component]
pub fn Challenge(
    id: u32,
    name: String,
    description: Option<String>,
    event_id: u32,
    category: Option<String>,
    #[prop(default = 3)] difficulty: i8,
    points: u32,
    #[prop(optional)] attachments: Vec<Attachment>,
    solved_challenges: RwSignal<Vec<u32>>
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
    let solved_challenges = solved_challenges.get();
    let button_classes = Memo::new(move |_| {
        let base = "border-2 border-black p-2 text-black rounded";
        if solved.get() {
            format!("{base} {}", "bg-green-500")
        } else if incorrect.get() {
            format!("{base} {}", "bg-red-500")
        } else {
            format!(
                "{base} {}",
                "bg-lavender-blush-100 hover:bg-lavender-blush-200"
            )
        }
    });

    let UseTimeoutFnReturn { start, stop, .. } =
        use_timeout_fn(move |_: ()| {
            // runs after the delay on the client
            incorrect.set(false);
        }, 3000.0);

    let open = RwSignal::new(false);

    if solved_challenges.contains(&id) {
        solved.set(true);
    } 

    view! {
        <div
            class="bg-yale-blue-50 hover:bg-yale-blue-100 rounded-2xl p-4 content-center"
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
            }</p>
            {
                if needs_truncate {
                    view! {
                        <button
                            class="ml-2 text-sm underline text-blue-600"
                            on:click=move |_| {
                                desc_expanded.set(!desc_expanded.get());
                            }
                        >
                            { move || if desc_expanded.get() { "Show Less" } else { "Show More" } }
                        </button>
                    }.into_any()
                } else {
                    view! { <></> }.into_any()
                }
            }
            <Difficulty rating=difficulty />
            <b>{format!("Points: {points}")}</b>
            <br />
            <label for="flag">
                <b>"Flag: "</b>
                <input
                    class="border-black border-1 m-1"
                    on:input=move |ev| {
                        let val = event_target_value(&ev);
                        flag.set(val);
                    }
                />
            </label>
            <button
                class=move || button_classes.get()
                disabled=move || solved.get() || incorrect.get()
                on:click=move |_| {
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
                "Submit"
            </button>
            // <button
            //     class=button_classes
            //     //loading=loading
            //     disabled=solved
            //     on:click=move |_| {
            //         loading.set(true);
            //         if input_flag.get() == "1" {
            //             loading.set(false);
            //             solved.set(true);
            //             println!("{}", solved.get())
            //         }
            //     }
            // >
            //     {move || { if solved.get() { "Solved" } else { "Submit" } }}
            // </button>

            <For
                each=move || attachments.clone()
                key=|a: &Attachment| a.file_name.clone()
                let(a)
            >
                <div>{a.file_name}</div>
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
                <b>"Difficulty: "</b>
                {"⭐".repeat(rating as usize)}
            </span>
        </div>
    }
}
