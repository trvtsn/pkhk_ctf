use crate::{
    components::{challenge::Challenge, navbar::NavBar},
    server::{enums::AdminEventPayloadKind, db, get_active_events, get_all_challenges_with_attachments, get_user_solved_challenges}
};
use leptos::prelude::*;
use leptos_use::{UseEventSourceOptions, UseEventSourceReturn, use_event_source_with_options};
use leptos::server::codee::string::FromToStringCodec;
use std::collections::HashMap;

/// Default Home Page
#[component]
pub fn Challenges() -> impl IntoView {
    let refresh = RwSignal::new(0);
    let challenges = Resource::new(move || refresh.get(), move |_| async move {
        get_all_challenges_with_attachments().await.unwrap_or_default()
    });

    let active_events = Resource::new(move || refresh.get(), move |_| async move {
        get_active_events().await.unwrap_or_default()
    });

    let solved_challenge_ids = RwSignal::new(Vec::<String>::default());

    Resource::new(move || (), move |_| async move {
        match get_user_solved_challenges().await {
            Ok(solved) => {
                solved_challenge_ids.set(solved);
                Ok(())
            },
            Err(e) => Err(e)
        }
    });

    let UseEventSourceReturn { message, .. } = 
        use_event_source_with_options::<String, FromToStringCodec>(
            "/admin_sse".to_string(), 
            UseEventSourceOptions::default().immediate(true)
        );

    Effect::new(move |_| {
        if let Some(msg) = message.get() {
            // fallback for debugging for now
            refresh.update(|n| *n += 1);
            match serde_json::from_str::<AdminEventPayloadKind>(&msg.data) {
                Ok(AdminEventPayloadKind::NewChallengeCreated) | 
                Ok(AdminEventPayloadKind::ChallengeEdited) |
                Ok(AdminEventPayloadKind::ChallengeDeleted) |
                Ok(AdminEventPayloadKind::EventEdited) |
                Ok(AdminEventPayloadKind::EventDeleted) |
                Ok(AdminEventPayloadKind::NewEventCreated) => {
                    refresh.update(|n| *n += 1);
                }
                Ok(_) => {},
                Err(e) => tracing::warn!("failed to parse AdminEventPayloadKind: {}", e)
            }

            // if let Ok(kind) = msg.data.parse::<AdminEventPayloadKind>() {
            //     match kind {
            //         AdminEventPayloadKind::NewChallengeCreated
            //         | AdminEventPayloadKind::ChallengeEdited
            //         | AdminEventPayloadKind::ChallengeDeleted => {
            //             refresh.update(|n| *n += 1)
            //         }
            //         _ => {}
            //     }
            // }
        }
    });


    let challenges_view = move || { view! { 
        <div class="challenges">
            <Transition fallback=move || view! { <div>"Loading..."</div> }>
                {move || {
                    let challenges = match challenges.get() {
                        Some(challenges) => {
                            let mut map = HashMap::<Option<String>, Vec<db::structs::ChallengeWithAttachments>>::new();
                            for ch in challenges.into_iter() {
                                map.entry(ch.challenge.category.clone()).or_default().push(ch);
                            }

                            let mut groups = map.into_iter().collect::<Vec<(Option<String>, Vec<db::structs::ChallengeWithAttachments>)>>();

                            // alphabetical sort, there's probably a better way to do this
                            groups.sort_by(|(a, _), (b, _)| a.as_deref().unwrap_or("").cmp(b.as_deref().unwrap_or("")));

                            view! {
                                <For
                                    each=move || groups.clone()
                                    key=|group: &(Option<String>, Vec<db::structs::ChallengeWithAttachments>)| group.0.clone()
                                    let(group)
                                >
                                    <div class="challenge-category p-2">
                                        <h2 class="text-2xl">
                                            { move || group.0.clone().unwrap_or_else(|| "Uncategorized".to_string()) }
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
                            }.into_any()
                        }
                        None => {
                            view! {}.into_any()
                        }
                    };

                    view! {
                        {challenges}
                    }
                }}
            </Transition>
        </div>
    }.into_any()};

    view! {
        <NavBar />
        <div class="grid justify-center p-4">
            <h1 class="text-4xl text-center">"Challenges"</h1>
            <Transition fallback=move || view! { <div>"Loading..."</div> }>
                {move || {
                    match active_events.get() {
                        Some(events) => {
                            if !events.is_empty() {
                                (challenges_view)().into_any()
                            } else {
                                view! { <p>"No events currently active"</p> }.into_any()
                            }
                        }
                        None => view! { <p>"Loading..."</p> }.into_any()
                    }
                }}
            </Transition>
        </div>
    }
}
