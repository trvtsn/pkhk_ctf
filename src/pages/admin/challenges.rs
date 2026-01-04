use std::collections::HashMap;

// use super::AdminNavBar;
// use crate::components::navbar::NavBar;
use leptos::prelude::*;
use leptos::{web_sys::{FormData, HtmlInputElement, Event, HtmlSelectElement}, wasm_bindgen::JsCast};

use crate::{components::admin::challenge::Challenge, server::{admin::{upload_file, AdminChallengeApi, get_all_challenge_categories, get_all_events}, db::{self, structs::{Attachment, ChallengeWithAttachments}}, get_all_challenges_with_attachments}};

#[derive(Debug, Clone, PartialEq)]
pub enum Actions {
    Create,
    Delete,
    Edit
}

/// Default Home Page
#[component]
pub fn Challenges() -> impl IntoView {
    let section = RwSignal::new(Actions::Create);
    let challenge_action = ServerAction::<AdminChallengeApi>::new();
    let category_add_new_selected = RwSignal::new(false);

    // load once on mount
    let cwa = Resource::new(move || (), move |_| async move {
        get_all_challenges_with_attachments().await.unwrap_or_default()
    });

    let categories = Resource::new(move || (), move |_| async move {
        get_all_challenge_categories().await.unwrap_or_default()
    });

    let events_resource = Resource::new(move || (), move |_| async move {
        get_all_events().await.unwrap_or_default()
    });

    let upload_action = Action::new_local(|data: &FormData| {
        upload_file(data.clone().into())
    });

    let attachment_filename = RwSignal::new("".to_string());

    Effect::new(move |_| {
        if let Some(Ok(api_result)) = upload_action.value().get() {
            attachment_filename.set(api_result.details.clone());
        }
    });

    view! {
        <button class="border-2 border-black p-2 text-black rounded" on:click=move |_| section.set(Actions::Create)>"Create"</button>
        <button class="border-2 border-black p-2 text-black rounded" on:click=move |_| section.set(Actions::Delete)>"Delete"</button>
        <button class="border-2 border-black p-2 text-black rounded" on:click=move |_| section.set(Actions::Edit)>"Edit"</button>

        <section class="main-panel">
            <Show when=move || section.get() == Actions::Create>
                <ActionForm action=challenge_action>
                    <label>
                        <b>"Event"</b>
                        <select class="bg-white border" name="action[create][event_id]">
                            <option value="">"-- Select Event --"</option>
                                <For
                                    each=move || events_resource.get().unwrap_or_default()
                                    key=|e: &db::structs::Event| e.id
                                    let(e: db::structs::Event)
                                >
                                    {
                                        let e1 = e.clone();
                                        let e2 = e.clone();
                                        let e3 = e.clone();
                                        view! {
                                            <option value={move || e1.id}>{move || e2.name.clone()} " (ID: " {move || e3.id} ")"</option>
                                        }
                                    }
                                </For>
                        </select>
                    </label>
                    <label>
                        <b>"Name"</b>
                        <input class="bg-white border" name="action[create][name]" />
                    </label>
                    <label>
                        <b>"Description"</b>
                        <input class="bg-white border" name="action[create][description]" />
                    </label>
                    <label>
                        <b>"Category"</b>
                        <select class="bg-white border" name="action[create][category]" on:change=move |ev: Event| {
                            let sel = ev.target().unwrap().unchecked_into::<HtmlSelectElement>();
                            let doc = leptos::web_sys::window().unwrap().document().unwrap();
                            let new_input = doc
                                .get_element_by_id("action_create_category_input")
                                .unwrap()
                                .unchecked_into::<HtmlInputElement>();

                            if sel.value() == "__new__" {
                                let _ = sel.remove_attribute("name");
                                let _ = new_input.set_attribute("name", "action[create][category]");
                                category_add_new_selected.set(true);
                            } else {
                                let _ = sel.set_attribute("name", "action[create][category]");
                                let _ = new_input.remove_attribute("name");
                                category_add_new_selected.set(false);
                            }
                        }>
                            <option value="">"-- Select Category --"</option>
                            <For
                                each=move || categories.get().unwrap_or_default()
                                key=|category: &String| category.clone()
                                let(category)
                            >
                                {move || {
                                    let c = category.clone();
                                    view! {
                                        <option value={c.clone()}>{c.clone()}</option>
                                    }
                                }}
                            </For>
                            <option value="__new__">"-- Add New --"</option>
                        </select>
                        <input class="bg-white border" hidden=move || !category_add_new_selected.get() type="text" id="action_create_category_input" />
                    </label>
                    <label>
                        <b>"Difficulty"</b>
                        <input class="bg-white border" type="number" name="action[create][difficulty]" />
                    </label>
                    <label>
                        <b>"Points"</b>
                        <input class="bg-white border" type="number" name="action[create][points]" />
                    </label>
                    <label>
                        <b>"Flag"</b>
                        <input class="bg-white border" name="action[create][flag]" />
                    </label>
                    <label>
                        <b>"Attachment (Max 16 MiB)"</b>
                        <input class="bg-white border" type="file" name="file"
                            on:change=move |ev: Event| {
                                let input = ev.target().unwrap().unchecked_into::<HtmlInputElement>();
                                if let Some(files) = input.files() {
                                    if files.length() > 0 {
                                        let file = files.get(0).unwrap();
                                        let fd = FormData::new().unwrap();
                                        fd.append_with_blob_and_filename("file", &file, &file.name()).unwrap();
                                        upload_action.dispatch_local(fd);
                                    }
                                }
                            }
                        />
                        <p>
                            { move || {
                                if upload_action.pending().get() {
                                    "Uploading...".to_string()
                                } else if let Some(Ok(val)) = upload_action.value().get() {
                                    format!("Uploaded: {}", val.details)
                                } else {
                                    "Choose a file".to_string()
                                }
                            }}
                        </p>
                    </label>
                    <input
                        type="hidden"
                        name="action[create][attachment]"
                        value={ move || attachment_filename.get() }
                    />
                    <input
                        type="submit"
                        class=r#"flex w-full justify-center rounded-md bg-indigo-600 px-3 py-1.5 text-sm font-semibold
                               leading-6 text-white shadow-sm hover:bg-indigo-500 focus-visible:outline 
                               focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600"#
                        value="Create"
                    />
                </ActionForm>
            </Show>

            <Show when=move || section.get() == Actions::Delete>
                "Delete"
                <ActionForm action=challenge_action>
                    <label>
                        <b>"Challenge ID"</b>
                        <input class="bg-white border" type="number" name="action[delete][id]" />
                    </label>
                    <input
                        type="submit"
                        class=r#"flex w-full justify-center rounded-md bg-indigo-600 px-3 py-1.5 text-sm font-semibold
                            leading-6 text-white shadow-sm hover:bg-indigo-500 focus-visible:outline 
                            focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600"#
                        value="Delete"
                    />
                </ActionForm>
                // <Suspense fallback=move || view! { <div>"Loading..."</div> }>
                //     {move || {
                //         let challenges = cwa.get().map(move |result| match result {
                //             Ok(challenges) => {
                //                 view! {
                //                     <For
                //                         // returns an owned Vec for For to iterate
                //                         // each=move || bar.get().unwrap().clone()
                //                         each=move || challenges.clone()
                //                         key=|challenge: &db::structs::ChallengeWithAttachments| challenge.challenge.id
                //                         let(challenge)
                //                         // key=|foobar: &String| foobar.clone()
                //                         // let(foobar)
                //                     >
                //                         <div class="challenge p-2">
                //                             // <p>{foobar}</p>
                //                             <Challenge
                //                                 title=challenge.challenge.name
                //                                 description=challenge.challenge.description
                //                                 difficulty=challenge.challenge.difficulty
                //                                 points=challenge.challenge.points
                //                                 attachments=challenge.attachments
                //                             />
                //                             <ActionForm action=challenge_action>
                //                                 <label>
                //                                     <b>"Challenge ID"</b>
                //                                     <input class="bg-white border" type="number" name="action[Delete][id]" />
                //                                 </label>
                //                                 //<button loading=loading on_click=move |_| { loading.set(true) }>
                //                                 <input
                //                                     type="submit"
                //                                     class=r#"flex w-full justify-center rounded-md bg-indigo-600 px-3 py-1.5 text-sm font-semibold
                //                                         leading-6 text-white shadow-sm hover:bg-indigo-500 focus-visible:outline 
                //                                         focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600"#
                //                                     value="Delete"
                //                                 />
                //                             </ActionForm>
                //                         </div>
                //                     </For>
                //                 }.into_any()
                //             }
                //             Err(e) => {
                //                 view! {
                //                     <div class="challenge p-2">
                //                         <p>"Bruh" {e.to_string()}</p>
                //                     </div>
                //                 }.into_any()
                //             }
                //         })
                //         .collect_view()
                //         .into_any();
                //
                //         view! {
                //             {challenges}
                //         }
                //     }}
                // </Suspense>
            </Show>

            <Show when=move || section.get() == Actions::Edit>
                "Edit"
                <ActionForm action=challenge_action>
                    <label>
                        <b>"Challenge ID"</b>
                        <input class="bg-white border" type="number" name="action[edit][id]" />
                    </label>
                    <label>
                        <b>"Event ID"</b>
                        <input class="bg-white border" type="number" name="action[edit][event_id]" />
                    </label>
                    <label>
                        <b>"Name"</b>
                        <input class="bg-white border" name="action[edit][name]" />
                    </label>
                    <label>
                        <b>"Description"</b>
                        <input class="bg-white border" name="action[edit][description]" />
                    </label>
                    <label>
                        <b>"Category"</b>
                        <select class="bg-white border" name="action[edit][category]" on:change=move |ev: Event| {
                            let sel = ev.target().unwrap().unchecked_into::<HtmlSelectElement>();
                            let doc = leptos::web_sys::window().unwrap().document().unwrap();
                            let new_input = doc
                                .get_element_by_id("action_edit_category_input")
                                .unwrap()
                                .unchecked_into::<HtmlInputElement>();

                            if sel.value() == "__new__" {
                                let _ = sel.remove_attribute("name");
                                let _ = new_input.set_attribute("name", "action[edit][category]");
                                category_add_new_selected.set(true);
                            } else {
                                let _ = sel.set_attribute("name", "action[edit][category]");
                                let _ = new_input.remove_attribute("name");
                                category_add_new_selected.set(false);
                            }
                        }>
                            <option value="">"-- Select Category --"</option>
                            <For
                                each=move || categories.get().unwrap_or_default()
                                key=|category: &String| category.clone()
                                let(category)
                            >
                                {move || {
                                    let c = category.clone();
                                    view! {
                                        <option value={c.clone()}>{c.clone()}</option>
                                    }
                                }}
                            </For>
                            <option value="__new__">"-- Add New --"</option>
                        </select>
                        <input class="bg-white border" hidden=move || !category_add_new_selected.get() type="text" id="action_edit_category_input" />
                    </label>
                    <label>
                        <b>"Difficulty"</b>
                        <input class="bg-white border" type="number" name="action[edit][difficulty]" />
                    </label>
                    <label>
                        <b>"Points"</b>
                        <input class="bg-white border" type="number" name="action[edit][points]" />
                    </label>
                    <label>
                        <b>"Flag"</b>
                        <input class="bg-white border" name="action[edit][flag]" />
                    </label>
                    <label>
                        <b>"Attachment (Max 16 MiB)"</b>
                        <input class="bg-white border" type="file" name="file"
                            on:change=move |ev: Event| {
                                let input = ev.target().unwrap().unchecked_into::<HtmlInputElement>();
                                if let Some(files) = input.files() {
                                    if files.length() > 0 {
                                        let file = files.get(0).unwrap();
                                        let fd = FormData::new().unwrap();
                                        fd.append_with_blob_and_filename("file", &file, &file.name()).unwrap();
                                        upload_action.dispatch_local(fd);
                                    }
                                }
                            }
                        />
                        <p>
                            { move || {
                                if upload_action.pending().get() {
                                    "Uploading...".to_string()
                                } else if let Some(Ok(val)) = upload_action.value().get() {
                                    format!("Uploaded: {}", val.details)
                                } else {
                                    "".to_string()
                                }
                            }}
                        </p>
                    </label>
                    //<button loading=loading on_click=move |_| { loading.set(true) }>
                    <input
                        type="submit"
                        class=r#"flex w-full justify-center rounded-md bg-indigo-600 px-3 py-1.5 text-sm font-semibold
                            leading-6 text-white shadow-sm hover:bg-indigo-500 focus-visible:outline 
                            focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-indigo-600"#
                        value="Edit"
                    />
                </ActionForm>
                // <Suspense fallback=move || view! { <div>"Loading..."</div> }>
                //     {move || {
                //         let challenges = cwa.get().map(move |result| match result {
                //             Ok(challenges) => {
                //                 view! {
                //                     <For
                //                         each=move || challenges.clone()
                //                         key=|challenge: &db::structs::ChallengeWithAttachments| challenge.challenge.id
                //                         let(challenge)
                //                     >
                //                         <div class="challenge p-2">
                //                             // <p>{foobar}</p>
                //                             <Challenge
                //                                 title=challenge.challenge.name
                //                                 description=challenge.challenge.description
                //                                 difficulty=challenge.challenge.difficulty
                //                                 points=challenge.challenge.points
                //                                 attachments=challenge.attachments
                //                             />
                //                         </div>
                //                     </For>
                //                 }.into_any()
                //             }
                //             Err(e) => {
                //                 view! {
                //                     <div class="challenge p-2">
                //                         <p>"Bruh" {e.to_string()}</p>
                //                     </div>
                //                 }.into_any()
                //             }
                //         })
                //         .collect_view()
                //         .into_any();
                // 
                //         view! {
                //             {challenges}
                //         }
                //     }}
                // </Suspense>
            </Show>
        </section>

        <div class="challenges">
            <Suspense fallback=move || view! { <div>"Loading..."</div> }>
                {move || {
                    let mut map = HashMap::<Option<String>, Vec<db::structs::ChallengeWithAttachments>>::new();
                    for ch in cwa.get().unwrap_or_default().into_iter() {
                        map.entry(ch.challenge.category.clone()).or_default().push(ch);
                    }

                    let mut groups = map.into_iter().collect::<Vec<(Option<String>, Vec<db::structs::ChallengeWithAttachments>)>>();

                    // alphabetical sort, there's probably a better way to do this
                    groups.sort_by(|(a, _), (b, _)| a.as_deref().unwrap_or("").cmp(b.as_deref().unwrap_or("")));

                    view! {
                        <For
                            each=move || groups.clone()
                            key=|group: &(Option<String>, Vec<db::structs::ChallengeWithAttachments>)| group.0.clone()
                            let(group)
                        >
                            <div class="challenge-category p-2">
                                <h2 class="text-2xl">
                                    { move || group.0.clone().unwrap_or_else(|| "Uncategorized".to_string()) }
                                </h2>

                                <div class="m-4 grid grid-cols-4 content-stretch">
                                    <For
                                        each=move || group.1.clone()
                                        key=|challenge: &db::structs::ChallengeWithAttachments| challenge.challenge.id
                                        let(challenge)
                                    >
                                        <div class="challenge p-2">
                                            <Challenge
                                                id=challenge.challenge.id
                                                name=challenge.challenge.name.clone()
                                                description=challenge.challenge.description.clone()
                                                event_id=challenge.challenge.event_id
                                                category=challenge.challenge.category.clone()
                                                difficulty=challenge.challenge.difficulty
                                                points=challenge.challenge.points
                                                attachments=challenge.attachments.clone()
                                            />
                                        </div>
                                    </For>
                                </div>
                            </div>
                        </For>
                    }
                }}
            </Suspense>
        </div>
    }
}
