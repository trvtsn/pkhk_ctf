use crate::app::RefreshUser;
use crate::components::toast::{ToastAppear, ToastMessageType};
use crate::components::utils::{Spinner, TruncatedDesc, ComponentSize};
use crate::server::db::structs::{Challenge, ChallengeWithAttachments, DbHintWithoutHint, HintWithoutHint};
use crate::server::proxmox::{ProxmoxVMInstance, ProxmoxVMTemplate};
use crate::server::{add_vm_time, destroy_vm, get_hint, restart_vm, start_vm};
use crate::server::{check_flag, db::structs::AttachmentWithoutBlob, enums::ResultStatus, structs::ApiResult};
use icondata as i;
use leptos::{prelude::*};
use leptos_icons::Icon;
use std::collections::HashMap;
use std::time::Duration;

#[component]
pub fn ChallengePopup(
    cwa: RwSignal<ChallengeWithAttachments>,
    cwa_popup: RwSignal<Option<ChallengeWithAttachments>>,
    solved_challenges: RwSignal<Vec<String>>,
    overlay_triggered: RwSignal<bool>,
    all_templates: RwSignal<Vec<ProxmoxVMTemplate>>,
    hints: RwSignal<Vec<DbHintWithoutHint>>,
    user_vms: RwSignal<Vec<ProxmoxVMInstance>>,
    hints_used: RwSignal<HashMap::<String, String>>,
    refresh_solved_challenges: RwSignal<i32>,
    refresh_user_vms: RwSignal<i32>
) -> impl IntoView {
    let flag_signal = RwSignal::new("".to_string());

    let toast_message_type = expect_context::<RwSignal<ToastMessageType>>();
    let toast_appear = expect_context::<RwSignal<ToastAppear>>();
    let incorrect = RwSignal::new(false);
    let refresh_user = expect_context::<RwSignal<RefreshUser>>();
    
    let solved = Memo::new(move |_| {
        if solved_challenges.get().contains(&cwa.get().challenge.id) { true } else { false }
    });

    let active_vm_origin_ids = Memo::new(move |_| {
        let user_vms = user_vms.get();
        user_vms.iter().map(|a| if a.running { a.origin_id } else { 0 }).collect::<Vec<u32>>()
    });

    let button_classes = Memo::new(move |_| {
        let base = r#"inline-flex items-center gap-2 rounded-lg text-white px-4 py-2 
            text-sm font-medium focus:outline-none focus:ring-2 active:scale-95 transition"#;
        if solved.get() {
            format!("{base} {}", "bg-green-600 hover:bg-green-700 focus:ring-green-400")
        } else if incorrect.get() {
            format!("{base} {}", "bg-red-600 hover:bg-red-700 focus:ring-red-400")
        } else {
            format!(
                "{base} {}",
                "bg-yale-blue-600 hover:bg-yale-blue-700 focus:ring-yale-blue-400"
            )
        }
    });

    let submit_btn_text = Memo::new(move |_| {
        if solved.get() { 
            "Solved" 
        } else if incorrect.get() { 
            "Incorrect"
        } else {
            "Submit"
        }
    });

    Effect::new(move |_| {
        if incorrect.get() {
            set_timeout(move || incorrect.set(false), Duration::from_secs(2));
        }
    });

    let check_flag_action = Action::new_local(move |(flag, challenge): &(String, Challenge)| {
        let flag = flag.clone();
        let challenge = challenge.clone();
        let challenge_points = challenge.clone().points;
        async move {
            if let Ok(ApiResult { result, details }) = check_flag(flag, challenge).await {
                if result == ResultStatus::Fail && details == "incorrect solution" {
                    incorrect.set(true);
                } else if result == ResultStatus::Success {
                    toast_appear.set(true);
                    toast_message_type.set(ToastMessageType::Custom(format!("Solved challenge +{challenge_points}p")));
                    refresh_user.update(|r| r.iteration += 1);
                    refresh_solved_challenges.update(|r| *r += 1);
                }
            }
        }
    });

    view! {
        <div
            class="absolute inset-0 z-20 flex content-center items-center justify-center rounded-lg p-4"
        >
            <div class="bg-card p-4 rounded-lg max-w-1/3 min-w-1/4">
                <div class="flex justify-between mb-4">
                    <h3 class=r#"font-bold text-3xl/8"#>{move || cwa.get().challenge.name}</h3>
                    <button 
                        class="cursor-pointer"
                        on:click=move |_| {
                            overlay_triggered.set(false);
                            cwa_popup.set(None);
                        }
                    >
                        <Icon icon=i::LuX />
                    </button>
                </div>

                {move || {
                    if let Some(illustration) = cwa.get().illustration { 
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

                <p class=r#"text-lg/8 mt-2 whitespace-pre-wrap"#>
                    {move || {
                        let description = RwSignal::new(cwa.get().challenge.description);
                        view! { <TruncatedDesc description /> }
                    }}
                </p>

                {move || {
                    let difficulty = cwa.get().challenge.difficulty;
                    view! { <Difficulty difficulty /> }
                }}
                
                <p class=r#"text-lg/8"#>
                    <b>"Points: "</b>
                    {move || cwa.get().challenge.points}
                </p>

                {move || {
                    let challenge_id = cwa.get().challenge.id;
                    let hints = hints.get().into_iter().filter(|h| h.challenge_id == challenge_id).collect::<Vec<HintWithoutHint>>();

                    if hints.is_empty() {
                        "".into_any()
                    } else {
                        let hints_used_value = hints_used.get();
                        let used_hint_ids = hints_used_value.into_iter().map(|h| h.0).collect::<Vec<String>>();

                        view! {
                            <label class=r#"block mb-1 text-sm font-medium text-text"#>"Hints"</label>
                            <div class="grid gap-2">
                                <ForEnumerate
                                    each=move || hints.clone()
                                    key=|hint: &HintWithoutHint| hint.id.clone()
                                    children={move |index, hint| {
                                        let get_hint_action = Action::new_local(move |(challenge_id, hint_id): &(String, String)| {
                                            let challenge_id = challenge_id.clone();
                                            let hint_id = hint_id.clone();
                                            async move {
                                                if let Ok(hint) = get_hint(challenge_id, hint_id).await {
                                                    hints_used.update(|map| {map.insert(hint.id, hint.hint);});
                                                    refresh_user.update(|r| r.iteration += 1);
                                                }
                                            }
                                        });

                                        let hint_id = hint.id.clone();
                                        let hint_points_penalty = hint.points_penalty;
                                        if used_hint_ids.contains(&hint.id) {
                                            view! {
                                                <div class="flex gap-2 items-center">
                                                    <label>{format!("Hint {}", index.get() + 1)}</label>
                                                    <p>{move || hints_used.get().get(&hint_id).cloned().unwrap_or_default()}</p>
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <div class="flex gap-2 items-center">
                                                    <label>{format!("Hint {}", index.get() + 1)}</label>
                                                    <button
                                                        class=r#"col-start-1 col-end-1 gap-2 items-center py-2 px-4 text-sm font-medium text-white 
                                                        rounded-lg transition focus:ring-2 focus:outline-none active:scale-95 
                                                        bg-yale-blue-600 hover:bg-yale-blue-700 focus:ring-yale-blue-400"#
                                                        on:click=move |_| {
                                                            let hint = hint.clone();
                                                            let challenge_id = cwa.get_untracked().challenge.id;
                                                            get_hint_action.dispatch((challenge_id, hint.id));
                                                        }
                                                    >
                                                        {move || {
                                                            if get_hint_action.pending().get() {
                                                                view! { <Spinner component_size=ComponentSize::Small /> }.into_any()
                                                            } else {
                                                                if hint_points_penalty != 0 {
                                                                    format!("Get (-{}p)", hint.points_penalty).into_any()
                                                                } else {
                                                                    "Get (FREE)".into_any()
                                                                }
                                                            }
                                                        }}
                                                    </button>
                                                </div>
                                            }.into_any()
                                        }
                                    }}
                                />
                            </div>
                        }.into_any()
                    }
                }}

                <div class="flex gap-2 items-center pt-2">
                    <label
                        hidden=move || solved.get()
                        for="flag"
                    >
                        <b>"Flag: "</b>
                    </label>
                    <input
                        hidden=move || solved.get()
                        class=r#"m-1 bg-white rounded-sm text-black"#
                        bind:value=flag_signal
                    />
                    <button
                        class=move || button_classes.get()
                        disabled=move || solved.get() || incorrect.get()
                        on:click=move |_| {
                            let flag = flag_signal.get();
                            let challenge = cwa.get().challenge;
                            check_flag_action.dispatch((flag, challenge));
                        }
                    >
                        {move || submit_btn_text.get()}
                    </button>
                </div>

                <div class="grid gap-2 pt-4">
                    <div class="flex gap-2 items-center">
                        <For
                            each=move || cwa.get().attachments.clone()
                            key=|a: &AttachmentWithoutBlob| a.id.clone()
                            let(a)
                        >
                            {move || a.file_name.clone()}
                            <a
                                download
                                href=move || format!("/file/{}", a.id)
                            >
                                <Icon icon=i::LuDownload />
                            </a>
                        </For>
                    </div>
                </div>

                {move || {
                    if !solved.get() {
                        let all_templates = all_templates.get();
                        let challenge = cwa.get().challenge;
                        let challenge_vm_ids = challenge.clone().vm_ids;
                        let template_ids = match challenge_vm_ids.is_some() {
                            true => challenge_vm_ids.unwrap_or_default().split(",").map(|c| c.parse::<u32>().unwrap_or_default()).collect::<Vec<u32>>(),
                            false => Vec::<u32>::new()
                        };
                        let templates = all_templates.into_iter().filter(|t| template_ids.contains(&t.id)).collect::<Vec<ProxmoxVMTemplate>>();
                        view! {
                            <div class="grid auto-rows-auto gap-2 pt-2">
                                // for each vm that the challenge has, show a button set
                                <For
                                    each=move || templates.clone()
                                    key=|template: &ProxmoxVMTemplate| template.id
                                    children=move |template| {
                                        let start_vm_action = Action::new_local(move |(template_id, challenge): &(u32, Challenge)| {
                                            let template_id = template_id.clone();
                                            let challenge = challenge.clone();
                                            async move {
                                                if let Ok(result) = start_vm(template_id, challenge).await {
                                                    toast_appear.set(true);
                                                    toast_message_type.set(ToastMessageType::Custom(result.details));
                                                    refresh_user_vms.update(|n| *n += 1);
                                                } else {
                                                    toast_appear.set(true);
                                                    toast_message_type.set(ToastMessageType::VMStartFail);
                                                }
                                            }
                                        });
                                        let restart_vm_action = Action::new_local(move |template_id: &u32| {
                                            let template_id = template_id.clone();
                                            async move {
                                                if let Ok(result) = restart_vm(template_id).await {
                                                    toast_appear.set(true);
                                                    toast_message_type.set(ToastMessageType::Custom(result.details));
                                                    refresh_user_vms.update(|n| *n += 1);
                                                } else {
                                                    toast_appear.set(true);
                                                    toast_message_type.set(ToastMessageType::VMRestartFail);
                                                }
                                            }
                                        });
                                        let add_vm_time_action = Action::new_local(move |template_id: &u32| {
                                            let template_id = template_id.clone();
                                            async move {
                                                if let Ok(result) = add_vm_time(template_id).await {
                                                    toast_appear.set(true);
                                                    toast_message_type.set(ToastMessageType::Custom(result.details));
                                                    refresh_user_vms.update(|n| *n += 1);
                                                } else {
                                                    toast_appear.set(true);
                                                    toast_message_type.set(ToastMessageType::VMAddTimeFail);
                                                }
                                            }
                                        });
                                        let destroy_vm_action = Action::new_local(move |template_id: &u32| {
                                            let template_id = template_id.clone();
                                            async move {
                                                if let Ok(result) = destroy_vm(template_id).await {
                                                    toast_appear.set(true);
                                                    toast_message_type.set(ToastMessageType::Custom(result.details));
                                                    refresh_user_vms.update(|n| *n += 1);
                                                } else {
                                                    toast_appear.set(true);
                                                    toast_message_type.set(ToastMessageType::VMDestroyFail);
                                                }
                                            }
                                        });
                                        
                                        view! {
                                            <div class="flex gap-2 items-center">
                                                <label>{template.name}</label>
                                                <Show when=move || !active_vm_origin_ids.get().contains(&template.id)>
                                                    <button
                                                        class=r#"col-start-1 col-end-1 gap-2 items-center py-2 px-4 text-sm font-medium text-white 
                                                        rounded-lg transition focus:ring-2 focus:outline-none active:scale-95 
                                                        bg-yale-blue-600 hover:bg-yale-blue-700 focus:ring-yale-blue-400"#
                                                        disabled=move || start_vm_action.pending().get()
                                                        on:click=move |_| {
                                                            let challenge = cwa.get().challenge;
                                                            start_vm_action.dispatch((template.id, challenge));
                                                        }
                                                    >
                                                        {move || {
                                                            if start_vm_action.pending().get() {
                                                                view! { <Spinner component_size=ComponentSize::Small /> }.into_any()
                                                            } else {
                                                                "Start VM".into_any()
                                                            }
                                                        }}
                                                    </button>
                                                </Show>

                                                <Show when=move || active_vm_origin_ids.get().contains(&template.id)>
                                                    <button
                                                        class=r#"col-start-2 col-end-2 gap-2 items-center py-2 px-4 text-sm font-medium text-white 
                                                        rounded-lg transition focus:ring-2 focus:outline-none active:scale-95 
                                                        bg-yale-blue-600 hover:bg-yale-blue-700 focus:ring-yale-blue-400"#
                                                        disabled=move || restart_vm_action.pending().get()
                                                        on:click=move |_| {
                                                            restart_vm_action.dispatch(template.id);
                                                        }
                                                    >
                                                        {move || {
                                                            if restart_vm_action.pending().get() {
                                                                view! { <Spinner component_size=ComponentSize::Small /> }.into_any()
                                                            } else {
                                                                "Restart VM".into_any()
                                                            }
                                                        }}
                                                    </button>

                                                    <button
                                                        class=r#"col-start-3 col-end-3 gap-2 items-center py-2 px-4 text-sm font-medium text-white 
                                                        rounded-lg transition focus:ring-2 focus:outline-none active:scale-95 
                                                        bg-yale-blue-600 hover:bg-yale-blue-700 focus:ring-yale-blue-400"#
                                                        disabled=move || add_vm_time_action.pending().get()
                                                        on:click=move |_| {
                                                            add_vm_time_action.dispatch(template.id);
                                                        }
                                                    >
                                                        {move || {
                                                            if add_vm_time_action.pending().get() {
                                                                view! { <Spinner component_size=ComponentSize::Small /> }.into_any()
                                                            } else {
                                                                "Add Time (+30 min)".into_any()
                                                            }
                                                        }}
                                                    </button>

                                                    <button
                                                        class=r#"col-start-4 col-end-4 gap-2 items-center py-2 px-4 text-sm font-medium text-white 
                                                        rounded-lg transition focus:ring-2 focus:outline-none active:scale-95 
                                                        bg-yale-blue-600 hover:bg-yale-blue-700 focus:ring-yale-blue-400"#
                                                        disabled=move || destroy_vm_action.pending().get()
                                                        on:click=move |_| {
                                                            destroy_vm_action.dispatch(template.id);
                                                        }
                                                    >
                                                        {move || {
                                                            if destroy_vm_action.pending().get() {
                                                                view! { <Spinner component_size=ComponentSize::Small /> }.into_any()
                                                            } else {
                                                                "Destroy VM".into_any()
                                                            }
                                                        }}
                                                    </button>
                                                </Show>
                                            </div>
                                        }
                                    }
                                />
                            </div>
                        }.into_any()
                    } else {
                        "".into_any()
                    }
                }}
            </div>
        </div>
    }
}

#[component]
pub fn Difficulty(difficulty: i8) -> impl IntoView {
    view! {
        {move || {
            view! {
                <div
                    class=r#"difficulty"#
                    role="img"
                    aria-label=format!("Difficulty: {} of 5", difficulty)
                >
                    <span class=r#"label"#>
                        <b class=r#"text-lg/8"#>"Difficulty: "</b>
                        {"⭐".repeat(difficulty as usize)}
                    </span>
                </div>
            }
        }}
    }
}
