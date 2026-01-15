use crate::{
    app::RefreshUser, components::{challenge::Challenge, navbar::NavBar}, server::{db::{self, structs::ChallengeWithAttachments}, enums::AdminEventPayloadKind, get_active_events, get_all_challenges_with_attachments, get_user_solved_challenges}
};
use leptos::prelude::*;
use leptos_use::{UseEventSourceOptions, UseEventSourceReturn, use_event_source_with_options};
use leptos::server::codee::string::FromToStringCodec;
use std::collections::HashMap;

/// Default Home Page
#[component]
pub fn Challenges() -> impl IntoView {
    let refresh = RwSignal::new(0);
    let refresh_user = expect_context::<RwSignal<RefreshUser>>();
    let challenges = RwSignal::<Vec<ChallengeWithAttachments>>::new(vec![]);
    let challenges_resource = Resource::new(move || refresh.get(), move |_| async move {
        get_all_challenges_with_attachments().await.unwrap_or_default()
    });

    let active_events = RwSignal::<Vec<db::structs::Event>>::new(vec![]);
    let active_events_resource = Resource::new(move || refresh.get(), move |_| async move {
        get_active_events().await.unwrap_or_default()
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

    let groups = Memo::new(move |_| {
        let mut map = HashMap::<Option<String>, Vec<db::structs::ChallengeWithAttachments>>::new();
        for ch in challenges.get().into_iter() {
            map.entry(ch.challenge.category.clone()).or_default().push(ch);
        }

        let mut groups = map.into_iter().collect::<Vec<(Option<String>, Vec<db::structs::ChallengeWithAttachments>)>>();

        // alphabetical sort, there's probably a better way to do this
        groups.sort_by(|(a, _), (b, _)| a.as_deref().unwrap_or("").cmp(b.as_deref().unwrap_or("")));
        
        groups
    });

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
                    let iteration = refresh_user.get().iteration + 1;
                    refresh_user.set(RefreshUser { iteration });
                }
                Ok(_) => {},
                Err(e) => tracing::warn!("failed to parse AdminEventPayloadKind: {}", e)
            }
        }

        let challenges_result = challenges_resource.get().unwrap_or_default();
        let events_result = active_events_resource.get().unwrap_or_default();
        let solved_challenges_result = solved_challenges_resource.get().unwrap_or_default();
        challenges.set(challenges_result);
        active_events.set(events_result);
        solved_challenge_ids.set(solved_challenges_result);
    });

    let challenges_view = move || { view! { 
        <div class="challenges">
            <Transition fallback=move || view! { <div>"Loading..."</div> }>
                <For
                    each=move || groups.get()
                    key=|group: &(Option<String>, Vec<db::structs::ChallengeWithAttachments>)| group.0.clone()
                    let(group)
                >
                    <div class="challenge-category p-2">
                        <h2 class="text-2xl">
                            { group.0.clone().unwrap_or_else(|| "Uncategorized".to_string()) }
                        </h2>

                        <div class="m-4 grid grid-cols-4 content-stretch">
                            <For
                                each=move || group.1.clone()
                                key=|challenge: &db::structs::ChallengeWithAttachments| challenge.challenge.id.clone()
                                let(challenge)
                            >
                                <div class="challenge p-2">
                                    <Challenge
                                        cwa=challenge
                                        solved_challenges=solved_challenge_ids
                                    />
                                </div>
                            </For>
                        </div>
                    </div>
                </For>
            </Transition>
        </div>
    }.into_any()};

    view! {
        <NavBar />
        <div class="grid justify-center p-4">
            <h1 class="text-4xl text-center">"Challenges"</h1>
            <Transition fallback=move || view! { <div>"Loading..."</div> }>
                {move || {
                    if !active_events.get().is_empty() {
                        (challenges_view)().into_any()
                    } else {
                        view! { <p>"No events currently active"</p> }.into_any()
                    }
                }}
            </Transition>
        </div>
    }
}
