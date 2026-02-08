use crate::app::RefreshUser;
use crate::components::utils::TruncatedDesc;
use crate::server::db::enums::AttachmentIdentifier;
use crate::server::db::structs::{ChallengeWithAttachments, ProxmoxInstance};
use crate::server::{add_vm_time, destroy_vm, get_illustration_id, restart_vm, start_vm};
use crate::server::{check_flag, db::structs::AttachmentWithoutBlob, enums::ResultStatus, structs::ApiResult};
use leptos::{prelude::*, task::spawn_local};
use leptos_use::{use_timeout_fn, UseTimeoutFnReturn};
// use thaw::*;

#[component]
pub fn ChallengePopup(
    cwa_popup: RwSignal<ChallengeWithAttachments>,
    solved_challenges: RwSignal<Vec<String>>,
    overlay_triggered: RwSignal<bool>,
    active_vms: RwSignal<Vec<ProxmoxInstance>>,
    refresh: RwSignal<i32>
) -> impl IntoView {
    let description_signal = RwSignal::new(None);
    let flag_signal = RwSignal::new("".to_string());

    let solved = RwSignal::new(false);
    let incorrect = RwSignal::new(false);

    let refresh_user = expect_context::<RwSignal<RefreshUser>>();

    let illustration = Resource::new(move || (), move |_| {
        let challenge_id = cwa_popup.get().challenge.id;
        async move { get_illustration_id(AttachmentIdentifier::ChallengeId(challenge_id)).await.unwrap_or_default() }
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
                <button 
                    class="cursor-pointer"
                    on:click=move |_| overlay_triggered.set(false)
                >
                    "x"
                </button>

                <Transition fallback=move || {
                    view! { <div>"Loading..."</div> }
                }>
                    {move || {
                        if let Some(id) = illustration.get().unwrap_or_default() { 
                            view! {
                                <div class="h-48 w-48 flex justify-center m-auto">
                                    <img 
                                        src=move || format!("/image/{}", id) 
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

                <label for="flag">
                    <b>"Flag: "</b>
                </label>
                <input
                    hidden=move || solved.get()
                    class=r#"m-1 bg-white rounded-sm border-black border-1"#
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
                        spawn_local(async move {
                            if let Ok(ApiResult { result, details }) = check_flag(flag, challenge.clone()).await
                            {
                                if result == ResultStatus::Fail && details == "incorrect solution" {
                                    incorrect.set(true);
                                    stop();
                                    start(());
                                } else if result == ResultStatus::Success {
                                    solved.set(true);
                                    // let active_vm_id = active_vm_id.clone();
                                    // _ = destroy_vm(active_vm_id).await;
                                    let iteration = refresh_user.get().iteration + 1;
                                    refresh_user.set(RefreshUser { iteration });
                                }
                            }
                        });
                    }
                >
                    {move || submit_btn_text.get()}
                </button>

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
                        let active_vms = active_vms.get();
                        let challenge = cwa_popup.get().challenge;
                        let mut active_vm_id = 0_u32;
                        for active_vm in active_vms {
                            if active_vm.challenge_id == challenge.id {
                                active_vm_id = active_vm.vm_id;
                            }
                        }
                        view! {
                            <Show when=move || active_vm_id == 0 && cwa_popup.get().challenge.vm_id.is_some()>
                                <button
                                    class=r#"inline-flex gap-2 items-center py-2 px-4 text-sm font-medium text-white 
                                    rounded-lg transition focus:ring-2 focus:outline-none active:scale-95 
                                    bg-yale-blue-600 hover:bg-yale-blue-700 focus:ring-yale-blue-400"#
                                    on:click=move |_| {
                                        let challenge = cwa_popup.get().challenge;
                                        spawn_local(async move {
                                            if let Ok(ApiResult { result, details }) = start_vm(challenge).await
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
                            <Show when=move || active_vm_id != 0>
                                <button
                                    class=r#"inline-flex gap-2 items-center py-2 px-4 text-sm font-medium text-white 
                                    rounded-lg transition focus:ring-2 focus:outline-none active:scale-95 
                                    bg-yale-blue-600 hover:bg-yale-blue-700 focus:ring-yale-blue-400"#
                                    on:click=move |_| {
                                        let active_vm_id = active_vm_id.clone();
                                        spawn_local(async move {
                                            _ = restart_vm(active_vm_id).await;
                                            refresh.update(|n| *n += 1);
                                        });
                                    }
                                >
                                    "Restart VM"
                                </button>

                                <button
                                    class=r#"inline-flex gap-2 items-center py-2 px-4 text-sm font-medium text-white 
                                    rounded-lg transition focus:ring-2 focus:outline-none active:scale-95 
                                    bg-yale-blue-600 hover:bg-yale-blue-700 focus:ring-yale-blue-400"#
                                    on:click=move |_| {
                                        let active_vm_id = active_vm_id.clone();
                                        spawn_local(async move {
                                            _ = add_vm_time(active_vm_id).await;
                                            refresh.update(|n| *n += 1);
                                        });
                                    }
                                >
                                    "Add Time (+30 min)"
                                </button>

                                <button
                                    class=r#"inline-flex gap-2 items-center py-2 px-4 text-sm font-medium text-white 
                                    rounded-lg transition focus:ring-2 focus:outline-none active:scale-95 
                                    bg-yale-blue-600 hover:bg-yale-blue-700 focus:ring-yale-blue-400"#
                                    on:click=move |_| {
                                        let active_vm_id = active_vm_id.clone();
                                        spawn_local(async move {
                                            _ = destroy_vm(active_vm_id).await;
                                            refresh.update(|n| *n += 1);
                                        });
                                    }
                                >
                                    "Destroy VM"
                                </button>
                            </Show>
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
