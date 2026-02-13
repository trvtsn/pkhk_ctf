use crate::{
    app::RefreshUser, components::{challenge::Challenge, challenge_popup::ChallengePopup, navbar::NavBar, utils::DimmingOverlay}, server::{db::{self, structs::ChallengeWithAttachments}, enums::AdminEventPayloadKind, get_active_events, get_all_challenges_with_attachments, get_user_vms, get_user_solved_challenges, proxmox::ProxmoxVMInstance}
};
use leptos::prelude::*;
use leptos_use::{UseEventSourceOptions, UseEventSourceReturn, use_event_source_with_options};
use leptos::server::codee::string::FromToStringCodec;
use std::collections::HashMap;

/// Default Home Page
#[component]
pub fn Challenges() -> impl IntoView {
    let cwa_popup = RwSignal::new(ChallengeWithAttachments::default());
    let overlay_triggered = RwSignal::new(false);
    let refresh = RwSignal::new(0);
    let refresh_user = expect_context::<RwSignal<RefreshUser>>();
    let challenges_resource = Resource::new(move || refresh.get(), move |_| async move {
        get_all_challenges_with_attachments().await.unwrap_or_default()
    });

    let active_events_resource = Resource::new(move || refresh.get(), move |_| async move {
        get_active_events().await.unwrap_or_default()
    });

    let user_vms_signal = RwSignal::new(Vec::<ProxmoxVMInstance>::default());
    let user_vms_resource = Resource::new(move || refresh.get(), move |_| {
        async move { get_user_vms().await.unwrap_or_default() }
    });

    let solved_challenge_ids = RwSignal::new(Vec::<String>::default());
    let solved_challenges_resource = Resource::new(move || (), move |_| async move {
        get_user_solved_challenges().await.unwrap_or_default()
    });

    let UseEventSourceReturn { message, .. } = 
        use_event_source_with_options::<String, FromToStringCodec>(
            "/admin_sse".to_string(), 
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

    let challenges_view = move || { view! {
        <DimmingOverlay overlay_triggered />
        <div 
            class=r#"challenges"# 
        >
            <Transition fallback=move || {
                view! { <div>"Loading..."</div> }
            }>
                {move || {
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
                        <ChallengePopup 
                            cwa_popup=cwa_popup 
                            solved_challenges=solved_challenge_ids 
                            overlay_triggered 
                            user_vms=user_vms_signal 
                            refresh 
                        />
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
                                        key=|challenge: &db::structs::ChallengeWithAttachments| {
                                            challenge.challenge.id.clone()
                                        }
                                        let(challenge)
                                    >
                                        <div class=r#"p-2 challenge"#>
                                            <Challenge
                                                cwa=challenge
                                                solved_challenges=solved_challenge_ids
                                                overlay_triggered
                                                cwa_popup=cwa_popup
                                                user_vms=user_vms_signal 
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
    }.into_any()};

    view! {
        <NavBar />
        <div 
            class=r#"grid justify-center p-4 bg-background text-text min-h-screen"#
        >
            <h1 class=r#"text-4xl text-center"#>"Challenges"</h1>
            <Transition fallback=move || {
                view! { <div>"Loading..."</div> }
            }>
                {move || {
                    if !active_events_resource.get().unwrap_or_default().is_empty() {
                        view! { {challenges_view} }.into_any()
                    } else {
                        view! { <p>"No events currently active"</p> }.into_any()
                    }
                }}
            </Transition>
        </div>
    }
}
