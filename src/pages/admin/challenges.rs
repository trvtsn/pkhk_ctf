use crate::components::toast::{ToastMessageType, push_new_toast};
use crate::components::utils::{ComponentSize, FileTooltip};
use crate::utils::{build_multi_file_form_data, build_single_file_form_data, collect_selected_options};
use crate::server::admin::{api::{get_all_challenge_templates, get_all_hints, get_all_user_groups, upload_illustration}};
use crate::server::db::structs::{AttachmentWithoutBlob, ChallengeWithAttachments, DbHint};
use crate::server::enums::{ServerEventPayload, ResultStatus};
use crate::server::proxmox::ProxmoxVMTemplate;
use crate::server::structs::ApiResult;
use crate::{components::{admin::challenge::Challenge, utils::Spinner}, server::{admin::{api::{upload_files, get_all_challenge_categories, get_all_events}}, db, api::get_all_challenges_with_attachments}};
use icondata as i;
use leptos::prelude::*;
use leptos::{web_sys::{Event, HtmlSelectElement}, wasm_bindgen::JsCast, task::spawn_local};
use leptos_icons::Icon;
use leptos_use::{UseEventSourceOptions, UseEventSourceReturn, use_event_source_with_options};
use leptos::server::codee::string::FromToStringCodec;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time::Duration};

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub struct Hint {
    pub id: String,
    pub value: RwSignal<String>,
    pub points_penalty: RwSignal<Option<u32>>
}

impl Hint {
    pub fn new(id: String, initial: impl Into<String>) -> Self {
        Self {
            id,
            value: RwSignal::new(initial.into()),
            points_penalty: RwSignal::new(None)
        }
    }
}

impl From<Hint> for crate::server::db::structs::Hint {
    fn from(hint: Hint) -> Self {
        Self {
            id: hint.id,
            hint: hint.value.get(),
            challenge_id: "".to_string(),
            points_penalty: hint.points_penalty.get().unwrap_or(0)
        }
    }
}

/// Default Home Page
#[component]
pub fn Challenges() -> impl IntoView {
    let created = RwSignal::new(false);
    let creating = RwSignal::new(false);

    let attachments_ref = NodeRef::new();
    let illustration_ref = NodeRef::new();

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
    let create_hints = RwSignal::new(Some(vec![Hint::new("0".to_string(), "")]));
    let next_hint_id = RwSignal::new(1_usize);

    let editing_ids = RwSignal::<Vec<String>>::new(vec![]);
    let pending_cwa_updates = RwSignal::<Vec<ChallengeWithAttachments>>::new(vec![]);
    let cwa_signal = RwSignal::<Vec<ChallengeWithAttachments>>::new(vec![]);
    let categories_signal = RwSignal::<Vec<String>>::new(vec![]);
    let events_signal = RwSignal::<Vec<db::structs::Event>>::new(vec![]);
    let templates_signal = RwSignal::<Vec<ProxmoxVMTemplate>>::new(vec![]);
    let user_groups_signal = RwSignal::new(vec![]);
    let all_hints_signal = RwSignal::new(vec![]);

    let cwa_resource = Resource::new(move || (), move |_| async move {
        get_all_challenges_with_attachments().await.unwrap_or_default()
    });
    let categories_resource = Resource::new(move || (), move |_| async move {
        get_all_challenge_categories().await.unwrap_or_default()
    });
    let events_resource = Resource::new(move || (), move |_| async move {
        get_all_events().await.unwrap_or_default()
    });
    let groups_resource = Resource::new(move || (), move |_| async move {
        get_all_user_groups().await.unwrap_or_default()
    });
    let challenge_templates_resource = Resource::new(move || (), move |_| async move {
        get_all_challenge_templates().await.unwrap_or_default()
    });
    let all_hints_resource = Resource::new(move || (), move |_| async move {
        get_all_hints().await.unwrap_or_default()
    });

    let create_submit_btn_text = Memo::new(move |_| {
        if created.get() { "Created!".to_string() } else { "Create".to_string() }
    });

    let grouped_challenges = Memo::new(move |_| {
        let mut map = HashMap::<Option<String>, Vec<ChallengeWithAttachments>>::new();
        for ch in cwa_signal.get().into_iter() {
            map.entry(ch.challenge.category.clone()).or_default().push(ch);
        }
        let mut groups = map
            .into_iter()
            .collect::<Vec<(Option<String>, Vec<ChallengeWithAttachments>)>>();
        // alphabetical sort, there's probably a better way to do this
        groups
            .sort_by(|(a, _), (b, _)| {
                a.as_deref().unwrap_or("").cmp(b.as_deref().unwrap_or(""))
            });

        groups
    });

    let UseEventSourceReturn { message, .. } =
        use_event_source_with_options::<String, FromToStringCodec>(
            "/admin/events".to_string(),
            UseEventSourceOptions::default().immediate(true)
        );

    Effect::new(move |_| {
        if let Some(msg) = message.get() {
            match serde_json::from_str::<ServerEventPayload>(&msg.data) {
                Ok(ServerEventPayload::ChallengeEdited(new_cwa)) => {
                    if editing_ids.get_untracked().contains(&new_cwa.challenge.id) {
                        pending_cwa_updates.update(|pending| {
                            pending.retain(|p| p.challenge.id != new_cwa.challenge.id);
                            pending.push(new_cwa);
                        });
                    } else {
                        cwa_signal.update(|challenges| {
                            if let Some(existing) = challenges.iter_mut().find(|c| c.challenge.id == new_cwa.challenge.id) {
                                *existing = new_cwa;
                            }
                        });
                    }
                },
                Ok(ServerEventPayload::ChallengeDeleted(id)) => {
                    cwa_signal.update(|cwa| cwa.retain(|cwa| cwa.challenge.id != id));
                },
                Ok(ServerEventPayload::NewChallengeCreated(new_cwa)) => {
                    cwa_signal.update(|cwa| cwa.push(new_cwa));
                }, 
                Ok(_) => {},
                Err(e) => tracing::warn!("failed to parse ServerEventPayload: {}", e)
            }
        }
    });

    // Flush pending SSE updates for challenges no longer being edited
    Effect::new(move |_| {
        let current_editing = editing_ids.get();
        let pending = pending_cwa_updates.get_untracked();
        let to_flush: Vec<_> = pending.iter()
            .filter(|p| !current_editing.contains(&p.challenge.id))
            .cloned()
            .collect();
        if !to_flush.is_empty() {
            pending_cwa_updates.update(|p| p.retain(|u| current_editing.contains(&u.challenge.id)));
            cwa_signal.update(|challenges| {
                for updated_cwa in to_flush {
                    if let Some(existing) = challenges.iter_mut().find(|c| c.challenge.id == updated_cwa.challenge.id) {
                        *existing = updated_cwa;
                    }
                }
            });
        }
    });

    view! {
        <div class=r#"flex gap-2 mb-4"#>
            <button
                class=r#"py-1 px-3 text-sm rounded-md border border-input-border bg-background hover:bg-background-hover"#
                on:click=move |_| {
                    creating.update(|c| *c = !*c);
                }
            >
                "Create"
            </button>
        </div>

        <div class=r#"flex flex-col gap-4"#>
            <Show when=move || creating.get()>
                <div class="grid">
                    <label class=r#"block mb-1 text-sm font-medium text-text"#>"Event"</label>
                    <select
                        class=r#"bg-background py-2 px-3 w-full text-sm rounded-md border border-input-border 
                        focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                        name="event_id"
                        required
                        bind:value=event_id
                    >
                        <option value="">"-- Select Event --"</option>
                        <Suspense fallback=move || {
                            view! { <Spinner component_size=ComponentSize::Small /> }
                        }>
                            {move || {
                                view! {
                                    <For
                                        each=move || events_signal.get()
                                        key=|e: &db::structs::Event| e.id.clone()
                                        let(e: db::structs::Event)
                                    >
                                        <option value=e.id>
                                            {e.name.clone()} " (ID: " {e.id.clone()} ")"
                                        </option>
                                    </For>
                                }
                            }}
                        </Suspense>
                    </select>
                </div>

                <div class="grid">
                    <label class=r#"block mb-1 text-sm font-medium text-text"#>"Name"</label>
                    <input
                        class=r#"bg-background py-2 px-3 w-full text-sm rounded-md border border-input-border 
                        focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                        name="name"
                        required
                        bind:value=name
                    />
                </div>

                <div class="grid">
                    <label class=r#"block mb-1 text-sm font-medium text-text"#>"Description"</label>
                    <textarea
                        class=r#"bg-background py-2 px-3 w-full text-sm rounded-md border border-input-border 
                        focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                        name="description"
                        on:input=move |ev: Event| {
                            let value = event_target_value(&ev);
                            description.set(Some(value));
                        }
                    />
                </div>

                <div class="grid">
                    <label class=r#"block mb-1 text-sm font-medium text-text"#>"Category"</label>
                    <select
                        class=r#"bg-background py-2 px-3 w-full text-sm rounded-md border border-input-border 
                        focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                        name="category"
                        on:change=move |ev: Event| {
                            let value = event_target_value(&ev);
                            if value == "__new__" {
                                category_add_new_selected.set(true);
                            } else {
                                category_add_new_selected.set(false);
                                category.set(Some(value));
                            }
                        }
                    >
                        <option value="">"-- Select Category --"</option>
                        <Suspense fallback=move || {
                            view! { <Spinner component_size=ComponentSize::Small /> }
                        }>
                            {move || {
                                view! {
                                    <For
                                        each=move || categories_signal.get()
                                        key=|category: &String| category.clone()
                                        let(category)
                                    >
                                        <option value=category>{category.clone()}</option>
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
                </div>

                <div class="grid">
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
                </div>

                <div class="grid">
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
                </div>
                
                <div class="grid">
                    <label class=r#"block mb-1 text-sm font-medium text-text"#>"Flag"</label>
                    <input
                        class=r#"bg-background py-2 px-3 w-full text-sm rounded-md border border-input-border 
                        focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                        name="flag"
                        bind:value=flag
                    />
                </div>

                <div class="grid">
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
                            visible_to_groups.set(collect_selected_options(&select));
                        }
                    >
                        <option value="all">"All"</option>
                        {move || {
                            let groups = user_groups_signal.get();
                            view! {
                                <For
                                    each=move || groups.clone()
                                    key=|group: &String| group.clone()
                                    let(group)
                                >
                                    <option value=group>{group.clone()}</option>
                                </For>
                            }
                        }}
                    </select>
                </div>

                <div class="grid">
                    <label class=r#"block mb-1 text-sm font-medium text-text"#>"Proxmox VM IDs (Optional)"</label>
                    <select
                        class=r#"bg-background py-2 px-3 w-full text-sm rounded-md border border-input-border 
                        focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                        name="vm_ids"
                        multiple=true
                        on:change=move |ev: Event| {
                            let sel = match ev.target() {
                                Some(target) => target.unchecked_into::<HtmlSelectElement>(),
                                None => { push_new_toast(ToastMessageType::ErrorOccurred); return }
                            };
                            vm_ids.set(Some(collect_selected_options(&sel).join(",")));
                        }
                    >
                        <Suspense fallback=move || {
                            view! { <Spinner component_size=ComponentSize::Small /> }
                        }>
                            {move || {
                                view! {
                                    <For
                                        each=move || templates_signal.get()
                                        key=|template: &ProxmoxVMTemplate| template.id
                                        let(template)
                                    >
                                        <option value=template.id>
                                            {format!("{} (VM ID: {})", template.name, template.id)}
                                        </option>
                                    </For>
                                }
                            }}
                        </Suspense>
                    </select>
                </div>

                <div class="grid">
                    <label class=r#"block mb-1 text-sm font-medium text-text"#>"Hints"</label>
                    <ForEnumerate
                        each=move || create_hints.get().unwrap_or_default()
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
                                            hint.value.set(value.clone());
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
                                            create_hints.update(|vec| {
                                                let vec = vec.get_or_insert_with(Vec::new);
                                                next_hint_id.set(next_hint_id.get_untracked() + 1);
                                                vec.push(Hint::new(next_hint_id.get_untracked().to_string(), ""));
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
                                                        let remove_at = index.get_untracked();

                                                        create_hints.update(|vec| {
                                                            let vec = vec.get_or_insert_with(Vec::new);
                                                            vec.remove(remove_at);
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
                </div>
                
                <div class="grid">
                    <label class=r#"block mb-1 text-sm font-medium text-text"#>
                        "Attachments (Max 16 MiB)"
                    </label>
                    <div class="grid gap-2">
                        <ForEnumerate
                            each=move || attachments.get().unwrap_or_default()
                            key=|a: &AttachmentWithoutBlob| a.id.clone()
                            children={move |index, a| {
                                let id = a.id.clone();
                                let file_name = a.file_name.clone();
                                view! {
                                    <FileTooltip
                                        file_name=file_name
                                        id=a.id.clone()
                                        on_download=format!("/file/{}", id)
                                        on_remove=Callback::new(move |()| {
                                            let remove_at = index.get_untracked();
                                            attachments.update(|a| {
                                                a.get_or_insert_default().remove(remove_at);
                                            });
                                        })
                                    />
                                }
                            }}
                        />
                        <div class="flex gap-2">
                            <input
                                class=r#"bg-background w-full text-sm p-3 rounded-lg shadow-sm"#
                                type="file"
                                name="attachments"
                                multiple
                                node_ref=attachments_ref
                            />
                        </div>
                    </div>
                </div>

                <div class="grid">
                    <label class=r#"block mb-1 text-sm font-medium text-text"#>
                        "Illustration (Max 16 MiB)"
                    </label>
                    <div class="grid gap-2">
                        {move || {
                            let illustration_signal_value = illustration.get();
                            if let Some(illustr) = illustration_signal_value {
                                let id = illustr.id.clone();
                                view! {
                                    <FileTooltip
                                        file_name=illustr.file_name.clone()
                                        id=illustr.id.clone()
                                        on_download=format!("/file/{}", id)
                                        on_remove=Callback::new(move |()| {
                                            illustration.set(None);
                                        })
                                    />
                                }.into_any()
                            } else {
                                "".into_any()
                            }
                        }}
                        <input
                            class=r#"bg-background w-full text-sm p-3 rounded-lg shadow-sm"#
                            type="file"
                            name="illustration"
                            node_ref=illustration_ref
                        />
                    </div>
                </div>

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
                            let event_id = event_id.get_untracked();
                            let name = name.get_untracked();
                            let description = description.get_untracked().unwrap_or_default();
                            let category = category.get_untracked().unwrap_or_default();
                            let difficulty = difficulty.get_untracked();
                            let points = points.get_untracked();
                            let flag = flag.get_untracked();
                            let visible_to_groups = visible_to_groups.get_untracked().join(",");
                            let vm_ids = vm_ids.get_untracked();
                            let hints = create_hints.get_untracked().unwrap_or_default().into_iter().map(DbHint::from).collect::<Vec<DbHint>>();

                            let attachments_ref = attachments_ref.get_untracked();
                            let illustration_ref = illustration_ref.get_untracked();

                            spawn_local(async move {
                                if let Some(fd) = build_multi_file_form_data(attachments_ref) {
                                    if let Ok(api_result) = upload_files(fd.into()).await {
                                        attachments.set(Some(api_result.details))
                                    }
                                }

                                if let Some(fd) = build_single_file_form_data(illustration_ref) {
                                    if let Ok(api_result) = upload_illustration(fd.into()).await {
                                        illustration.set(Some(api_result.details));
                                    }
                                }

                                let attachments = attachments.get_untracked();
                                let illustration = illustration.get_untracked();

                                if let Ok(ApiResult { result, .. }) = crate::server::admin::api::challenge(crate::server::admin::ChallengeAction::Create {
                                        event_id,
                                        name,
                                        description,
                                        category,
                                        difficulty,
                                        points,
                                        flag,
                                        visible_to_groups,
                                        vm_ids,
                                        hints: hints.into(),
                                        attachments,
                                        illustration,
                                    })
                                    .await && result == ResultStatus::Success
                                {
                                    created.set(true);
                                    push_new_toast(ToastMessageType::ChallengeCreated);
                                    set_timeout(move || created.set(false), Duration::from_secs(2));
                                } else {
                                    push_new_toast(ToastMessageType::ChallengeCreateFail);
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
                view! { <Spinner component_size=ComponentSize::Big /> }
            }>
                {move || {
                    let cwa = cwa_resource.get().unwrap_or_default();
                    cwa_signal.set(cwa);

                    let events = events_resource.get().unwrap_or_default();
                    events_signal.set(events);

                    let all_hints = all_hints_resource.get().unwrap_or_default();
                    all_hints_signal.set(all_hints);

                    let user_groups = groups_resource.get().unwrap_or_default();
                    user_groups_signal.set(user_groups);

                    let categories = categories_resource.get().unwrap_or_default();
                    categories_signal.set(categories);
                    
                    let templates = challenge_templates_resource.get().unwrap_or_default();
                    templates_signal.set(templates);

                    view! {
                        <For
                            each=move || grouped_challenges.get()
                            key=|group: &(Option<String>, Vec<ChallengeWithAttachments>)| {
                                group.0.clone()
                            }
                            let(group)
                        >
                            {
                                let category = group.0.clone();
                                view! {
                                    <div class=r#"p-2 challenge-category"#>
                                        <h2 class=r#"text-2xl"#>
                                            {category.clone().unwrap_or_else(|| "Uncategorized".to_string())}
                                        </h2>

                                        <div class=r#"grid grid-cols-4 m-4 content-stretch"#>
                                            <For
                                                each=move || {
                                                    grouped_challenges.get()
                                                        .into_iter()
                                                        .find(|(cat, _)| *cat == category)
                                                        .map(|(_, challenges)| challenges)
                                                        .unwrap_or_default()
                                                }
                                                key=|cwa: &ChallengeWithAttachments| format!("{cwa:?}")
                                                let(challenge)
                                            >
                                                <div class=r#"p-2 challenge"#>
                                                    <Challenge
                                                        cwa=challenge
                                                        editing_ids
                                                        categories=categories_signal
                                                        events=events_signal
                                                        templates=templates_signal
                                                        hints=all_hints_signal
                                                        user_groups=user_groups_signal
                                                    />
                                                </div>
                                            </For>
                                        </div>
                                    </div>
                                }
                            }
                        </For>
                    }
                }}
            </Transition>
        </div>
    }
}
