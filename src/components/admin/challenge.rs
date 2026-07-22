use crate::components::toast::{ToastMessageType, push_new_toast};
use crate::components::utils::{TruncatedDesc, FileTooltip, Difficulty};
use crate::pages::admin::challenges::Hint;
use crate::server::admin::api::{upload_files, upload_illustration};
use crate::server::db::structs::DbHint;
use crate::server::proxmox::ProxmoxVMTemplate;
use crate::server::{db::{self, structs::{AttachmentWithoutBlob, Challenge, ChallengeWithAttachments}}, enums::ResultStatus, structs::ApiResult};
use crate::utils::{action_btn_text, build_multi_file_form_data, build_single_file_form_data, collect_selected_options, csv_contains};
use icondata as i;
use leptos::{prelude::*, task::spawn_local, wasm_bindgen::JsCast, web_sys::{Event, HtmlSelectElement}};
use leptos_icons::Icon;

/// Admin-facing challenge card in the admin Challenges section.
/// Inline editing for name, category, points, attachments, hints, etc.
#[component]
pub fn Challenge(
    cwa: ChallengeWithAttachments,
    editing_ids: RwSignal<Vec<String>>,
    categories: RwSignal<Vec<String>>,
    events: RwSignal<Vec<db::structs::Event>>,
    templates: RwSignal<Vec<ProxmoxVMTemplate>>,
    hints: RwSignal<Vec<DbHint>>,
    user_groups: RwSignal<Vec<String>>
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
    let visible_to_groups_signal = RwSignal::new(visible_to_groups.clone());
    let attachments_signal = RwSignal::new(attachments.clone());
    let illustration_signal = RwSignal::new(illustration.clone());
    let proxmox_vm_id_signal = RwSignal::new(vm_ids.clone());

    let event_id_edit = RwSignal::new(event_id);
    let name_edit = RwSignal::new(name);
    let description_edit = RwSignal::new(description);
    let category_edit = RwSignal::new(category);
    let difficulty_edit = RwSignal::new(difficulty);
    let points_edit = RwSignal::new(points);
    let attachments_edit = RwSignal::new(attachments);
    let flag_edit = RwSignal::new("".to_string());
    let illustration_edit = RwSignal::new(illustration);
    let visible_to_groups_edit = RwSignal::new(visible_to_groups);
    let proxmox_vm_id_edit = RwSignal::new(vm_ids);
    let hints_edit = RwSignal::new(vec![Hint::new("1".to_string(), "")]);

    let attachments_ref = NodeRef::new();
    let illustration_ref = NodeRef::new();

    let next_hint_id = RwSignal::new(1_usize);
    let category_add_new_selected = RwSignal::new(false);
    let editing = RwSignal::new(false);
    let deleting = RwSignal::new(false);
    let deleted = RwSignal::new(false);

    Effect::new(move |_| {
        let id = id_signal.get_untracked();
        if editing.get() {
            editing_ids.update(|ids| {
                if !ids.contains(&id) {
                    ids.push(id);
                }
            });
        } else {
            editing_ids.update(|ids| ids.retain(|i| i != &id));
        }
    });

    let delete_submit_btn_text = action_btn_text(move || deleting.get(), "Confirm Delete", "Delete");
    let edit_submit_btn_text = action_btn_text(move || editing.get(), "Confirm Edit", "Edit");

    let any_changes_made = Memo::new(move |_| {
        let challenge_id = id_signal.get();
        
        let hints_edit = hints_edit.get().into_iter().map(|h| {
            let mut hint = DbHint::from(h); 
            hint.challenge_id = challenge_id.clone(); 
            hint
        }).collect::<Vec<DbHint>>();

        let initial_hints_value = vec![Hint::new("1".to_string(), "")].into_iter().map(|h| {
            let mut hint = DbHint::from(h); 
            hint.challenge_id = challenge_id.clone(); 
            hint
        }).collect::<Vec<DbHint>>();

        if event_id_signal.get() == event_id_edit.get() &&
            name_signal.get() == name_edit.get() && 
            description_signal.get() == description_edit.get() && 
            category_signal.get() == category_edit.get() &&
            difficulty_signal.get() == difficulty_edit.get() &&
            points_signal.get() == points_edit.get() &&
            flag_edit.get().is_empty() &&
            visible_to_groups_signal.get() == visible_to_groups_edit.get() &&
            proxmox_vm_id_signal.get() == proxmox_vm_id_edit.get() &&
            initial_hints_value == hints_edit &&
            attachments_signal.get() == attachments_edit.get() &&
            illustration_signal.get() == illustration_edit.get()
        { false } else { true }
    });

    view! {
        <div class=r#"content-center p-4 rounded-lg bg-card hover:bg-card-hover text-text break-all"#>
            <Show when=move || !editing.get() && !deleted.get()>
                <h3 class=r#"font-bold text-3xl/8 mb-4"#>{move || name_signal.get()}</h3>
                {move || {
                    if let Some(illustration) = illustration_signal.get() { 
                        view! {
                            <div class="flex justify-center m-auto mb-4">
                                <img 
                                    src=move || format!("/image/{}", illustration.id) 
                                    class=r#"shadow-sm"#
                                />
                            </div>
                        }.into_any()
                    } else {
                        "".into_any()
                    }
                }}
                <p class=r#"text-lg/8"#>
                    <b>"ID: "</b>
                    {move || id_signal.get()}
                </p>
                <p class=r#"text-lg/8"#>
                    <b>"Event ID: "</b>
                    {move || event_id_signal.get()}
                </p>
                <p class=r#"text-lg/8 whitespace-pre-wrap"#>
                    <TruncatedDesc description=description_signal />
                </p>
                <Difficulty difficulty=difficulty_signal.get() />
                <p class=r#"text-lg/8"#>
                    <b>"Points: "</b>
                    {points_signal.get()}
                </p>
                <p class=r#"text-lg/8"#>
                    <b>"Visible To Groups: "</b>
                    {
                        let visible_to_groups = visible_to_groups_signal.get().replace(",", ", ");
                        if visible_to_groups.clone().is_empty() {
                            view! {
                                <i>"None"</i>
                            }.into_any()
                        } else {
                            visible_to_groups.into_any()
                        }
                    }
                </p>
            </Show>

            <Show when=move || editing.get()>
                <div class="grid gap-3">
                    <div class="grid">
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
                                <option value=e.id>{e.name.clone()} " (ID: " {e.id.clone()} ")"</option>
                            </For>
                        </select>
                    </div>

                    <div class="grid">
                        <label class=r#"block mb-1 text-sm font-medium"#>"Name"</label>
                        <input
                            class=r#"bg-background py-2 px-3 w-full text-sm rounded-md border border-input-border 
                            focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                            name="name"
                            value=move || name_signal.get()
                            bind:value=name_edit
                        />
                    </div>

                    <div class="grid">
                        <label class=r#"block mb-1 text-sm font-medium"#>"Description"</label>
                        <textarea
                            class=r#"bg-background py-2 px-3 w-full text-sm rounded-md border border-input-border 
                            focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                            name="description"
                            prop:value=move || description_signal.get().unwrap_or_default()
                            on:input=move |ev: Event| {
                                let value = event_target_value(&ev);
                                description_edit.set(Some(value));
                            }
                        />
                    </div>

                    <div class="grid">
                        <label class=r#"block mb-1 text-sm font-medium"#>"Category"</label>
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
                                    category_edit.set(Some(value));
                                }
                            }
                        >
                            <option value="">"-- Select Category --"</option>
                            <For
                                each=move || categories.get()
                                key=|category: &String| category.clone()
                                children=move |category| {
                                    let selected = category_edit.get().map(|c| c == category).unwrap_or(false);
                                    view! {
                                        <option 
                                            value=category
                                            selected=selected
                                        >
                                            {category.clone()}
                                        </option>
                                    }
                                }
                            />
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
                    </div>

                    <div class="grid">
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
                    </div>

                    <div class="grid">
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
                    </div>

                    <div class="grid">
                        <label class=r#"block mb-1 text-sm font-medium"#>"Flag"</label>
                        <input
                            class=r#"bg-background py-2 px-3 w-full text-sm rounded-md border border-input-border 
                            focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                            name="flag"
                            bind:value=flag_edit
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
                                let sel = match ev.target() {
                                    Some(sel) => sel.unchecked_into::<HtmlSelectElement>(),
                                    None => { push_new_toast(ToastMessageType::ErrorOccurred); return }
                                };
                                let picked = collect_selected_options(&sel);

                                visible_to_groups_edit.set(picked.join(","));
                            }
                        >
                            <option 
                                value="all"
                                selected=move || csv_contains(&visible_to_groups_edit.get(), "all")
                            >
                                "All"
                            </option>
                            {move || {
                                let groups = user_groups.get();
                                view! {
                                    <For
                                        each=move || groups.clone()
                                        key=|group: &String| group.clone()
                                        children=move |group| {
                                            let selected = csv_contains(&visible_to_groups_edit.get(), &group);

                                            view! {
                                                <option 
                                                    value=group
                                                    selected=selected
                                                >
                                                    {group.clone()}
                                                </option>
                                            }
                                        }
                                    />
                                }
                            }}
                        </select>
                    </div>

                    <div class="grid">
                        <label class=r#"block mb-1 text-sm font-medium"#>"Proxmox VM IDs (Optional)"</label>
                        <select
                            class=r#"bg-background py-2 px-3 w-full text-sm rounded-md border border-input-border 
                            focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                            name="vm_ids"
                            multiple=true
                            on:change=move |ev: Event| {
                                let sel = match ev.target() {
                                    Some(target) => target.unchecked_into::<HtmlSelectElement>(),
                                    None => return
                                };
                                let picked = collect_selected_options(&sel);

                                proxmox_vm_id_edit.set(Some(picked.join(",")));
                            }
                        >
                            {move || {
                                view! {
                                    <For
                                        each=move || templates.get()
                                        key=|template: &ProxmoxVMTemplate| template.id
                                        children=move |template| {
                                            let selected = csv_contains(
                                                &proxmox_vm_id_edit.get().unwrap_or_default(),
                                                &template.id.to_string()
                                            );

                                            view! {
                                                <option 
                                                    value=template.id
                                                    selected=selected
                                                >
                                                    {format!("{} (VM ID: {})", template.name, template.id)}
                                                </option>
                                            }
                                        }
                                    />
                                }
                            }}
                        </select>
                    </div>

                    <div class="grid">
                        {move || {
                            let challenge_id = id_signal.get();
                            let hints = hints.get().into_iter().filter(|h| h.challenge_id == challenge_id).collect::<Vec<DbHint>>();
                            if !hints.is_empty() {
                                hints_edit.set(
                                    hints.into_iter()
                                        .map(|h| crate::pages::admin::challenges::Hint {
                                            id: h.id,
                                            value: ArcRwSignal::new(h.hint),
                                            points_penalty: ArcRwSignal::new(Some(h.points_penalty))
                                        })
                                        .collect::<Vec<crate::pages::admin::challenges::Hint>>()
                                );
                            }

                            view! {
                                <label class=r#"block mb-1 text-sm font-medium text-text"#>"Hints"</label>
                                <ForEnumerate
                                    each=move || hints_edit.get()
                                    key=|hint: &Hint| hint.id.clone()
                                    children={move |index, hint| {
                                        let value = hint.value.clone();
                                        let value_input = hint.value.clone();
                                        let penalty = hint.points_penalty.clone();
                                        let penalty_input = hint.points_penalty.clone();
                                        view! {
                                            <div class="flex gap-2">
                                                <input
                                                    class=r#"bg-background py-2 px-3 w-full text-sm rounded-md border border-input-border
                                                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                                                    name="hint"
                                                    prop:value=move || value.get()
                                                    on:input=move |ev| {
                                                        value_input.set(event_target_value(&ev));
                                                    }
                                                />
                                                <input
                                                    class="w-1/2"
                                                    type="number"
                                                    min=0
                                                    prop:value=move || penalty.get()
                                                    on:input=move |ev| {
                                                        let value = event_target_value(&ev);
                                                        penalty_input.set(Some(value.parse::<u32>().unwrap_or_default()));
                                                    }
                                                    placeholder="0"
                                                />
                                                <button 
                                                    class="cursor-pointer"
                                                    on:click=move |_| {
                                                        hints_edit.update(|vec| {
                                                            next_hint_id.set(next_hint_id.get_untracked() + 1);
                                                            vec.push(Hint::new(next_hint_id.get_untracked().to_string(), ""));
                                                        });
                                                    }
                                                >
                                                    <Icon icon=i::LuPlus />
                                                </button>
                                                <Show when=move || index.get() != 0>
                                                    <button 
                                                        class="cursor-pointer"
                                                        on:click=move |_| {
                                                            let remove_at = index.get_untracked();

                                                            hints_edit.update(|vec| {
                                                                vec.remove(remove_at);
                                                            });
                                                        } 
                                                    >
                                                        <Icon icon=i::LuX />
                                                    </button>
                                                </Show>
                                            </div>
                                        }
                                    }}
                                />
                            }
                        }}
                    </div>

                    <div class="grid">
                        <label class=r#"block mb-1 text-sm font-medium"#>"Attachments"</label>
                        <div class="grid gap-2">
                            <ForEnumerate
                                each=move || attachments_edit.get()
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
                                                attachments_edit.update(|a| {
                                                    a.remove(remove_at);
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
                        <label class=r#"block mb-1 text-sm font-medium"#>"Illustration"</label>
                        <div class="grid gap-2">
                            {move || {
                                let illustration = illustration_edit.get();
                                if let Some(illustration) = illustration {
                                    let id = illustration.id.clone();
                                    view! {
                                        <FileTooltip
                                            file_name=illustration.file_name.clone()
                                            id=illustration.id.clone()
                                            on_download=format!("/file/{}", id)
                                            on_remove=Callback::new(move |()| {
                                                illustration_edit.set(None);
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
                </div>
            </Show>

            <div class=r#"flex flex-row-reverse gap-3 mt-2"#>
                <Show when=move || editing.get() || deleting.get()>
                    <button
                        class=r#"py-2 px-4 text-sm rounded-md border border-input-border hover:bg-background-hover"#
                        on:click=move |_| {
                            editing.set(false);
                            deleting.set(false);
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
                            let challenge_id = id_signal.get_untracked();
                            let event_id = event_id_edit.get_untracked();
                            let name = name_edit.get_untracked();
                            let description = description_edit.get_untracked();
                            let category = category_edit.get_untracked();
                            let difficulty = difficulty_edit.get_untracked();
                            let points = points_edit.get_untracked();
                            let flag = flag_edit.get_untracked();
                            let visible_to_groups = visible_to_groups_edit.get_untracked();
                            let vm_ids = proxmox_vm_id_edit.get_untracked();
                            let hints = hints_edit.get_untracked().into_iter().map(|h| {
                                let mut hint = DbHint::from(h); 
                                hint.challenge_id = challenge_id.clone(); 
                                hint
                            }).collect::<Vec<DbHint>>();

                            let attachments_ref = attachments_ref.get_untracked();
                            let illustration_ref = illustration_ref.get_untracked();

                            if editing.get() {
                                spawn_local(async move {
                                    if let Some(fd) = build_multi_file_form_data(attachments_ref) {
                                        if let Ok(api_result) = upload_files(fd.into()).await {
                                            attachments_edit.set(api_result.details);
                                        }
                                    }

                                    if let Some(fd) = build_single_file_form_data(illustration_ref) {
                                        if let Ok(api_result) = upload_illustration(fd.into()).await {
                                            illustration_edit.set(Some(api_result.details))
                                        }
                                    }

                                    // using .get_untracked as we're inside an on:click event handler and don't require an active subscription to these values
                                    // consider changing the .get()'s of the initial values above
                                    let attachments = if attachments_edit.get_untracked().is_empty() { None } else { Some(attachments_edit.get_untracked()) };
                                    let illustration = illustration_edit.get_untracked();

                                    if !any_changes_made.get_untracked() {
                                        editing.set(false);
                                        push_new_toast(ToastMessageType::NoChangesMade);
                                    } else {
                                        if let Ok(ApiResult { result, .. }) = crate::server::admin::api::challenge(crate::server::admin::ChallengeAction::Edit {
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
                                            push_new_toast(ToastMessageType::ChallengeEdited);
                                            editing.set(false);
                                        } else {
                                            push_new_toast(ToastMessageType::ChallengeEditFail);
                                        }
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
                        if deleting.get_untracked() {
                            let challenge_id = id_signal.get_untracked();
                            spawn_local(async move {
                                tracing::debug!("deleting challenge ID: {challenge_id}");
                                if let Ok(ApiResult { result, .. }) = crate::server::admin::api::challenge(crate::server::admin::ChallengeAction::Delete {
                                        id: challenge_id,
                                    })
                                    .await && result == ResultStatus::Success
                                {
                                    push_new_toast(ToastMessageType::ChallengeDeleted);
                                    deleting.set(false);
                                    deleted.set(true);
                                } else {
                                    push_new_toast(ToastMessageType::ChallengeDeleteFail);
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
