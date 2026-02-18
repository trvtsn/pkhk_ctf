use std::collections::HashMap;

use crate::app::RefreshUser;
use crate::components::utils::TruncatedDesc;
use crate::server::db::structs::{ChallengeWithAttachments, DbHintWithoutHint, HintWithoutHint, HintsUsed};
use crate::server::proxmox::{ProxmoxVMInstance, ProxmoxVMTemplate};
use crate::server::{add_vm_time, destroy_vm, get_all_hints_without_hints, get_challenge_hints_without_hints, get_hint, restart_vm, start_vm};
use crate::server::{check_flag, db::structs::AttachmentWithoutBlob, enums::ResultStatus, structs::ApiResult};
use icondata as i;
use leptos::{prelude::*, task::spawn_local};
use leptos_icons::Icon;
use leptos_use::{use_timeout_fn, UseTimeoutFnReturn};

#[component]
pub fn ChallengePopup(
    cwa_popup: RwSignal<ChallengeWithAttachments>,
    solved_challenges: RwSignal<Vec<String>>,
    overlay_triggered: RwSignal<bool>,
    user_vms: RwSignal<Vec<ProxmoxVMInstance>>,
    all_templates: RwSignal<Vec<ProxmoxVMTemplate>>,
    hints: RwSignal<Vec<DbHintWithoutHint>>,
    hints_used: RwSignal<Vec<HintsUsed>>,
    refresh: RwSignal<i32>
) -> impl IntoView {
    let description_signal = RwSignal::new(None);
    let flag_signal = RwSignal::new("".to_string());
    let hints_bought_signal = RwSignal::new(HashMap::<String, String>::new());

    let solved = RwSignal::new(false);
    let incorrect = RwSignal::new(false);

    let refresh_user = expect_context::<RwSignal<RefreshUser>>();
    
    let card_classes = Memo::new(move |_| {
        let base = "absolute inset-0 z-20 flex content-center items-center justify-center rounded-lg p-4";
        if overlay_triggered.get() {
            base.to_string()
        } else {
            format!("{base} hidden")
        }
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
        if solved_challenges.get().contains(&cwa_popup.get().challenge.id) { 
            solved.set(true);
            "Solved" 
        } else if incorrect.get() { 
            "Incorrect"
        } else {
            "Submit"
        }
    });

    Effect::new(move |_| {
        let challenge_id = cwa_popup.get_untracked().challenge.id.clone();
        let hints_used = hints_used.get(); // effect re-runs only when hints_used changes

        for hint_used in hints_used {
            let hint_id = hint_used.hint_id.clone();

            if hints_bought_signal.get_untracked().contains_key(&hint_id) {
                continue;
            }

            let challenge_id = challenge_id.clone();
            let hint_id_clone = hint_id.clone();
            spawn_local(async move {
                if let Ok(hint) = get_hint(challenge_id, hint_id_clone.clone()).await {
                    hints_bought_signal.update(|map| {
                        map.insert(hint.id.clone(), hint.hint);
                    });
                }
            });
        }
    });

    let UseTimeoutFnReturn { start, stop, .. } =
        use_timeout_fn(move |_: ()| {
            // runs after the delay on the client
            incorrect.set(false);
        }, 2000.0);

    view! {
        <div
            class=move || card_classes.get()
        >
            <div class="bg-card p-4 rounded-lg">
                <div class="flex justify-end">
                    <button 
                        class="cursor-pointer"
                        on:click=move |_| overlay_triggered.set(false)
                    >
                        <Icon icon=i::LuX />
                    </button>
                </div>

                <Transition fallback=move || {
                    view! { <div>"Loading..."</div> }
                }>
                    {move || {
                        let illustration = cwa_popup.get().illustration;

                        if let Some(illustration_id) = illustration { 
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
                <h3 class=r#"font-bold text-3xl/8"#>{move || cwa_popup.get().challenge.name.clone()}</h3>
                <p class=r#"text-lg/8"#>
                <Transition fallback=move || {
                    view! { <div>"..."</div> }
                }>
                    {move || {
                        description_signal.set(cwa_popup.get().challenge.description);
                        view! { <TruncatedDesc description=description_signal /> }
                    }}
                </Transition>
                    
                </p>
                <Difficulty difficulty=cwa_popup />
                <p class=r#"text-lg/8"#>
                    <b>"Points: "</b>
                    {move || cwa_popup.get().challenge.points}
                </p>
                <br />

                <Transition>
                    {move || {
                        if solved_challenges.get_untracked().contains(&cwa_popup.get().challenge.id) { 
                            solved.set(true);
                        } else {
                            solved.set(false);
                        }

                        if !solved.get_untracked() {
                            let challenge_id = cwa_popup.get_untracked().challenge.id;
                            let hints = hints.get_untracked().into_iter().filter(|h| h.challenge_id == challenge_id).collect::<Vec<HintWithoutHint>>();
                            let hints_used = hints_used.get_untracked();
                            let used_hint_ids = hints_used.into_iter().map(|h| h.hint_id).collect::<Vec<String>>();
                            view! {
                                <label class=r#"block mb-1 text-sm font-medium text-text"#>"Hints"</label>
                                <ForEnumerate
                                    each=move || hints.clone()
                                    key=|hint: &HintWithoutHint| hint.id.clone()
                                    children={move |index, hint| {
                                        let hint_id = hint.id.clone();
                                        if used_hint_ids.contains(&hint.id) {
                                            let bought_hint = hints_bought_signal.get_untracked().get(&hint_id).cloned();
                                            view! {
                                                <div class="flex gap-2">
                                                    <label>{format!("Hint {}", index.get() + 1)}</label>
                                                    <p>{bought_hint.unwrap_or_default()}</p>
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <div class="flex gap-2">
                                                    <label>{format!("Hint {}", index.get_untracked())}</label>
                                                    <button
                                                        class=r#"col-start-1 col-end-1 gap-2 items-center py-2 px-4 text-sm font-medium text-white 
                                                        rounded-lg transition focus:ring-2 focus:outline-none active:scale-95 
                                                        bg-yale-blue-600 hover:bg-yale-blue-700 focus:ring-yale-blue-400"#
                                                        on:click=move |_| {
                                                            let hint = hint.clone();
                                                            let challenge_id = cwa_popup.get_untracked().challenge.id;
                                                            spawn_local(async move {
                                                                if let Ok(hint) = get_hint(challenge_id, hint.id).await
                                                                {
                                                                    refresh.update(|n| *n += 1);
                                                                }
                                                            });
                                                        }
                                                    >
                                                        {format!("Get (-{}p)", hint.points_penalty)}
                                                    </button>
                                                </div>
                                            }.into_any()
                                        }
                                    }}
                                />
                            }.into_any()
                        } else {
                            "".into_any()
                        }
                    }}
                </Transition>

                <div class="flex gap-2 items-center">
                    <label
                        hidden=move || solved.get()
                        for="flag"
                    >
                        <b>"Flag: "</b>
                    </label>
                    <input
                        hidden=move || solved.get()
                        class=r#"m-1 bg-white rounded-sm"#
                        bind:value=flag_signal
                    />
                    <button
                        class=move || button_classes.get()
                        disabled=move || solved.get() || incorrect.get()
                        on:click=move |_| {
                            let start = start.clone();
                            let stop = stop.clone();
                            let refresh_user = refresh_user;
                            let flag = flag_signal.get();
                            let challenge = cwa_popup.get().challenge;
                            let challenge_vm_ids = challenge.clone().vm_ids.unwrap_or_default().split(",").map(|c| c.parse::<u32>().unwrap_or_default()).collect::<Vec<u32>>();
                            spawn_local(async move {
                                if let Ok(ApiResult { result, details }) = check_flag(flag, challenge.clone()).await
                                {
                                    if result == ResultStatus::Fail && details == "incorrect solution" {
                                        incorrect.set(true);
                                        stop();
                                        start(());
                                    } else if result == ResultStatus::Success {
                                        solved.set(true);
                                        for template_id in challenge_vm_ids {
                                            _ = destroy_vm(template_id).await;
                                        }
                                        refresh_user.update(|r| r.iteration += 1);
                                    }
                                }
                            });
                        }
                    >
                        {move || submit_btn_text.get()}
                    </button>
                </div>

                <For
                    each=move || cwa_popup.get().attachments.clone()
                    key=|a: &AttachmentWithoutBlob| a.id.clone()
                    let(a)
                >
                    <a
                        download
                        href=move || format!("/file/{}", a.id)
                        class=r#"text-blue-600 underline"#
                    >
                        {a.file_name}
                    </a>
                </For>

                <Transition>
                    {move || {
                        if solved_challenges.get().contains(&cwa_popup.get().challenge.id) { 
                            solved.set(true);
                        } else {
                            solved.set(false);
                        }

                        if !solved.get_untracked() {
                            let all_templates = all_templates.get();
                            let challenge = cwa_popup.get().challenge;
                            let challenge_vm_ids = challenge.clone().vm_ids;
                            let template_ids = match challenge_vm_ids.is_some() {
                                true => challenge_vm_ids.unwrap_or_default().split(",").map(|c| c.parse::<u32>().unwrap_or_default()).collect::<Vec<u32>>(),
                                false => Vec::<u32>::new()
                            };
                            let mut templates = all_templates.into_iter().filter(|t| template_ids.contains(&t.id)).collect::<Vec<ProxmoxVMTemplate>>();
                            view! {
                                <div class="grid auto-rows-auto gap-2 pt-2">
                                    // for each vm that the challenge has, show a button set
                                    <For
                                        each=move || templates.clone()
                                        key=|template: &ProxmoxVMTemplate| template.id
                                        let(template)
                                    >
                                        <div class="flex gap-2 items-center">
                                            <label>{template.name}</label>
                                            <Show when=move || {
                                                let user_vms = user_vms.get();
                                                let active_vm_origin_ids = user_vms.iter().map(|a| if a.running { a.origin_id } else { 0 }).collect::<Vec<u32>>();
                                                !active_vm_origin_ids.contains(&template.id)
                                            }>
                                                <button
                                                    class=r#"col-start-1 col-end-1 gap-2 items-center py-2 px-4 text-sm font-medium text-white 
                                                    rounded-lg transition focus:ring-2 focus:outline-none active:scale-95 
                                                    bg-yale-blue-600 hover:bg-yale-blue-700 focus:ring-yale-blue-400"#
                                                    on:click=move |_| {
                                                        let challenge = cwa_popup.get().challenge;
                                                        spawn_local(async move {
                                                            if let Ok(ApiResult { result, details }) = start_vm(template.id, challenge).await
                                                            {
                                                                if result == ResultStatus::Fail && details == "failed to start vm" {
                                                                    refresh.update(|n| *n += 1);
                                                                } else {
                                                                    refresh.update(|n| *n += 1);
                                                                }
                                                            }
                                                        });
                                                    }
                                                >
                                                    "Start VM"
                                                </button>
                                            </Show>

                                            <Show when=move || {
                                                let user_vms = user_vms.get();
                                                let active_vm_origin_ids = user_vms.iter().map(|a| if a.running { a.origin_id } else { 0 }).collect::<Vec<u32>>();
                                                active_vm_origin_ids.contains(&template.id)
                                            }>
                                                <button
                                                    class=r#"col-start-2 col-end-2 gap-2 items-center py-2 px-4 text-sm font-medium text-white 
                                                    rounded-lg transition focus:ring-2 focus:outline-none active:scale-95 
                                                    bg-yale-blue-600 hover:bg-yale-blue-700 focus:ring-yale-blue-400"#
                                                    on:click=move |_| {
                                                        spawn_local(async move {
                                                            _ = restart_vm(template.id).await;
                                                            refresh.update(|n| *n += 1);
                                                        });
                                                    }
                                                >
                                                    "Restart VM"
                                                </button>

                                                <button
                                                    class=r#"col-start-3 col-end-3 gap-2 items-center py-2 px-4 text-sm font-medium text-white 
                                                    rounded-lg transition focus:ring-2 focus:outline-none active:scale-95 
                                                    bg-yale-blue-600 hover:bg-yale-blue-700 focus:ring-yale-blue-400"#
                                                    on:click=move |_| {
                                                        spawn_local(async move {
                                                            _ = add_vm_time(template.id).await;
                                                            refresh.update(|n| *n += 1);
                                                        });
                                                    }
                                                >
                                                    "Add Time (+30 min)"
                                                </button>

                                                <button
                                                    class=r#"col-start-4 col-end-4 gap-2 items-center py-2 px-4 text-sm font-medium text-white 
                                                    rounded-lg transition focus:ring-2 focus:outline-none active:scale-95 
                                                    bg-yale-blue-600 hover:bg-yale-blue-700 focus:ring-yale-blue-400"#
                                                    on:click=move |_| {
                                                        spawn_local(async move {
                                                            _ = destroy_vm(template.id).await;
                                                            refresh.update(|n| *n += 1);
                                                        });
                                                    }
                                                >
                                                    "Destroy VM"
                                                </button>
                                            </Show>
                                        </div>
                                    </For>
                                </div>
                            }.into_any()
                        } else {
                            "".into_any()
                        }
                    }}
                </Transition>
            </div>
        </div>
    }
}

#[component]
pub fn Difficulty(difficulty: RwSignal<ChallengeWithAttachments>) -> impl IntoView {
    view! {
        <Transition fallback=move || {
            view! { <div>"..."</div> }
        }>
            {move || {
                view! {
                    <div
                        class=r#"difficulty"#
                        role="img"
                        aria-label=format!("Difficulty: {} of 5", difficulty.get().challenge.difficulty)
                    >
                        <span class=r#"label"#>
                            <b class=r#"text-lg/8"#>"Difficulty: "</b>
                            {"⭐".repeat(difficulty.get().challenge.difficulty as usize)}
                        </span>
                    </div>
                }
            }}
        </Transition>
    }
}
