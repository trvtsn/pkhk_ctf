use std::collections::HashMap;

use leptos::prelude::*;
use leptos::{web_sys::{FormData, HtmlInputElement, Event, HtmlSelectElement}, wasm_bindgen::JsCast, task::spawn_local};
use leptos_use::{UseTimeoutFnReturn, use_timeout_fn};

use crate::server::db::structs::AttachmentWithoutBlob;
use crate::server::enums::ResultStatus;
use crate::server::structs::ApiResult;
use crate::{components::admin::challenge::Challenge, server::{admin::{upload_files}, db, get_all_challenges_with_attachments}};

#[derive(Debug, Clone, PartialEq)]
pub enum Actions {
    Create,
    Delete,
    Edit,
    None
}

/// Default Home Page
#[component]
pub fn Challenges() -> impl IntoView {
    let section = RwSignal::new(Actions::None);
    let category_add_new_selected = RwSignal::new(false);
    
    let event_id = RwSignal::new("".to_string());
    let name = RwSignal::new("".to_string());
    let description = RwSignal::new("".to_string());
    let category = RwSignal::new("".to_string());
    let difficulty = RwSignal::new(0_i8);
    let points = RwSignal::new(0_u32);
    let flag = RwSignal::new("".to_string());
    let attachments = RwSignal::<Option<Vec<AttachmentWithoutBlob>>>::new(None);

    let refresh = RwSignal::new(0);

    // load once on mount
    let cwa = Resource::new(move || refresh.get(), move |_| async move {
        get_all_challenges_with_attachments().await.unwrap_or_default()
    });

    let categories_signal = RwSignal::<Vec<String>>::new(vec![]);
    let events_signal = RwSignal::<Vec<db::structs::Event>>::new(vec![]);

    let upload_action = Action::new_local(|data: &FormData| {
        upload_files(data.clone().into())
    });

    Effect::new(move |_| {
        if let Some(Ok(api_result)) = upload_action.value().get() {
            attachments.set(Some(api_result.details.clone()));
        }
    });

    let created = RwSignal::new(false);
    let creating = RwSignal::new(false);
    let create_submit_btn_text = Memo::new(move |_| {
        if created.get() { "Created!".to_string() } else { "Create".to_string() }
    });
    // let delete_submit_btn_text = Memo::new(move |_| {
    //     if deleted.get() { "Deleted!".to_string() } else { "Delete".to_string() }
    // });
    // let edit_submit_btn_text = Memo::new(move |_| {
    //     if edited.get() { "Edited!".to_string() } else { "Edit".to_string() }
    // });

    let UseTimeoutFnReturn { start, stop, .. } =
        use_timeout_fn(move |action: RwSignal<bool>| {
            // runs after the delay on the client
            action.set(false);
        }, 3000.0);

    view! {
        <div class="flex gap-2 mb-4">
            <button class="border border-gray-300 px-3 py-1 rounded-md text-sm hover:bg-gray-50" on:click=move |_| {
                if creating.get() {
                    creating.set(false);
                    section.set(Actions::None);
                } else {
                    creating.set(true);
                    section.set(Actions::Create);
                }
            }>"Create"</button>
            // <button class="border border-gray-300 px-3 py-1 rounded-md text-sm hover:bg-gray-50" on:click=move |_| section.set(Actions::Delete)>"Delete"</button>
            // <button class="border border-gray-300 px-3 py-1 rounded-md text-sm hover:bg-gray-50" on:click=move |_| section.set(Actions::Edit)>"Edit"</button>
        </div>

        <div class="flex flex-col gap-4">
            <Show when=move || section.get() == Actions::Create>
                <label class="block text-sm font-medium text-gray-700 mb-1">"Event"</label>
                <select class="w-full rounded-md border border-gray-300 bg-white px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" name="event_id" on:change=move |ev: Event| {
                    let value = event_target_value(&ev);
                    event_id.set(value);
                }>
                    <option value="">"-- Select Event --"</option>
                        <For
                            each=move || events_signal.get()
                            key=|e: &db::structs::Event| e.id.clone()
                            let(e: db::structs::Event)
                        >
                            {
                                let e1 = e.clone();
                                let e2 = e.clone();
                                let e3 = e.clone();
                                view! {
                                    <option value={move || e1.id.clone()}>{move || e2.name.clone()} " (ID: " {move || e3.id.clone()} ")"</option>
                                }
                            }
                        </For>
                </select>
                
                <label class="block text-sm font-medium text-gray-700 mb-1">"Name"</label>
                <input class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" name="name" on:change=move |ev: Event| {
                    let value = event_target_value(&ev);
                    name.set(value);
                }/>

                <label class="block text-sm font-medium text-gray-700 mb-1">"Description"</label>
                <input class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" name="description" on:change=move |ev: Event| {
                    let value = event_target_value(&ev);
                    description.set(value);
                }/>

                <label class="block text-sm font-medium text-gray-700 mb-1">"Category"</label>
                <select class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" name="category" on:change=move |ev: Event| {
                    let sel = ev.target().unwrap().unchecked_into::<HtmlSelectElement>();
                    let doc = leptos::web_sys::window().unwrap().document().unwrap();
                    let new_input = doc
                        .get_element_by_id("action_create_category_input")
                        .unwrap()
                        .unchecked_into::<HtmlInputElement>();

                    if sel.value() == "__new__" {
                        let _ = sel.remove_attribute("name");
                        let _ = new_input.set_attribute("name", "category");
                        category_add_new_selected.set(true);
                    } else {
                        let _ = sel.set_attribute("name", "category");
                        let _ = new_input.remove_attribute("name");
                        category_add_new_selected.set(false);
                    }

                    category.set(sel.value())
                }>
                    <option value="">"-- Select Category --"</option>
                    <For
                        each=move || categories_signal.get()
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
                <input class="mt-2 w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" hidden=move || !category_add_new_selected.get() type="text" id="action_create_category_input" on:change=move |ev: Event| {
                    let value = event_target_value(&ev);
                    category.set(value)
                }/>
                
                <label class="block text-sm font-medium text-gray-700 mb-1">"Difficulty"</label>
                <input class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" type="number" min="1" max="5" name="difficulty" on:change=move |ev: Event| {
                    let value = event_target_value(&ev);
                    difficulty.set(value.parse::<i8>().unwrap_or_default());
                }/>
                
                <label class="block text-sm font-medium text-gray-700 mb-1">"Points"</label>
                    <input class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" type="number" min="0" name="points" on:change=move |ev: Event| {
                        let value = event_target_value(&ev);
                        points.set(value.parse::<u32>().unwrap_or_default());
                    }/>
                <label class="block text-sm font-medium text-gray-700 mb-1">"Flag"</label>
                    <input class="w-full rounded-md border border-gray-300 px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" name="flag" on:change=move |ev: Event| {
                        let value = event_target_value(&ev);
                        flag.set(value);
                    }/>
                <label class="block text-sm font-medium text-gray-700 mb-1">"Attachment (Max 16 MiB)"</label>
                <input class="w-full text-sm" type="file" name="attachment" multiple
                    on:change=move |ev: Event| {
                        let input = ev.target().unwrap().unchecked_into::<HtmlInputElement>();
                        if let Some(files) = input.files() && files.length() > 0 {
                            let file = files.get(0).unwrap();
                            let fd = FormData::new().unwrap();
                            fd.append_with_blob_and_filename("file", &file, &file.name()).unwrap();
                            upload_action.dispatch_local(fd);
                        }
                    }
                />
                <p>
                    { move || {
                        if upload_action.pending().get() {
                            "Uploading...".to_string()
                        // } else if let Some(Ok(val)) = upload_action.value().get() {
                        //     format!("Uploaded: {}", val.details.file_name)
                        // } else {
                        } else {
                            "".to_string()
                        }
                    }}
                </p>
                <div class="flex gap-3 mt-2">
                    <button type="button" class="px-4 py-2 rounded-md border border-gray-300 text-sm hover:bg-gray-50">"Cancel"</button>
                    <button
                        class="ml-auto inline-flex items-center px-4 py-2 rounded-md bg-indigo-600 text-white text-sm font-semibold shadow-sm hover:bg-indigo-500 focus:outline-none focus:ring-2 focus:ring-indigo-500"
                        on:click=move |_| {
                            // let start = start.clone();
                            // let stop = stop.clone();
                            let event_id = event_id.get().clone();
                            let name = name.get().clone();
                            let description = description.get().clone();
                            let category = category.get().clone();
                            let difficulty = difficulty.get();
                            let points = points.get();
                            let flag = flag.get();
                            let attachments = attachments.get().clone();
                            spawn_local(async move {
                                if let Ok(ApiResult { result, .. }) = crate::server::admin::challenge(
                                    crate::server::admin::ChallengeAction::Create { event_id, name, description, category, difficulty, points, flag, attachments } 
                                ).await && result == ResultStatus::Success {
                                    created.set(true);
                                    refresh.update(|n| *n += 1);
                                    // stop();
                                    // start(created);
                                }
                            });
                        }
                    >
                        { move || create_submit_btn_text.get() }
                    </button>
                </div>
            </Show>
        </div>

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
                                        key=|challenge: &db::structs::ChallengeWithAttachments| challenge.challenge.id.clone()
                                        let(challenge)
                                    >
                                        <div class="challenge p-2">
                                            <Challenge cwa=RwSignal::new(challenge) refresh categories=categories_signal events=events_signal/>
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
