use crate::server::db::structs::Attachment;
use leptos::prelude::*;
// use thaw::*;

#[component]
pub fn Challenge(
    title: String,
    description: Option<String>,
    #[prop(default = 3)] difficulty: i8,
    points: u32,
    #[prop(optional)] attachments: Vec<Attachment>,
) -> impl IntoView {
    let challenge_action = ServerAction::<crate::server::Challenge>::new();
    // let input_flag = RwSignal::new("".to_string());
    // let solved = RwSignal::new(false);
    // let loading = RwSignal::new(false);
    //
    // let button_classes = Memo::new(move |_| {
    //     let base = "border-2 border-black p-2 text-black rounded";
    //     if solved.get() {
    //         format!("{base} {}", "bg-green-500")
    //     } else {
    //         format!(
    //             "{base} {}",
    //             "bg-lavender-blush-100 hover:bg-lavender-blush-200"
    //         )
    //     }
    // });

    let open = RwSignal::new(false);

    view! {
        <div
            class="bg-yale-blue-50 hover:bg-yale-blue-100 rounded-2xl p-4 content-center"
            on:click=move |_| {
                open.set(true);
            }
        >
            <h3 class="text-3xl/8">{title}</h3>
            <p class="text-lg/8">{description}</p>
            <Difficulty rating=difficulty />
            <b>{format!("Points: {points}")}</b>
            <br />
            <label for="flag">
                <b>"Flag: "</b>
            </label>
            <ActionForm action=challenge_action>
                <input
                    name="action[Check][flag]"
                    class="bg-white border"
                />

                <input
                    type="submit"
                    class="bg-white border"
                    value="Submit"
                />
            </ActionForm>
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
