use crate::server::admin::{get_all_challenge_templates, get_all_user_groups, upload_illustration};
use crate::server::db::structs::{AttachmentWithoutBlob, ChallengeWithAttachments};
use crate::server::enums::{AdminEventPayloadKind, ResultStatus};
use crate::server::proxmox::ProxmoxVMTemplate;
use crate::server::structs::ApiResult;
use crate::{components::admin::challenge::Challenge, server::{admin::{upload_files, get_all_challenge_categories, get_all_events}, db, get_all_challenges_with_attachments}};
use gloo_timers::future::sleep;
use leptos::prelude::*;
use leptos::{web_sys::{FormData, HtmlInputElement, Event, HtmlSelectElement, HtmlOptionElement}, wasm_bindgen::JsCast, task::spawn_local};
use leptos_use::{UseEventSourceOptions, UseEventSourceReturn, use_event_source_with_options};
use leptos::server::codee::string::FromToStringCodec;
use std::{collections::HashMap, time::Duration};

/// Default Home Page
#[component]
pub fn Challenges() -> impl IntoView {
    let category_add_new_selected = RwSignal::new(false);
    
    let event_id = RwSignal::new("".to_string());
    let name = RwSignal::new("".to_string());
    let description = RwSignal::new(None);
    let category = RwSignal::new(None);
    let difficulty = RwSignal::new(0_i8);
    let points = RwSignal::new(0_u32);
    let flag = RwSignal::new("".to_string());
    let visible_to_groups = RwSignal::new(vec![]);
    let attachments = RwSignal::<Option<Vec<AttachmentWithoutBlob>>>::new(None);
    let illustration = RwSignal::<Option<AttachmentWithoutBlob>>::new(None);
    let vm_ids = RwSignal::new(None);

    let refresh = RwSignal::new(0);
    let categories_signal = RwSignal::<Vec<String>>::new(vec![]);
    let events_signal = RwSignal::<Vec<db::structs::Event>>::new(vec![]);
    let templates_signal = RwSignal::<Vec<ProxmoxVMTemplate>>::new(vec![]);

    let cwa_resource = Resource::new(move || refresh.get(), move |_| async move {
        get_all_challenges_with_attachments().await.unwrap_or_default()
    });
    let categories_resource = Resource::new(move || refresh.get(), move |_| async move {
        let all_categories = get_all_challenge_categories().await.unwrap_or_default();
        categories_signal.set(all_categories.clone());
        all_categories
    });
    let events_resource = Resource::new(move || refresh.get(), move |_| async move {
        let all_events = get_all_events().await.unwrap_or_default();
        events_signal.set(all_events.clone());
        all_events
    });
    let groups_resource = Resource::new(move || refresh.get(), move |_| async move {
        get_all_user_groups().await.unwrap_or_default()
    });
    let challenge_templates_resource = Resource::new(move || refresh.get(), move |_| async move {
        let templates = get_all_challenge_templates().await.unwrap_or_default();
        templates_signal.set(templates.clone());
        templates
    });

    let file_upload_action = Action::new_local(|data: &FormData| {
        upload_files(data.clone().into())
    });

    let illustration_upload_action = Action::new_local(|data: &FormData| {
        upload_illustration(data.clone().into())
    });

    let UseEventSourceReturn { message, .. } = 
        use_event_source_with_options::<String, FromToStringCodec>(
            "/admin_sse".to_string(), 
            UseEventSourceOptions::default().immediate(true)
        );

    Effect::new(move |_| {
        if let Some(Ok(api_result)) = file_upload_action.value().get() {
            attachments.set(Some(api_result.details.clone()));
        } else if let Some(Ok(api_result)) = illustration_upload_action.value().get() {
            illustration.set(Some(api_result.details.clone()));
        }

        if let Some(msg) = message.get() {
            match serde_json::from_str::<AdminEventPayloadKind>(&msg.data) {
                Ok(AdminEventPayloadKind::ChallengeEdited)  => refresh.update(|n| *n += 1),
                Ok(_) => {},
                Err(e) => tracing::warn!("failed to parse AdminEventPayloadKind: {}", e)
            }
        }
    });

    let created = RwSignal::new(false);
    let creating = RwSignal::new(false);
    let create_submit_btn_text = Memo::new(move |_| {
        if created.get() { "Created!".to_string() } else { "Create".to_string() }
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
        <div class=r#"flex gap-2 mb-4"#>
            <button
                class=r#"py-1 px-3 text-sm rounded-md border border-input-border hover:bg-background-hover"#
                on:click=move |_| {
                    if creating.get() {
                        creating.set(false);
                    } else {
                        creating.set(true);
                    }
                }
            >
                "Create"
            </button>
        </div>

        <div class=r#"flex flex-col gap-4"#>
            <Show when=move || creating.get()>
                <label class=r#"block mb-1 text-sm font-medium text-text"#>"Event"</label>
                <select
                    class=r#"bg-background py-2 px-3 w-full text-sm rounded-md border border-input-border 
                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                    name="event_id"
                    bind:value=event_id
                >
                    <option value="">"-- Select Event --"</option>
                    <Suspense fallback=move || {
                        view! { <div>"Loading..."</div> }
                    }>
                        {move || {
                            let events = events_resource.get().unwrap_or_default();
                            view! {
                                <For
                                    each=move || events.clone()
                                    key=|e: &db::structs::Event| e.id.clone()
                                    let(e: db::structs::Event)
                                >
                                    <option value=e
                                        .id
                                        .clone()>
                                        {e.name.clone()} " (ID: " {e.id.clone()} ")"
                                    </option>
                                </For>
                            }
                        }}
                    </Suspense>
                </select>

                <label class=r#"block mb-1 text-sm font-medium text-text"#>"Name"</label>
                <input
                    class=r#"bg-background py-2 px-3 w-full text-sm rounded-md border border-input-border 
                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                    name="name"
                    bind:value=name
                />

                <label class=r#"block mb-1 text-sm font-medium text-text"#>"Description"</label>
                <input
                    class=r#"bg-background py-2 px-3 w-full text-sm rounded-md border border-input-border 
                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                    name="description"
                    on:change=move |ev: Event| {
                        let value = event_target_value(&ev);
                        description.set(Some(value));
                    }
                />

                <label class=r#"block mb-1 text-sm font-medium text-text"#>"Category"</label>
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
                        category.set(Some(sel.value()))
                    }
                >
                    <option value="">"-- Select Category --"</option>
                    <Suspense fallback=move || {
                        view! { <div>"Loading..."</div> }
                    }>
                        {move || {
                            let categories = categories_resource.get().unwrap_or_default();
                            view! {
                                <For
                                    each=move || categories.clone()
                                    key=|category: &String| category.clone()
                                    let(category)
                                >
                                    <option value=category.clone()>{category.clone()}</option>
                                </For>
                            }
                        }}
                    </Suspense>
                    <option value="__new__">"-- Add New --"</option>
                </select>
                <input
                    class=r#"bg-background py-2 px-3 mt-2 w-full text-sm rounded-md border border-input-border 
                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                    hidden=move || !category_add_new_selected.get()
                    type="text"
                    id="action_create_category_input"
                    value=""
                    on:change=move |ev: Event| {
                        let value = event_target_value(&ev);
                        category.set(Some(value));
                    }
                />

                <label class=r#"block mb-1 text-sm font-medium text-text"#>"Difficulty"</label>
                <input
                    class=r#"bg-background py-2 px-3 w-full text-sm rounded-md border border-input-border 
                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                    type="number"
                    min="1"
                    max="5"
                    name="difficulty"
                    on:change=move |ev: Event| {
                        let value = event_target_value(&ev);
                        difficulty.set(value.parse::<i8>().unwrap_or_default());
                    }
                />

                <label class=r#"block mb-1 text-sm font-medium text-text"#>"Points"</label>
                <input
                    class=r#"bg-background py-2 px-3 w-full text-sm rounded-md border border-input-border 
                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                    type="number"
                    min="0"
                    name="points"
                    on:change=move |ev: Event| {
                        let value = event_target_value(&ev);
                        points.set(value.parse::<u32>().unwrap_or_default());
                    }
                />

                <label class=r#"block mb-1 text-sm font-medium text-text"#>"Flag"</label>
                <input
                    class=r#"bg-background py-2 px-3 w-full text-sm rounded-md border border-input-border 
                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                    name="flag"
                    bind:value=flag
                />

                <label class=r#"block mb-1 text-sm font-medium text-text"#>"Visible To Groups"</label>
                <select
                    class=r#"bg-background py-2 px-3 w-full text-sm rounded-md border border-input-border 
                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                    name="visible_to_groups"
                    multiple=true
                    on:change=move |ev: Event| {
                        let select = match ev.current_target() {
                            Some(t) => match t.dyn_into::<HtmlSelectElement>() {
                                Ok(s) => s,
                                Err(_) => return,
                            },
                            None => return
                        };

                        let selected = select.selected_options();
                        let mut picked: Vec<String> = Vec::new();

                        for i in 0..selected.length() {
                            if let Some(item) = selected.item(i) {
                                if let Ok(opt) = item.dyn_into::<HtmlOptionElement>() {
                                    picked.push(opt.value());
                                }
                            }
                        }

                        visible_to_groups.set(picked);
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

                <label class=r#"block mb-1 text-sm font-medium text-text"#>"Proxmox VM IDs (Optional)"</label>
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

                        vm_ids.set(Some(picked.join(",")));
                    }
                >
                    <Suspense fallback=move || {
                        view! { <div>"Loading..."</div> }
                    }>
                        {move || {
                            let templates = challenge_templates_resource.get().unwrap_or_default();
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

                <label class=r#"block mb-1 text-sm font-medium text-text"#>
                    "Attachment (Max 16 MiB)"
                </label>
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
                />
                <p>{uploading_file_text.get()}</p>

                <label class=r#"block mb-1 text-sm font-medium text-text"#>
                    "Illustration (Max 16 MiB)"
                </label>
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
                />
                <p>{uploading_illustration_text.get()}</p>

                <div class=r#"flex gap-3 mt-2"#>
                    <button
                        type="button"
                        class=r#"py-2 px-4 text-sm rounded-md border border-input-border hover:bg-background-hover"#
                        on:click=move |_| {
                            creating.set(false);
                        }
                    >
                        "Cancel"
                    </button>
                    <button
                        class=r#"inline-flex items-center py-2 px-4 ml-auto text-sm font-semibold 
                        text-white rounded-md shadow-sm focus:ring-2 focus:outline-none 
                        bg-yale-blue-600 hover:bg-yale-blue-500 focus:ring-yale-blue-500"#
                        on:click=move |_| {
                            let event_id = event_id.get();
                            let name = name.get();
                            let description = description.get().unwrap_or_default();
                            let category = category.get().unwrap_or_default();
                            let difficulty = difficulty.get();
                            let points = points.get();
                            let flag = flag.get();
                            let visible_to_groups = visible_to_groups.get().join(",");
                            let attachments = attachments.get();
                            let illustration = illustration.get();
                            let vm_ids = vm_ids.get();
                            spawn_local(async move {
                                if let Ok(ApiResult { result, .. }) = crate::server::admin::challenge(crate::server::admin::ChallengeAction::Create {
                                        event_id,
                                        name,
                                        description,
                                        category,
                                        difficulty,
                                        points,
                                        flag,
                                        visible_to_groups,
                                        vm_ids,
                                        attachments,
                                        illustration,
                                    })
                                    .await && result == ResultStatus::Success
                                {
                                    created.set(true);
                                    refresh.update(|n| *n += 1);
                                    sleep(Duration::from_secs(2)).await;
                                    created.set(false);
                                }
                            });
                        }
                    >
                        {create_submit_btn_text.get()}
                    </button>
                </div>
            </Show>
        </div>

        <div class=r#"challenges pt-4"#>
            <Transition fallback=move || {
                view! { <div>"Loading..."</div> }
            }>
                {move || {
                    let mut map = HashMap::<Option<String>, Vec<ChallengeWithAttachments>>::new();
                    for ch in cwa_resource.get().unwrap_or_default().into_iter() {
                        map.entry(ch.challenge.category.clone()).or_default().push(ch);
                    }
                    let mut groups = map
                        .into_iter()
                        .collect::<Vec<(Option<String>, Vec<ChallengeWithAttachments>)>>();
                    groups
                        .sort_by(|(a, _), (b, _)| {
                            a.as_deref().unwrap_or("").cmp(b.as_deref().unwrap_or(""))
                        });

                    // alphabetical sort, there's probably a better way to do this

                    view! {
                        <For
                            each=move || groups.clone()
                            key=|group: &(Option<String>, Vec<ChallengeWithAttachments>)| {
                                group.0.clone()
                            }
                            let(group)
                        >
                            <div class=r#"p-2 challenge-category"#>
                                <h2 class=r#"text-2xl"#>
                                    {group.0.clone().unwrap_or_else(|| "Uncategorized".to_string())}
                                </h2>

                                <div class=r#"grid grid-cols-4 m-4 content-stretch"#>
                                    <For
                                        each=move || group.1.clone()
                                        key=|challenge: &ChallengeWithAttachments| {
                                            challenge.challenge.id.clone()
                                        }
                                        let(challenge)
                                    >
                                        <div class=r#"p-2 challenge"#>
                                            <Challenge
                                                cwa=challenge
                                                refresh
                                                categories=categories_signal
                                                events=events_signal
                                                templates=templates_signal
                                            />
                                        </div>
                                    </For>
                                </div>
                            </div>
                        </For>
                    }
                }}
            </Transition>
        </div>
    }
}
