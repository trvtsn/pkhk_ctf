use std::collections::HashMap;

// use super::AdminNavBar;
// use crate::components::navbar::NavBar;
use leptos::prelude::*;

use crate::{components::admin::challenge::Challenge, server::{admin::{AdminChallengeApi, get_all_attachment_filenames, get_all_events}, db, get_all_challenges_with_attachments}};

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
        match get_all_challenges_with_attachments().await {
            Ok(cwa) => Ok(cwa),
            Err(e) => Err(e)
        }
    });

    let attachment_filenames = Resource::new(move || (), move |_| async move {
        match get_all_attachment_filenames().await {
            Ok(filenames) => Ok(filenames),
            Err(e) => Err(e)
        }
    });

    let events_resource = Resource::new(move || (), move |_| async move {
        match get_all_events().await {
            Ok(events) => Ok(events),
            Err(e) => Err(e)
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
                        <b>"Event ID"</b>
                        <input class="bg-white border" type="number" name="action[create][event_id]" />
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
                        // <select>
                        //     <For
                        //         each=move || {
                        //             cwa.get().map(|cat| match cat {
                        //                 Ok(cat) => {},
                        //                 Err(e) => {}
                        //             })
                        //         }
                        //         key=|ch_cat: &String| ch_cat
                        //         let(ch_cat)
                        //     >
                        //         <option name="action[Create][category]">{cwa}</option>
                        //     </For>
                        //     <option>"+ Add New"</option>
                        // </select>
                        // <Show when=move || category_add_new_selected.get()>
                            <input class="bg-white border" name="action[create][category]" />
                        // </Show>
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
                        <b>"Attachment"</b>
                        // <select>
                        //     {
                        //         let attachment_filenames = attachment_filenames.get().map(|f| match f {
                        //             Ok(f) => f,
                        //             Err(e) => Vec::<String>::default()
                        //         });

                        //         view! {
                        //             <For
                        //                 each=move || attachment_filenames.unwrap_or_default()
                        //                 key=|f: &String| f.clone()
                        //                 let(f)
                        //             >
                        //                 <option value={f}>{f}</option>
                        //             </For>
                        //         }
                        //     }
                        // </select>
                        <input class="bg-white border" type="file" name="attachment" />
                    </label>
                    //<button loading=loading on_click=move |_| { loading.set(true) }>
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
                    //<button loading=loading on_click=move |_| { loading.set(true) }>
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
                        <input class="bg-white border" name="action[edit][category]" />
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
                        <b>"Attachment"</b>
                        <input class="bg-white border" type="file" name="attachment" />
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
                    let challenges = cwa.get().map(move |result| match result {
                        Ok(challenges) => {
                            let mut map = HashMap::<Option<String>, Vec<db::structs::ChallengeWithAttachments>>::new();
                            for ch in challenges.into_iter() {
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
                            }.into_any()
                        }
                        Err(e) => {
                            view! {
                                <div class="challenge p-2">
                                    <p>"Bruh" {e.to_string()}</p>
                                </div>
                            }.into_any()
                        }
                    })
                    .collect_view()
                    .into_any();

                    view! {
                        {challenges}
                    }
                }}
            </Suspense>
        </div>
    }
}
