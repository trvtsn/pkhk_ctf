use crate::app::RefreshUser;
use crate::components::utils::TruncatedDesc;
use crate::server::db::structs::{Challenge, ChallengeWithAttachments};
use crate::server::proxmox::ProxmoxVMInstance;
use crate::server::{check_flag, db::structs::AttachmentWithoutBlob, destroy_vm, enums::ResultStatus, structs::ApiResult};
use icondata as i;
use leptos::prelude::*;
use leptos_icons::Icon;
use leptos_use::{use_timeout_fn, UseTimeoutFnReturn};

#[component]
pub fn Challenge(
    cwa: ChallengeWithAttachments,
    solved_challenges: RwSignal<Vec<String>>,
    overlay_triggered: RwSignal<bool>,
    cwa_popup: RwSignal<ChallengeWithAttachments>,
    user_vms: RwSignal<Vec<ProxmoxVMInstance>>,
    refresh_solved_challenges: RwSignal<i32>
) -> impl IntoView {
    let ChallengeWithAttachments { challenge, attachments, illustration } = cwa.clone();
    let challenge_signal = RwSignal::new(challenge.clone());
    let description_signal = RwSignal::new(challenge.description.clone());
    let difficulty_signal = RwSignal::new(challenge.difficulty);
    let flag_signal = RwSignal::new("".to_string());
    let illustration_signal = RwSignal::new(illustration);

    let solved = RwSignal::new(false);
    let incorrect = RwSignal::new(false);

    let refresh_user = expect_context::<RwSignal<RefreshUser>>();

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
        if solved_challenges.get().contains(&challenge_signal.get().id) { 
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

    let check_flag_action = Action::new(move |(flag, challenge, template_id): &(String, Challenge, u32)| {
        let start = start.clone();
        let stop = stop.clone();
        let flag = flag.clone();
        let challenge = challenge.clone();
        let template_id = template_id.clone();
        async move {
            if let Ok(ApiResult { result, details }) = check_flag(flag, challenge).await {
                if result == ResultStatus::Fail && details == "incorrect solution" {
                    incorrect.set(true);
                    stop();
                    start(());
                } else if result == ResultStatus::Success {
                    _ = destroy_vm(template_id).await;
                    refresh_user.update(|r| r.iteration += 1);
                    refresh_solved_challenges.update(|r| *r += 1);
                }
            }
        }
    });

    view! {
        <div
            class=r#"content-center p-4 rounded-lg bg-card hover:bg-card-hover"#
        >
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

            <div class="flex items-center justify-between">
                <h3 class=r#"font-bold text-3xl/8"#>{move || challenge_signal.get().name.clone()}</h3>
                <button 
                    class="cursor-pointer"
                    on:click=move |_| {
                        overlay_triggered.set(true);
                        cwa_popup.set(cwa.clone());
                    }
                >
                    <Icon icon=i::LuMaximize2 />
                </button>
            </div>

            <p class=r#"text-lg/8"#>
                <TruncatedDesc description=description_signal />
            </p>

            <Difficulty difficulty_signal />

            <p class=r#"text-lg/8"#>
                <b>"Points: "</b>
                {move || challenge_signal.get().points}
            </p>
            <br />

            <div class="flex gap-2 items-center">
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
                        let challenge = challenge_signal.get();
                        let user_vms = user_vms.get();
                        let mut user_vm_id = 0_u32;
                        for user_vm in user_vms {
                            if user_vm.challenge_id == challenge.id {
                                user_vm_id = user_vm.id;
                            }
                        }
                        check_flag_action.dispatch((flag, challenge, user_vm_id));
                    }
                >
                    {move || submit_btn_text.get()}
                </button>
            </div>

            <div class="grid auto-rows-auto grid-cols-3 gap-2">
                <div class="col-start-1 col-end-1">
                    <For
                        each=move || attachments.clone()
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
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn Difficulty(difficulty_signal: RwSignal<i8>) -> impl IntoView {
    view! {
        <Transition fallback=move || {
            view! { <div>"..."</div> }
        }>
            {move || {
                view! {
                    <div
                        class=r#"difficulty"#
                        role="img"
                        aria-label=format!("Difficulty: {} of 5", difficulty_signal.get())
                    >
                        <span class=r#"label"#>
                            <b class=r#"text-lg/8"#>"Difficulty: "</b>
                            {"⭐".repeat(difficulty_signal.get() as usize)}
                        </span>
                    </div>
                }
            }}
        </Transition>
    }
}
