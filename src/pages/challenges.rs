use crate::{
    app::RefreshUser, components::{challenge::Challenge, navbar::NavBar}, server::{db, enums::AdminEventPayloadKind, get_active_events, get_all_challenges_with_attachments, get_user_solved_challenges}
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
    let challenges_resource = Resource::new(move || refresh.get(), move |_| async move {
        get_all_challenges_with_attachments().await.unwrap_or_default()
    });

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
                    let iteration = refresh_user.get_untracked().iteration + 1;
                    refresh_user.set(RefreshUser { iteration });
                }
                Ok(_) => {},
                Err(_) => tracing::warn!("failed to parse AdminEventPayloadKind")
            }
        }
    });

    let challenges_view = move || { view! {
        <div class=r#"challenges"#>
            <Transition fallback=move || {
                view! { <div>"Loading..."</div> }
            }>
                {move || {
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
                    groups
                        .sort_by(|(a, _), (b, _)| {
                            a.as_deref().unwrap_or("").cmp(b.as_deref().unwrap_or(""))
                        });
                    solved_challenge_ids.set(solved_challenges_resource.get().unwrap_or_default());

                    // alphabetical sort, there's probably a better way to do this

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
                                    {group.0.clone().unwrap_or_else(|| "Uncategorized".to_string())}
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
        <div class=r#"grid justify-center p-4 bg-background text-text h-full"#>
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
