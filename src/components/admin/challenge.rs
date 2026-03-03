use crate::components::toast::{ToastAppear, ToastMessageType};
use crate::components::utils::TruncatedDesc;
use crate::pages::admin::challenges::Hint;
use crate::server::admin::upload_illustration;
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
    let illustration_signal = RwSignal::new(illustration.clone());

    let event_id_edit = RwSignal::new(event_id);
    let name_edit = RwSignal::new(name.clone());
    let description_edit = RwSignal::new(description.clone());
    let category_edit = RwSignal::new(category.clone());
    let difficulty_edit = RwSignal::new(difficulty);
    let points_edit = RwSignal::new(points);
    let attachments_edit = RwSignal::new(attachments.clone());
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
    let toast_message_type = expect_context::<RwSignal<ToastMessageType>>();
    let toast_appear = expect_context::<RwSignal<ToastAppear>>();

    view! {
        <div class=r#"content-center p-4 rounded-lg bg-card hover:bg-card-hover text-text break-all"#>
            <Show when=move || !editing.get() && !deleted.get()>
                <h3 class=r#"font-bold text-3xl/8 mb-4"#>{move || name_signal.get().clone()}</h3>
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
                    {move || id_signal.get().clone()}
                </p>
                <p class=r#"text-lg/8"#>
                    <b>"Event ID: "</b>
                    {move || event_id_signal.get().clone()}
                </p>
                <p class=r#"text-lg/8 whitespace-pre-wrap"#>
                    <TruncatedDesc description=description_signal />
                </p>
                <Difficulty rating=difficulty_signal.get() />
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
                                <option value=e
                                    .id
                                    .clone()>{e.name.clone()} " (ID: " {e.id.clone()} ")"</option>
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
                                children=move |category| {
                                    let selected = category_edit.get().map(|c| c == category).unwrap_or(false);
                                    view! {
                                        <option 
                                            value=category.clone()
                                            selected=selected
                                        >
                                            {category.clone()}
                                        </option>
                                    }
                                }
                            />
                            <option value="__new__">"-- Add New --"</option>
                        </select>
                        <Show when=move || category_add_new_selected.get()>
                            <input
                                class=r#"bg-background py-2 px-3 w-full text-sm rounded-md border border-input-border 
                                focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                                type="text"
                                id="action_create_category_input"
                                value=""
                                on:change=move |ev: Event| {
                                    let value = event_target_value(&ev);
                                    category_edit.set(Some(value));
                                }
                            />
                        </Show>
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
                            <option 
                                value="all"
                                selected=move || {
                                    visible_to_groups_edit
                                        .get().split(",")
                                        .map(String::from)
                                        .collect::<Vec<String>>()
                                        .contains(&"all".to_string())
                                }
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
                                            let selected = visible_to_groups_edit
                                                .get().split(",")
                                                .map(String::from)
                                                .collect::<Vec<String>>()
                                                .contains(&group);

                                            view! {
                                                <option 
                                                    value=group.clone()
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
                            {move || {
                                let templates = templates.get();
                                view! {
                                    <For
                                        each=move || templates.clone()
                                        key=|template: &ProxmoxVMTemplate| template.id
                                        children=move |template| {
                                            let selected = proxmox_vm_id_edit
                                                .get()
                                                .unwrap_or_default()
                                                .split(",")
                                                .map(String::from)
                                                .collect::<Vec<String>>()
                                                .contains(&template.id.to_string());

                                            view! {
                                                <option 
                                                    value={template.id}
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
                                            value: RwSignal::new(h.hint),
                                            points_penalty: RwSignal::new(Some(h.points_penalty))
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
                    </div>

                    <div class="grid">
                        <label class=r#"block mb-1 text-sm font-medium"#>"Attachments"</label>
                        <div class="grid gap-2">
                            <ForEnumerate
                                each=move || attachments_edit.get()
                                key=|a: &AttachmentWithoutBlob| a.id.clone()
                                children={move |index, a| {
                                    let show_tooltip = RwSignal::new(false);
                                    let id = a.id.clone();
                                    let file_name = a.file_name.clone();
                                    view! {  
                                        <div class="flex gap-2 items-center">
                                            <span
                                                class="relative inline-block"
                                                on:mouseenter=move |_| show_tooltip.set(true)
                                                on:mouseleave=move |_| show_tooltip.set(false)
                                                // keyboard focus
                                                on:focus=move |_| show_tooltip.set(true)
                                                on:blur=move |_| show_tooltip.set(false)
                                                tabindex="0"
                                            >
                                                {file_name}
                                                <Show when=move || show_tooltip.get()>
                                                    <div
                                                        role="tooltip"
                                                        class=r#"absolute left-1/2 bottom-full -translate-x-1/2 whitespace-nowrap 
                                                            rounded p-1 text-xs bg-card shadow-sm z-1"#
                                                    >
                                                        {format!("ID: {}", a.id)}
                                                    </div>
                                                </Show>
                                            </span>
                                            <a
                                                download
                                                href=move || format!("/file/{}", id)
                                            >
                                                <Icon icon=i::LuDownload />
                                            </a>
                                            <button 
                                                class="cursor-pointer"
                                                on:click=move |_| {
                                                    let remove_at = index.get();

                                                    attachments_edit.update(|a| {
                                                        a.remove(remove_at);
                                                    });
                                                } 
                                            >
                                                <Icon icon=i::LuX />
                                            </button>
                                        </div>
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
                                    let show_tooltip = RwSignal::new(false);
                                    let id = illustration.id.clone();
                                    view! {
                                        <div class="flex gap-2 items-center">
                                            <span
                                                class="relative inline-block"
                                                on:mouseenter=move |_| show_tooltip.set(true)
                                                on:mouseleave=move |_| show_tooltip.set(false)
                                                on:focus=move |_| show_tooltip.set(true)
                                                on:blur=move |_| show_tooltip.set(false)
                                                tabindex="0"
                                            >
                                                {move || illustration.file_name.clone()}
                                                <Show when=move || show_tooltip.get()>
                                                    <div
                                                        role="tooltip"
                                                        class=r#"absolute left-1/2 bottom-full -translate-x-1/2 whitespace-nowrap 
                                                            rounded p-1 text-xs bg-card shadow-sm z-1"#
                                                    >
                                                        {format!("ID: {}", illustration.id)}
                                                    </div>
                                                </Show>
                                            </span>
                                            <a
                                                download
                                                href=move || format!("/file/{}", id)
                                            >
                                                <Icon icon=i::LuDownload />
                                            </a>
                                            <button 
                                                class="cursor-pointer"
                                                on:click=move |_| {
                                                    illustration_edit.set(None);
                                                } 
                                            >
                                                <Icon icon=i::LuX />
                                            </button>
                                        </div>
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
                            spawn_local(async move {
                                editing.set(false);
                                deleting.set(false);
                            });
                        }
                    >
                        "Cancel"
                    </button>
                </Show>
                <Show when=move || editing.get()>
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
                            let vm_ids = proxmox_vm_id_edit.get();
                            let hints = hints_edit.get().into_iter().map(|h| {
                                let mut hint = Into::<DbHint>::into(h); 
                                hint.challenge_id = challenge_id.clone(); 
                                hint
                            }).collect::<Vec<DbHint>>();
                            if editing.get() {
                                spawn_local(async move {
                                    if let Some(att_el) = attachments_ref.get() {
                                        if let Some(files) = att_el.files() {
                                            if files.length() > 0 {
                                                let fd = FormData::new().unwrap();
                                                for i in 0..files.length() {
                                                    let file = files.get(i).unwrap();
                                                    fd.append_with_blob_and_filename("file", &file, &file.name()).unwrap();
                                                }

                                                if let Ok(api_result) = upload_files(fd.into()).await {
                                                    attachments_edit.set(api_result.details);
                                                }
                                            }
                                        }
                                    }

                                    if let Some(illustr_el) = illustration_ref.get() {
                                        if let Some(files) = illustr_el.files() {
                                            if files.length() > 0 {
                                                let file = files.get(0).unwrap();
                                                let fd = FormData::new().unwrap();
                                                fd.append_with_blob_and_filename("file", &file, &file.name()).unwrap();

                                                if let Ok(api_result) = upload_illustration(fd.into()).await {
                                                    illustration_edit.set(Some(api_result.details))
                                                }
                                            }
                                        }
                                    }

                                    let attachments = if attachments_edit.get().is_empty() { None } else { Some(attachments_edit.get()) };
                                    let illustration = illustration_edit.get();

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
                                        toast_appear.set(true);
                                        toast_message_type.set(ToastMessageType::ChallengeEdited);
                                        refresh.update(|n| *n += 1);
                                        editing.set(false);
                                        event_id_signal.set(event_id);
                                        name_signal.set(name);
                                        description_signal.set(description);
                                        category_signal.set(category);
                                        difficulty_signal.set(difficulty);
                                        points_signal.set(points);
                                    }
                                });
                            } else {
                                editing.set(true)
                            }
                        }
                    >
                        "Confirm Edit"
                    </button>
                </Show>
                <Show when=move || deleting.get()>
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
                                        toast_appear.set(true);
                                        toast_message_type.set(ToastMessageType::ChallengeDeleted);
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
                        "Confirm Delete"
                    </button>
                </Show>
                <Show when=move || !editing.get() && !deleting.get()>
                    <button
                        type="button"
                        hidden=move || deleting.get()
                        class=r#"inline-flex gap-2 items-center py-2 px-4 text-sm font-medium text-white 
                        rounded-lg transition focus:ring-2 focus:outline-none active:scale-95 
                        bg-yale-blue-600 hover:bg-yale-blue-700 focus:ring-yale-blue-400"#
                        on:click=move |_| {
                            if !editing.get() {
                                editing.set(true)
                            }
                        }
                    >
                        "Edit"
                    </button>
                    <button
                        hidden=move || editing.get()
                        class=r#"inline-flex items-center py-2 px-4 ml-auto text-sm font-semibold text-white 
                        bg-red-600 rounded-md shadow-sm hover:bg-red-500 focus:ring-2 focus:outline-none 
                        focus:ring-yale-blue-500"#
                        on:click=move |_| {
                            if !deleting.get() {
                                deleting.set(true);
                            }
                        }
                    >
                        "Delete"
                    </button>
                </Show>
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
