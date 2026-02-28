use crate::{
    app::RefreshUser, components::{challenge::Challenge, challenge_popup::ChallengePopup, navbar::NavBar, utils::{ComponentSize, DimmingOverlay, Spinner}}, server::{db, enums::AdminEventPayloadKind, get_active_events, get_all_challenges_with_attachments, get_all_hints_without_hints, get_all_templates, get_hint, get_used_hints, get_user_solved_challenges, get_user_vms, proxmox::{ProxmoxVMInstance, ProxmoxVMTemplate}}
};
use leptos::prelude::*;
use leptos_use::{UseEventSourceOptions, UseEventSourceReturn, use_event_source_with_options};
use leptos::server::codee::string::FromToStringCodec;
use std::collections::HashMap;

/// Default Home Page
#[component]
pub fn Challenges() -> impl IntoView {
    let cwa_popup = RwSignal::new(None);
    let overlay_triggered = RwSignal::new(false);
    let refresh = RwSignal::new(0);
    let refresh_solved_challenges = RwSignal::new(0);
    let refresh_user = expect_context::<RwSignal<RefreshUser>>();
    let refresh_user_vms = RwSignal::new(0);

    let challenges_resource = Resource::new(move || refresh.get(), move |_| async move {
        get_all_challenges_with_attachments().await.unwrap_or_default()
    });

    let active_events_resource = Resource::new(move || refresh.get(), move |_| async move {
        get_active_events().await.unwrap_or_default()
    });

    let user_vms_signal = RwSignal::new(Vec::<ProxmoxVMInstance>::default());
    let user_vms_resource = Resource::new(move || refresh_user_vms.get(), move |_| async move { 
        get_user_vms().await.unwrap_or_default()
    });

    let solved_challenge_ids = RwSignal::new(Vec::<String>::default());
    let solved_challenges_resource = Resource::new(move || refresh_solved_challenges.get(), move |_| async move {
        get_user_solved_challenges().await.unwrap_or_default()
    });

    let all_templates_signal = RwSignal::<Vec<ProxmoxVMTemplate>>::new(vec![]);
    let all_templates_resource = Resource::new(move || refresh_user_vms.get(), move |_| async move {
        get_all_templates().await.unwrap_or_default()
    });

    let hints_signal = RwSignal::new(vec![]);
    let hints_resource = Resource::new(move || refresh.get(), move |_| async move {
        get_all_hints_without_hints().await.unwrap_or_default()
    });

    let hints_used_signal = RwSignal::new(HashMap::<String, String>::new());
    let hints_used_resource = Resource::new(move || (), move |_| async move { 
        get_used_hints().await.unwrap_or_default() 
    });
    
    let get_hint_action = Action::new(move |(challenge_id, hint_id): &(String, String)| {
        let challenge_id = challenge_id.clone();
        let hint_id = hint_id.clone();
        async move {
            if let Ok(hint) = get_hint(challenge_id, hint_id).await {
                hints_used_signal.update(|map| {map.insert(hint.id, hint.hint);});
            }
        }
    });

    let UseEventSourceReturn { message, .. } = 
        use_event_source_with_options::<String, FromToStringCodec>(
            "/events".to_string(), 
            UseEventSourceOptions::default().immediate(true)
        );

    Effect::new(move |_| {
        if let Some(msg) = message.get() {
            match serde_json::from_str::<AdminEventPayloadKind>(&msg.data) {
                Ok(AdminEventPayloadKind::NewChallengeCreated) | 
                Ok(AdminEventPayloadKind::ChallengeEdited) |
                Ok(AdminEventPayloadKind::ChallengeDeleted) |
                Ok(AdminEventPayloadKind::EventEdited) |
                Ok(AdminEventPayloadKind::EventDeleted) |
                Ok(AdminEventPayloadKind::NewEventCreated) => {
                    refresh.update(|n| *n += 1);
                    refresh_user.update(|r| r.iteration += 1);
                }
                Ok(_) => {},
                Err(_) => tracing::warn!("failed to parse AdminEventPayloadKind")
            }
        }
    });

    view! {
        <NavBar />
        <div 
            class=r#"p-4 bg-background text-text min-h-screen"#
        >
            <h1 class=r#"text-4xl text-center"#>"Challenges"</h1>
            <DimmingOverlay overlay_triggered />
            <div 
                class=r#"grid challenges justify-center mt-8"# 
            >
                <Transition fallback=move || {
                    view! { <Spinner component_size=ComponentSize::Big /> }
                }>
                    {move || {
                        if active_events_resource.get().unwrap_or_default().is_empty() {
                            view! { <p>"No events currently active"</p> }.into_any()
                        } else {
                            let all_templates = all_templates_resource.get().unwrap_or_default();
                            all_templates_signal.set(all_templates);

                            let hints = hints_resource.get().unwrap_or_default();
                            hints_signal.set(hints);

                            let hints_used = hints_used_resource.get().unwrap_or_default();
                            for hint_used in hints_used {
                                get_hint_action.dispatch((hint_used.challenge_id, hint_used.hint_id));
                            }

                            let user_vms = user_vms_resource.get().unwrap_or_default();
                            user_vms_signal.set(user_vms);

                            let mut map = HashMap::<
                                Option<String>,
                                Vec<db::structs::ChallengeWithAttachments>,
                            >::new();
                            for ch in challenges_resource.get().unwrap_or_default().into_iter() {
                                map.entry(ch.challenge.category.clone()).or_default().push(ch);
                            }
                            let mut groups = map
                                .into_iter()
                                .collect::<
                                    Vec<(Option<String>, Vec<db::structs::ChallengeWithAttachments>)>,
                                >();
                            // alphabetical sort, there's probably a better way to do this
                            groups
                                .sort_by(|(a, _), (b, _)| {
                                    a.as_deref().unwrap_or("").cmp(b.as_deref().unwrap_or(""))
                                });
                            solved_challenge_ids.set(solved_challenges_resource.get().unwrap_or_default());

                            view! {
                                <For
                                    each=move || groups.clone()
                                    key=|
                                        group: &(
                                            Option<String>,
                                            Vec<db::structs::ChallengeWithAttachments>,
                                        )|
                                    group.0.clone()
                                    let(group)
                                >
                                    <div class=r#"p-2 challenge-category"#>
                                        <h2 class=r#"text-2xl"#>
                                            {group.0.clone().unwrap_or("Uncategorized".to_string())}
                                        </h2>

                                        <div class=r#"grid grid-cols-4 m-4 content-stretch"#>
                                            <For
                                                each=move || group.1.clone()
                                                key=|cwa: &db::structs::ChallengeWithAttachments| {
                                                    cwa.challenge.id.clone()
                                                }
                                                children=move |cwa| {
                                                    let cwa = RwSignal::new(cwa);
                                                    view! {
                                                        <Show when=move || overlay_triggered.get() && 
                                                            cwa_popup.get().unwrap_or_default() == cwa.get()
                                                        >
                                                            <ChallengePopup 
                                                                cwa
                                                                cwa_popup
                                                                solved_challenges=solved_challenge_ids 
                                                                overlay_triggered 
                                                                all_templates=all_templates_signal
                                                                hints=hints_signal
                                                                user_vms=user_vms_signal
                                                                hints_used=hints_used_signal
                                                                refresh_solved_challenges
                                                                refresh_user_vms
                                                            />
                                                        </Show>

                                                        <div class=r#"p-2 challenge"#>
                                                            <Challenge
                                                                cwa
                                                                solved_challenges=solved_challenge_ids
                                                                overlay_triggered
                                                                cwa_popup
                                                                user_vms=user_vms_signal
                                                                refresh_solved_challenges
                                                            />
                                                        </div>
                                                    }
                                                }
                                            />
                                        </div>
                                    </div>
                                </For>
                            }.into_any()
                        }
                    }}
                </Transition>
            </div>
        </div>
    }
}
