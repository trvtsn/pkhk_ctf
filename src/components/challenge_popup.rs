use std::collections::HashMap;

use crate::app::RefreshUser;
use crate::components::utils::{Spinner, TruncatedDesc, ComponentSize};
use crate::server::db::structs::{Challenge, ChallengeWithAttachments, DbHintWithoutHint, HintWithoutHint};
use crate::server::proxmox::{ProxmoxVMTemplate};
use crate::server::{StartVM, add_vm_time, destroy_vm, get_hint, get_used_hints, get_user_vms, restart_vm, start_vm};
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
    all_templates: RwSignal<Vec<ProxmoxVMTemplate>>,
    hints: RwSignal<Vec<DbHintWithoutHint>>
) -> impl IntoView {
    let description_signal = RwSignal::new(None);
    let flag_signal = RwSignal::new("".to_string());
    let hints_bought_signal = RwSignal::new(HashMap::<String, String>::new());
    let refresh = RwSignal::new(0);

    let hints_used_resource = Resource::new(move || hints_bought_signal.get(), move |_| async move { 
        get_used_hints().await.unwrap_or_default() 
    });
    let user_vms_resource = Resource::new(move || refresh.get(), move |_| async move { 
        get_user_vms().await.unwrap_or_default()
    });

    let incorrect = RwSignal::new(false);
    let refresh_user = expect_context::<RwSignal<RefreshUser>>();
    
    let solved = Memo::new(move |_| {
        if solved_challenges.get().contains(&cwa_popup.get().challenge.id) { true } else { false }
    });

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

    let get_hint_action = Action::new(move |(challenge_id, hint_id): &(String, String)| {
        let challenge_id = challenge_id.clone();
        let hint_id = hint_id.clone();
        async move {
            get_hint(challenge_id, hint_id).await
        }
    });
    let start_vm_action = Action::new(move |(template_id, challenge): &(u32, Challenge)| {
        let template_id = template_id.clone();
        let challenge = challenge.clone();
        async move {
            start_vm(template_id, challenge).await
        }
    });
    let restart_vm_action = Action::new(move |template_id: &u32| {
        let template_id = template_id.clone();
        async move {
            restart_vm(template_id).await
        }
    });
    let add_vm_time_action = Action::new(move |template_id: &u32| {
        let template_id = template_id.clone();
        async move {
            add_vm_time(template_id).await
        }
    });
    let destroy_vm_action = Action::new(move |template_id: &u32| {
        let template_id = template_id.clone();
        async move {
            destroy_vm(template_id).await
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

    let UseTimeoutFnReturn { start, stop, .. } =
        use_timeout_fn(move |_: ()| {
            // runs after the delay on the client
            incorrect.set(false);
        }, 2000.0);

    Effect::new(move |_| {
        if let Some(Ok(ApiResult { .. })) = start_vm_action.value().get() {
            refresh.update(|n| *n += 1);
        }
        if let Some(Ok(ApiResult { .. })) = restart_vm_action.value().get() {
            refresh.update(|n| *n += 1);
        }
        if let Some(Ok(ApiResult { .. })) = add_vm_time_action.value().get() {
            refresh.update(|n| *n += 1);
        }
        if let Some(Ok(ApiResult { .. })) = destroy_vm_action.value().get() {
            refresh.update(|n| *n += 1);
        }
        if let Some(Ok(hint)) = get_hint_action.value().get() {
            hints_bought_signal.update(|map| {map.insert(hint.id, hint.hint);});
            refresh_user.update(|r| r.iteration += 1);
        }
    });

    let result_view = move || if !overlay_triggered.get() {
        "".into_any()
    } else {
        let start = start.clone();
        let stop = stop.clone();
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
                            let challenge_id = cwa_popup.get().challenge.id;
                            let hints = hints.get().into_iter().filter(|h| h.challenge_id == challenge_id).collect::<Vec<HintWithoutHint>>();
                            if !solved.get() && !hints.is_empty() {
                                let hints_used = hints_used_resource.get().unwrap_or_default();
                                let used_hint_ids = hints_used.into_iter().map(|h| h.hint_id).collect::<Vec<String>>();
                                view! {
                                    <label class=r#"block mb-1 text-sm font-medium text-text"#>"Hints"</label>
                                    <div class="grid gap-2">
                                        <ForEnumerate
                                            each=move || hints.clone()
                                            key=|hint: &HintWithoutHint| hint.id.clone()
                                            children={move |index, hint| {
                                                let hint_id = hint.id.clone();
                                                if used_hint_ids.contains(&hint.id) {
                                                    let challenge_id = challenge_id.clone();
                                                    get_hint_action.dispatch((challenge_id, hint.id));

                                                    view! {
                                                        <div class="flex gap-2 items-center">
                                                            <label>{format!("Hint {}", index.get() + 1)}</label>
                                                            <p>{move || hints_bought_signal.get().get(&hint_id).cloned().unwrap_or_default()}</p>
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
                                                                    let challenge_id = cwa_popup.get_untracked().challenge.id;
                                                                    get_hint_action.dispatch((challenge_id, hint.id));
                                                                }
                                                            >
                                                                {move || {
                                                                    if get_hint_action.pending().get() {
                                                                        view! { <Spinner component_size=ComponentSize::Small /> }.into_any()
                                                                    } else {
                                                                        view! { { format!("Get (-{}p)", hint.points_penalty) } }.into_any()
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
                            if !solved.get() {
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
                                                    let user_vms = user_vms_resource.get().unwrap_or_default();
                                                    let active_vm_origin_ids = user_vms.iter().map(|a| if a.running { a.origin_id } else { 0 }).collect::<Vec<u32>>();
                                                    !active_vm_origin_ids.contains(&template.id)
                                                }>
                                                    <button
                                                        class=r#"col-start-1 col-end-1 gap-2 items-center py-2 px-4 text-sm font-medium text-white 
                                                        rounded-lg transition focus:ring-2 focus:outline-none active:scale-95 
                                                        bg-yale-blue-600 hover:bg-yale-blue-700 focus:ring-yale-blue-400"#
                                                        on:click=move |_| {
                                                            let challenge = cwa_popup.get().challenge;
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

                                                <Show when=move || {
                                                    let user_vms = user_vms_resource.get().unwrap_or_default();
                                                    let active_vm_origin_ids = user_vms.iter().map(|a| if a.running { a.origin_id } else { 0 }).collect::<Vec<u32>>();
                                                    active_vm_origin_ids.contains(&template.id)
                                                }>
                                                    <button
                                                        class=r#"col-start-2 col-end-2 gap-2 items-center py-2 px-4 text-sm font-medium text-white 
                                                        rounded-lg transition focus:ring-2 focus:outline-none active:scale-95 
                                                        bg-yale-blue-600 hover:bg-yale-blue-700 focus:ring-yale-blue-400"#
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
        }.into_any()
    };

    view! {
        {result_view}
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
