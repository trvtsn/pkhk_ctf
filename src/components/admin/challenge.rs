use crate::server::{CheckFlag, admin::{AdminChallengeApi, challenge}, check_flag, db::structs::{Attachment, Challenge}, enums::ResultStatus, structs::ApiResult};
use leptos::{prelude::*, task::spawn_local};
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

    let attachment_filename = RwSignal::<String>::new("".to_string());
    // let editing = RwSignal::new(false);
    // let deleted = RwSignal::new(false);
    // let challenge_action = ServerAction::<AdminChallengeApi>::new();

    view! {
        <div
            class="bg-yale-blue-50 hover:bg-yale-blue-100 rounded-2xl p-4 content-center"
            // on:click=move |_| {
                // open.set(true);
            // }
        >
            // <Show when=move || !editing.get() && !deleted.get()>
                <h3 class="text-3xl/8">{name.clone()}</h3>
                <h3 class="text-lg/8">"ID: "{id}</h3>
                <h3 class="text-lg/8">"Event ID: "{event_id}</h3>
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
                </p>

                <Difficulty rating=difficulty />
                <b>{format!("Points: {points}")}</b>
                <br />

                <For
                    each=move || attachments.clone()
                    key=|a: &Attachment| a.file_name.clone()
                    let(a)
                >
                    {attachment_filename.set(format!("/file/{}", a.file_name))}
                    <a href=move || attachment_filename.get()>{a.file_name}</a>
                </For>
            // </Show>

            // <Show when=move || editing.get()>
            //     <ActionForm action=challenge_action>
            //         <input class="bg-white border" type="hidden" name="action[edit][id]"></input>
            //         <label>
            //             <b>"Event ID"</b>
            //             <input class="bg-white border" type="number" name="action[edit][event_id]"></input>
            //         </label>
            //         <label>
            //             <b>"Name"</b>
            //             <input class="bg-white border" name="action[edit][name]"></input>
            //         </label>
            //         <label>
            //             <b>"Description"</b>
            //             <input class="bg-white border" name="action[edit][description]"></input>
            //         </label>
            //         <label>
            //             <b>"Category"</b>
            //             <input class="bg-white border" name="action[edit][category]"></input>
            //         </label>
            //         <label>
            //             <b>"Difficulty"</b>
            //             <input class="bg-white border" type="number" name="action[edit][difficulty]"></input>
            //         </label>
            //         <label>
            //             <b>"Points"</b>
            //             <input class="bg-white border" type="number" name="action[edit][points]"></input>
            //         </label>
            //         <label>
            //             <b>"Flag"</b>
            //             <input class="bg-white border" name="action[edit][flag]"></input>
            //         </label>
            //         <label>
            //             <b>"Attachment"</b>
            //             <input class="bg-white border" type="file" name="attachment" />
            //         </label>
            //         //<button loading=loading on_click=move |_| { loading.set(true) }>
            //         <input
            //             type="submit"
            //             class=r#"flex w-full justify-center rounded-md bg-indigo-600 px-3 py-1.5 text-sm font-semibold
            //                 leading-6 text-white shadow-sm hover:bg-indigo-500 focus-visible:outline 
            //                 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600"#
            //             value="Edit"
            //         />
            //     </ActionForm>
            //     <button 
            //         class="border-2 border-black p-2 text-black rounded" 
            //         on:click=move |_| {
            //             spawn_local(async move {
            //                 editing.set(false);
            //             });
            //         }
            //     >"Cancel"</button>
            // </Show>

            // <button 
            //     class="border-2 border-black p-2 text-black rounded" 
            //     on:click=move |_| {
            //         spawn_local(async move {
            //             editing.set(true);
            //         });
            //     }
            // >"Edit"</button>

            // <button 
            //     class="border-2 border-black p-2 text-black rounded" 
            //     on:click=move |_| {
            //         spawn_local(async move {
            //             deleted.set(true);
            //         });
            //     }
            // >"Delete"</button>
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
