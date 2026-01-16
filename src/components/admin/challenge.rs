use crate::components::utils::TruncatedDesc;
use crate::server::{admin::{upload_files}, db::{self, structs::{AttachmentWithoutBlob, Challenge, ChallengeWithAttachments}}, enums::ResultStatus, structs::ApiResult};
use leptos::{prelude::*, task::spawn_local, wasm_bindgen::JsCast, web_sys::{Event, FormData, HtmlInputElement, HtmlSelectElement}};

#[component]
pub fn Challenge(
    cwa: ChallengeWithAttachments,
    refresh: RwSignal<i32>,
    categories: RwSignal<Vec<String>>,
    events: RwSignal<Vec<db::structs::Event>>
) -> impl IntoView {
    let ChallengeWithAttachments { challenge, attachments } = cwa;
    let Challenge { id, event_id, name, description, category, difficulty, points } = challenge;

    let id_signal = RwSignal::new(id);
    let event_id_signal = RwSignal::new(event_id.clone());
    let name_signal = RwSignal::new(name.clone());
    let description_signal = RwSignal::new(description.clone());
    let category_signal = RwSignal::new(category.clone());
    let difficulty_signal = RwSignal::new(difficulty);
    let points_signal = RwSignal::new(points);
    let attachments_signal = RwSignal::new(attachments.clone());

    let event_id_edit = RwSignal::new(event_id);
    let name_edit = RwSignal::new(name.clone());
    let description_edit = RwSignal::new(description.clone());
    let category_edit = RwSignal::new(category.clone());
    let difficulty_edit = RwSignal::new(difficulty);
    let points_edit = RwSignal::new(points);
    let attachments_edit = RwSignal::new(attachments.clone());
    let flag_edit = RwSignal::new("".to_string());

    let category_add_new_selected = RwSignal::new(false);
    let editing = RwSignal::new(false);
    let deleting = RwSignal::new(false);
    let deleted = RwSignal::new(false);

    let upload_action = Action::new_local(|data: &FormData| {
        upload_files(data.clone().into())
    });

    Effect::new(move |_| {
        if let Some(Ok(api_result)) = upload_action.value().get() {
            attachments_signal.set(api_result.details.clone());
        }
    });

    let delete_submit_btn_text = Memo::new(move |_| {
        if deleting.get() { "Confirm Delete".to_string() } else { "Delete".to_string() }
    });
    let edit_submit_btn_text = Memo::new(move |_| {
        if editing.get() { "Confirm Edit".to_string() } else { "Edit".to_string() }
    });

    view! {
        <div class="bg-yale-blue-50 hover:bg-yale-blue-100 rounded-lg p-4 content-center">
            <Show when=move || !editing.get() && !deleted.get()>
                <h3 class="text-3xl/8 font-bold">{move || name_signal.get().clone()}</h3>
                <p class="text-lg/8"><b>"ID: "</b>{move || id_signal.get().clone()}</p>
                <p class="text-lg/8"><b>"Event ID: "</b>{move || event_id_signal.get().clone()}</p>
                <p class="text-lg/8">
                    <TruncatedDesc description_signal/>
                </p>
                <Difficulty rating=difficulty_signal.get() />
                <p class="text-lg/8"><b>"Points: "</b> {points_signal.get()}</p>
                <br />
                
                <For
                    each=move || attachments_signal.get().clone()
                    key=|a: &AttachmentWithoutBlob| a.id.clone()
                    let(a)
                >
                    <a 
                        download 
                        href=move || format!("/file/{}", a.id.clone()) 
                        class="underline text-blue-600"
                    >{ a.file_name.clone() }</a>
                </For>
            </Show>

            <Show when=move || editing.get()>
                <label class="block text-sm font-medium text-gray-700 mb-1">"Event"</label>
                <select 
                    class="w-full rounded-md border border-gray-300 bg-white px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" 
                    name="event_id" 
                    bind:value=event_id_edit
                >
                    <option value="">"-- Select Event --"</option>
                    <For
                        each=move || events.get()
                        key=|e: &crate::server::db::structs::Event| e.id.clone()
                        let(e: crate::server::db::structs::Event)
                    >
                        <option value={e.id.clone()}>{e.name.clone()} " (ID: " {e.id.clone()} ")"</option>
                    </For>
                </select>

                <label class="block text-sm font-medium text-gray-700 mb-1">"Name"</label>
                <input 
                    class="w-full rounded-md border border-gray-300 bg-white px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" 
                    name="name" 
                    value=move || name_signal.get()
                    bind:value=name_edit
                />

                <label class="block text-sm font-medium text-gray-700 mb-1">"Description"</label>
                <input 
                    class="w-full rounded-md border border-gray-300 bg-white px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" 
                    name="description" 
                    value=move || description_signal.get().unwrap_or_default() 
                    on:change=move |ev: Event| {
                        let value = event_target_value(&ev);
                        description_edit.set(Some(value));
                    }
                />

                <label class="block text-sm font-medium text-gray-700 mb-1">"Category"</label>
                <select 
                    class="w-full rounded-md border border-gray-300 bg-white px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" 
                    name="category" 
                    on:change=move |ev: Event| {
                        let sel = ev.target().unwrap().unchecked_into::<HtmlSelectElement>();
                        let doc = leptos::web_sys::window().unwrap().document().unwrap();
                        let new_input = doc
                            .get_element_by_id("action_edit_category_input")
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

                        category_edit.set(Some(sel.value()));
                    }
                >
                    <option value="">"-- Select Category --"</option>
                    <For
                        each=move || categories.get()
                        key=|category: &String| category.clone()
                        let(category)
                    >
                        <option value={category.clone()}>{category.clone()}</option>
                    </For>
                    <option value="__new__">"-- Add New --"</option>
                </select>
                <input 
                    class="w-full rounded-md border border-gray-300 bg-white px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" 
                    hidden=move || !category_add_new_selected.get() 
                    type="text" 
                    id="action_edit_category_input" 
                    on:change=move |ev: Event| {
                        let value = event_target_value(&ev);
                        category_edit.set(Some(value));
                    }
                />

                <label class="block text-sm font-medium text-gray-700 mb-1">"Difficulty"</label>
                <input 
                    class="w-full rounded-md border border-gray-300 bg-white px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" 
                    type="number" 
                    name="difficulty" 
                    min=0 
                    max=5 
                    value=move || difficulty_signal.get() 
                    on:change=move |ev: Event| {
                        let value = event_target_value(&ev);
                        difficulty_edit.set(value.parse::<i8>().unwrap_or_default());
                    }
                />

                <label class="block text-sm font-medium text-gray-700 mb-1">"Points"</label>
                <input 
                    class="w-full rounded-md border border-gray-300 bg-white px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500" 
                    type="number" 
                    name="points" 
                    value=move || points_signal.get() 
                    on:change=move |ev: Event| {
                        let value = event_target_value(&ev);
                        points_edit.set(value.parse::<u32>().unwrap_or_default());
                    }
                />

                <label class="block text-sm font-medium text-gray-700 mb-1">"Flag"</label>
                <input 
                    class="w-full rounded-md border border-gray-300 bg-white px-3 py-2 text-sm focus:outline-none focus:ring-2 focus:ring-indigo-500"
                    name="flag" 
                    bind:value=flag_edit
                />

                <label class="block text-sm font-medium text-gray-700 mb-1">"Attachment"</label>
                <input class="w-full text-sm" type="file" name="attachment" multiple on:change=move |ev: Event| {
                    let input = ev.target().unwrap().unchecked_into::<HtmlInputElement>();
                    if let Some(files) = input.files() && files.length() > 0 {
                        let file = files.get(0).unwrap();
                        let fd = FormData::new().unwrap();
                        fd.append_with_blob_and_filename("file", &file, &file.name()).unwrap();
                        upload_action.dispatch_local(fd);
                    }
                }/>
            </Show>

            <div class="flex flex-row-reverse gap-3 mt-2">
                <Show when=move || editing.get() || deleting.get()>
                    <button 
                        class="px-4 py-2 rounded-md border border-gray-300 text-sm hover:bg-gray-50" 
                        on:click=move |_| {
                            spawn_local(async move {
                                editing.set(false);
                                deleting.set(false);
                            });
                        }
                    >"Cancel"</button>
                </Show>
                <button 
                    type="button" 
                    hidden=move || deleting.get() 
                    class=r#"inline-flex items-center gap-2 rounded-lg text-white px-4 py-2 text-sm 
                    font-medium focus:outline-none focus:ring-2 active:scale-95 transition bg-indigo-600 
                    hover:bg-indigo-700 focus:ring-indigo-400"#
                    on:click=move |_| {
                        let challenge_id = id_signal.get();
                        let event_id = event_id_edit.get();
                        let name = name_edit.get();
                        let description = description_edit.get();
                        let category = category_edit.get();
                        let difficulty = difficulty_edit.get();
                        let points = points_edit.get();
                        let flag = flag_edit.get();
                        let attachments = attachments_edit.get();
                        if editing.get() {
                            spawn_local(async move {
                                // update in db
                                if let Ok(ApiResult { result, .. }) = crate::server::admin::challenge(
                                    crate::server::admin::ChallengeAction::Edit { 
                                        id: challenge_id.clone(), 
                                        event_id: event_id.clone(), 
                                        name: name.clone(), 
                                        description: description.clone().unwrap_or_default(), 
                                        category: category.clone().unwrap_or_default(), 
                                        difficulty,
                                        points,
                                        flag: flag.clone(), 
                                        attachments: Some(attachments.clone()),
                                    }
                                ).await && result == ResultStatus::Success {
                                    refresh.update(|n| *n += 1);
                                    editing.set(false);

                                    event_id_signal.set(event_id);
                                    name_signal.set(name);
                                    description_signal.set(description);
                                    category_signal.set(category);
                                    difficulty_signal.set(difficulty);
                                    points_signal.set(points);
                                    attachments_signal.set(attachments);
                                }
                            });
                        } else {
                            editing.set(true)
                        }
                    }
                >{move || edit_submit_btn_text.get()}</button>
                <button
                    hidden=move || editing.get()
                    class=r#"ml-auto inline-flex items-center px-4 py-2 rounded-md bg-red-600 text-white text-sm 
                    font-semibold shadow-sm hover:bg-red-500 focus:outline-none focus:ring-2 focus:ring-indigo-500"#
                    on:click=move |_| {
                        if deleting.get() {
                            let challenge_id = id_signal.get().clone();
                            spawn_local(async move {
                                tracing::debug!("deleting challenge ID: {challenge_id}");
                                if let Ok(ApiResult { result, .. }) = crate::server::admin::challenge(
                                    crate::server::admin::ChallengeAction::Delete { id: challenge_id.clone() } 
                                ).await && result == ResultStatus::Success {
                                    deleting.set(false);
                                    deleted.set(true);
                                    refresh.update(|n| *n += 1);
                                }
                            });
                        } else {
                            deleting.set(true);
                        }
                    }
                >
                    {move || delete_submit_btn_text.get()}
                </button>
            </div>
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
