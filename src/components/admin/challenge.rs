use crate::components::utils::TruncatedDesc;
use crate::pages::admin::challenges::Hint;
use crate::server::admin::{get_all_user_groups, upload_illustration};
use crate::server::db::structs::DbHint;
use crate::server::proxmox::ProxmoxVMTemplate;
use crate::server::{admin::{upload_files}, db::{self, structs::{AttachmentWithoutBlob, Challenge, ChallengeWithAttachments}}, enums::ResultStatus, structs::ApiResult};
use icondata as i;
use leptos::{prelude::*, task::spawn_local, wasm_bindgen::JsCast, web_sys::{Event, FormData, HtmlInputElement, HtmlSelectElement, HtmlOptionElement}};
use leptos_icons::Icon;

#[component]
pub fn Challenge(
    cwa: ChallengeWithAttachments,
    refresh: RwSignal<i32>,
    categories: RwSignal<Vec<String>>,
    events: RwSignal<Vec<db::structs::Event>>,
    templates: RwSignal<Vec<ProxmoxVMTemplate>>,
    hints: RwSignal<Vec<DbHint>>
) -> impl IntoView {
    let ChallengeWithAttachments { challenge, attachments, illustration } = cwa;
    let Challenge { id, event_id, name, description, category, difficulty, points, visible_to_groups, vm_ids } = challenge;

    let id_signal = RwSignal::new(id);
    let event_id_signal = RwSignal::new(event_id.clone());
    let name_signal = RwSignal::new(name.clone());
    let description_signal = RwSignal::new(description.clone());
    let category_signal = RwSignal::new(category.clone());
    let difficulty_signal = RwSignal::new(difficulty);
    let points_signal = RwSignal::new(points);
    let attachments_signal = RwSignal::new(attachments.clone());
    let visible_to_groups_signal = RwSignal::new(visible_to_groups.clone());
    let illustration_signal = RwSignal::new(illustration.clone());

    let event_id_edit = RwSignal::new(event_id);
    let name_edit = RwSignal::new(name.clone());
    let description_edit = RwSignal::new(description.clone());
    let category_edit = RwSignal::new(category.clone());
    let difficulty_edit = RwSignal::new(difficulty);
    let points_edit = RwSignal::new(points);
    let attachments_edit = RwSignal::new(Some(attachments.clone()));
    let flag_edit = RwSignal::new("".to_string());
    let illustration_edit = RwSignal::new(None);
    let visible_to_groups_edit = RwSignal::new(visible_to_groups);
    let proxmox_vm_id_edit = RwSignal::new(vm_ids);
    let hints_edit = RwSignal::new(vec![]);

    let next_hint_id = RwSignal::new(1_usize);
    let category_add_new_selected = RwSignal::new(false);
    let editing = RwSignal::new(false);
    let deleting = RwSignal::new(false);
    let deleted = RwSignal::new(false);

    let groups_resource = Resource::new(move || refresh.get(), move |_| async move {
        get_all_user_groups().await.unwrap_or_default()
    });

    let file_upload_action = Action::new_local(|data: &FormData| {
        upload_files(data.clone().into())
    });

    let illustration_upload_action = Action::new_local(|data: &FormData| {
        upload_illustration(data.clone().into())
    });

    Effect::new(move |_| {
        if let Some(Ok(api_result)) = file_upload_action.value().get() {
            attachments_edit.set(Some(api_result.details.clone()));
        } else if let Some(Ok(api_result)) = illustration_upload_action.value().get() {
            illustration_edit.set(Some(api_result.details.clone()));
        }
    });

    let delete_submit_btn_text = Memo::new(move |_| {
        if deleting.get() { "Confirm Delete".to_string() } else { "Delete".to_string() }
    });
    let edit_submit_btn_text = Memo::new(move |_| {
        if editing.get() { "Confirm Edit".to_string() } else { "Edit".to_string() }
    });

    let uploading_file_text = Memo::new(move |_| {
        if file_upload_action.pending().get() {
            "Uploading...".to_string()
        // } else if let Some(Ok(val)) = upload_action.value().get() {
        //     format!("Uploaded: {}", val.details.file_name)
        // } else {
        } else {
            "".to_string()
        }
    });

    let uploading_illustration_text = Memo::new(move |_| {
        if illustration_upload_action.pending().get() {
            "Uploading...".to_string()
        // } else if let Some(Ok(val)) = upload_action.value().get() {
        //     format!("Uploaded: {}", val.details.file_name)
        // } else {
        } else {
            "".to_string()
        }
    });

    view! {
        <div class=r#"content-center p-4 rounded-lg bg-card hover:bg-card-hover text-text"#>
            <Show when=move || !editing.get() && !deleted.get()>
                <Transition fallback=move || {
                    view! { <div>"Loading..."</div> }
                }>
                    {move || {
                        if let Some(illustration_id) = illustration_signal.get() { 
                            view! {
                                <div class="h-48 w-48 flex justify-center m-auto">
                                    <img 
                                        src=move || format!("/image/{}", illustration_id) 
                                        class=r#"text-blue-600 underline object-cover shadow-sm"#
                                    />
                                </div>
                            }.into_any()
                        } else {
                            "".into_any()
                        }
                    }}
                </Transition>
                <h3 class=r#"font-bold text-3xl/8"#>{move || name_signal.get().clone()}</h3>
                <p class=r#"text-lg/8"#>
                    <b>"ID: "</b>
                    {move || id_signal.get().clone()}
                </p>
                <p class=r#"text-lg/8"#>
                    <b>"Event ID: "</b>
                    {move || event_id_signal.get().clone()}
                </p>
                <p class=r#"text-lg/8"#>
                    <TruncatedDesc description=description_signal />
                </p>
                <Difficulty rating=difficulty_signal.get() />
                <p class=r#"text-lg/8"#>
                    <b>"Points: "</b>
                    {points_signal.get()}
                </p>
                <p class=r#"text-lg/8"#>
                    <b>"Visible To Groups: "</b>
                    {visible_to_groups_signal.get().replace(",", ", ")}
                </p>
                <br />

                <For
                    each=move || attachments_signal.get().clone()
                    key=|a: &AttachmentWithoutBlob| a.id.clone()
                    let(a)
                >
                    <a
                        download
                        href=move || format!("/file/{}", a.id.clone())
                        class=r#"text-blue-600 underline"#
                    >
                        {a.file_name.clone()}
                    </a>
                </For>
            </Show>

            <Show when=move || editing.get()>
                <label class=r#"block mb-1 text-sm font-medium"#>"Event"</label>
                <select
                    class=r#"bg-background py-2 px-3 w-full text-sm rounded-md border border-input-border 
                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                    name="event_id"
                    bind:value=event_id_edit
                >
                    <option value="">"-- Select Event --"</option>
                    <For
                        each=move || events.get()
                        key=|e: &crate::server::db::structs::Event| e.id.clone()
                        let(e: crate::server::db::structs::Event)
                    >
                        <option value=e
                            .id
                            .clone()>{e.name.clone()} " (ID: " {e.id.clone()} ")"</option>
                    </For>
                </select>

                <label class=r#"block mb-1 text-sm font-medium"#>"Name"</label>
                <input
                    class=r#"bg-background py-2 px-3 w-full text-sm rounded-md border border-input-border 
                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                    name="name"
                    value=move || name_signal.get()
                    bind:value=name_edit
                />

                <label class=r#"block mb-1 text-sm font-medium"#>"Description"</label>
                <input
                    class=r#"bg-background py-2 px-3 w-full text-sm rounded-md border border-input-border 
                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                    name="description"
                    value=move || description_signal.get().unwrap_or_default()
                    on:change=move |ev: Event| {
                        let value = event_target_value(&ev);
                        description_edit.set(Some(value));
                    }
                />

                <label class=r#"block mb-1 text-sm font-medium"#>"Category"</label>
                <select
                    class=r#"bg-background py-2 px-3 w-full text-sm rounded-md border border-input-border 
                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                    name="category"
                    on:change=move |ev: Event| {
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
                        category_edit.set(Some(sel.value()));
                    }
                >
                    <option value="">"-- Select Category --"</option>
                    <For
                        each=move || categories.get()
                        key=|category: &String| category.clone()
                        let(category)
                    >
                        <option value=category.clone()>{category.clone()}</option>
                    </For>
                    <option value="__new__">"-- Add New --"</option>
                </select>
                <input
                    class=r#"bg-background py-2 px-3 w-full text-sm rounded-md border border-input-border 
                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                    hidden=move || !category_add_new_selected.get()
                    type="text"
                    id="action_create_category_input"
                    value=""
                    on:change=move |ev: Event| {
                        let value = event_target_value(&ev);
                        category_edit.set(Some(value));
                    }
                />

                <label class=r#"block mb-1 text-sm font-medium"#>"Difficulty"</label>
                <input
                    class=r#"bg-background py-2 px-3 w-full text-sm rounded-md border border-input-border 
                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
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

                <label class=r#"block mb-1 text-sm font-medium"#>"Points"</label>
                <input
                    class=r#"bg-background py-2 px-3 w-full text-sm rounded-md border border-input-border 
                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                    type="number"
                    name="points"
                    value=move || points_signal.get()
                    on:change=move |ev: Event| {
                        let value = event_target_value(&ev);
                        points_edit.set(value.parse::<u32>().unwrap_or_default());
                    }
                />

                <label class=r#"block mb-1 text-sm font-medium"#>"Flag"</label>
                <input
                    class=r#"bg-background py-2 px-3 w-full text-sm rounded-md border border-input-border 
                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                    name="flag"
                    bind:value=flag_edit
                />

                <label class=r#"block mb-1 text-sm font-medium text-text"#>"Visible To Groups"</label>
                <select
                    class=r#"bg-background py-2 px-3 w-full text-sm rounded-md border border-input-border 
                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                    name="visible_to_groups"
                    multiple=true
                    on:change=move |ev: Event| {
                        let sel = ev.target().unwrap().unchecked_into::<HtmlSelectElement>();
                        let selected = sel.selected_options();
                        let mut picked: Vec<String> = Vec::new();

                        for i in 0..selected.length() {
                            if let Some(item) = selected.item(i) {
                                if let Ok(opt) = item.dyn_into::<HtmlOptionElement>() {
                                    picked.push(opt.value());
                                }
                            }
                        }

                        visible_to_groups_edit.set(picked.join(","));
                    }
                >
                    <option value="all">"All"</option>
                    <Suspense fallback=move || {
                        view! { <div>"Loading..."</div> }
                    }>
                        {move || {
                            let groups = groups_resource.get().unwrap_or_default();
                            view! {
                                <For
                                    each=move || groups.clone()
                                    key=|group: &String| group.clone()
                                    let(group)
                                >
                                    <option value={group.clone()}>{group.clone()}</option>
                                </For>
                            }
                        }}
                    </Suspense>
                </select>

                <label class=r#"block mb-1 text-sm font-medium"#>"Proxmox VM IDs (Optional)"</label>
                <select
                    class=r#"bg-background py-2 px-3 w-full text-sm rounded-md border border-input-border 
                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                    name="vm_ids"
                    multiple=true
                    on:change=move |ev: Event| {
                        let sel = ev.target().unwrap().unchecked_into::<HtmlSelectElement>();
                        let selected = sel.selected_options();
                        let mut picked: Vec<String> = Vec::new();

                        for i in 0..selected.length() {
                            if let Some(item) = selected.item(i) {
                                if let Ok(opt) = item.dyn_into::<HtmlOptionElement>() {
                                    picked.push(opt.value());
                                }
                            }
                        }

                        proxmox_vm_id_edit.set(Some(picked.join(",")));
                    }
                >
                    <Suspense fallback=move || {
                        view! { <div>"Loading..."</div> }
                    }>
                        {move || {
                            let templates = templates.get();
                            view! {
                                <For
                                    each=move || templates.clone()
                                    key=|template: &ProxmoxVMTemplate| template.id
                                    let(template)
                                >
                                    <option value={template.id}>{format!("{} (VM ID: {})", template.name, template.id)}</option>
                                </For>
                            }
                        }}
                    </Suspense>
                </select>

                <Transition>
                    {move || {
                        let challenge_id = id_signal.get();
                        let hints = hints.get().into_iter().filter(|h| h.challenge_id == challenge_id).collect::<Vec<DbHint>>();
                        hints_edit.set(hints.into_iter()
                            .map(|h| crate::pages::admin::challenges::Hint {
                                id: h.id,
                                value: RwSignal::new(h.hint),
                                points_penalty: RwSignal::new(Some(h.points_penalty))
                            })
                            .collect::<Vec<crate::pages::admin::challenges::Hint>>()
                        );
                        if hints_edit.get_untracked().is_empty() {
                            hints_edit.set(vec![Hint::new(next_hint_id.get().to_string(), "")]);
                        }

                        view! {
                            <label class=r#"block mb-1 text-sm font-medium text-text"#>"Hints"</label>
                            <ForEnumerate
                                each=move || hints_edit.get()
                                key=|hint: &Hint| hint.id.clone()
                                children={move |index, hint| {
                                    view! {
                                        <div class="flex gap-2">
                                            <input
                                                class=r#"bg-background py-2 px-3 w-full text-sm rounded-md border border-input-border 
                                                focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                                                name="hint"
                                                prop:value=move || hint.value.get()
                                                on:input=move |ev| {
                                                    let value = event_target_value(&ev);
                                                    hint.value.set(value);
                                                }
                                            />
                                            <input
                                                class="w-1/2"
                                                type="number"
                                                min=0
                                                prop:value=move || hint.points_penalty.get()
                                                on:input=move |ev| {
                                                    let value = event_target_value(&ev);
                                                    hint.points_penalty.set(Some(value.parse::<u32>().unwrap_or_default()));
                                                }
                                                placeholder="0"
                                            />
                                            <button 
                                                class="cursor-pointer"
                                                on:click=move |_| {
                                                    hints_edit.update(|vec| {
                                                        next_hint_id.set(next_hint_id.get() + 1);
                                                        vec.push(Hint::new(next_hint_id.get().to_string(), ""));
                                                        leptos::logging::log!("{vec:?}");
                                                    });
                                                }
                                            >
                                                <Icon icon=i::LuPlus />
                                            </button>
                                            {
                                                if index.get() != 0 {
                                                    view! {
                                                        <button 
                                                            class="cursor-pointer"
                                                            on:click=move |_| {
                                                                let remove_at = index.get();

                                                                hints_edit.update(|vec| {
                                                                    vec.remove(remove_at);
                                                                    leptos::logging::log!("{vec:?}");
                                                                });
                                                            } 
                                                        >
                                                            <Icon icon=i::LuX />
                                                        </button>
                                                    }.into_any()
                                                } else {
                                                    "".into_any()
                                                }
                                            }
                                        </div>
                                    }
                                }}
                            />
                        }
                    }}
                </Transition>

                <label class=r#"block mb-1 text-sm font-medium"#>"Attachment"</label>
                <input
                    class=r#"bg-background w-full text-sm p-3 rounded-lg shadow-sm"#
                    type="file"
                    name="attachment"
                    multiple
                    on:change=move |ev: Event| {
                        let input = ev.target().unwrap().unchecked_into::<HtmlInputElement>();
                        if let Some(files) = input.files() && files.length() > 0 {
                            let file = files.get(0).unwrap();
                            let fd = FormData::new().unwrap();
                            fd.append_with_blob_and_filename("file", &file, &file.name()).unwrap();
                            file_upload_action.dispatch_local(fd);
                        }
                    }
                /><p>{move || uploading_file_text.get()}</p>

                <label class=r#"block mb-1 text-sm font-medium"#>"Illustration"</label>
                <input
                    class=r#"bg-background w-full text-sm p-3 rounded-lg shadow-sm"#
                    type="file"
                    name="illustration"
                    on:change=move |ev: Event| {
                        let input = ev.target().unwrap().unchecked_into::<HtmlInputElement>();
                        if let Some(files) = input.files() && files.length() > 0 {
                            let file = files.get(0).unwrap();
                            let fd = FormData::new().unwrap();
                            fd.append_with_blob_and_filename("file", &file, &file.name()).unwrap();
                            illustration_upload_action.dispatch_local(fd);
                        }
                    }
                /><p>{move || uploading_illustration_text.get()}</p>
            </Show>

            <div class=r#"flex flex-row-reverse gap-3 mt-2"#>
                <Show when=move || editing.get() || deleting.get()>
                    <button
                        class=r#"py-2 px-4 text-sm rounded-md border border-input-border hover:bg-background-hover"#
                        on:click=move |_| {
                            spawn_local(async move {
                                editing.set(false);
                                deleting.set(false);
                            });
                        }
                    >
                        "Cancel"
                    </button>
                </Show>
                <button
                    type="button"
                    hidden=move || deleting.get()
                    class=r#"inline-flex gap-2 items-center py-2 px-4 text-sm font-medium text-white 
                    rounded-lg transition focus:ring-2 focus:outline-none active:scale-95 
                    bg-yale-blue-600 hover:bg-yale-blue-700 focus:ring-yale-blue-400"#
                    on:click=move |_| {
                        let challenge_id = id_signal.get();
                        let event_id = event_id_edit.get();
                        let name = name_edit.get();
                        let description = description_edit.get();
                        let category = category_edit.get();
                        let difficulty = difficulty_edit.get();
                        let points = points_edit.get();
                        let flag = flag_edit.get();
                        let visible_to_groups = visible_to_groups_edit.get();
                        let attachments = attachments_edit.get();
                        let illustration = illustration_edit.get();
                        let vm_ids = proxmox_vm_id_edit.get();
                        let hints = hints_edit.get().into_iter().map(|h| {
                            let mut hint = Into::<DbHint>::into(h); 
                            hint.challenge_id = challenge_id.clone(); 
                            hint
                        }).collect::<Vec<DbHint>>();
                        if editing.get() {
                            spawn_local(async move {
                                if let Ok(ApiResult { result, .. }) = crate::server::admin::challenge(crate::server::admin::ChallengeAction::Edit {
                                        id: challenge_id.clone(),
                                        event_id: event_id.clone(),
                                        name: name.clone(),
                                        description: description.clone().unwrap_or_default(),
                                        category: category.clone().unwrap_or_default(),
                                        difficulty,
                                        points,
                                        flag: flag.clone(),
                                        visible_to_groups,
                                        attachments: attachments.clone(),
                                        illustration: illustration.clone(),
                                        vm_ids,
                                        hints: hints.into()
                                    })
                                    .await && result == ResultStatus::Success
                                {
                                    refresh.update(|n| *n += 1);
                                    editing.set(false);
                                    event_id_signal.set(event_id);
                                    name_signal.set(name);
                                    description_signal.set(description);
                                    category_signal.set(category);
                                    difficulty_signal.set(difficulty);
                                    points_signal.set(points);
                                    attachments_signal.set(attachments.unwrap_or_default());
                                }
                            });
                        } else {
                            editing.set(true)
                        }
                    }
                >
                    {move || edit_submit_btn_text.get()}
                </button>
                <button
                    hidden=move || editing.get()
                    class=r#"inline-flex items-center py-2 px-4 ml-auto text-sm font-semibold text-white 
                    bg-red-600 rounded-md shadow-sm hover:bg-red-500 focus:ring-2 focus:outline-none 
                    focus:ring-yale-blue-500"#
                    on:click=move |_| {
                        if deleting.get() {
                            let challenge_id = id_signal.get().clone();
                            spawn_local(async move {
                                tracing::debug!("deleting challenge ID: {challenge_id}");
                                if let Ok(ApiResult { result, .. }) = crate::server::admin::challenge(crate::server::admin::ChallengeAction::Delete {
                                        id: challenge_id.clone(),
                                    })
                                    .await && result == ResultStatus::Success
                                {
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
        <div class=r#"difficulty"# role="img" aria-label=format!("Difficulty: {} of 5", rating)>
            <span class=r#"label"#>
                <b class=r#"text-lg/8"#>"Difficulty: "</b>
                {"⭐".repeat(rating as usize)}
            </span>
        </div>
    }
}
