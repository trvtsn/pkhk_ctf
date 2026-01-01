use std::collections::HashMap;

use crate::{
    components::{challenge::Challenge, navbar::NavBar},
    server::{db, get_all_challenges_with_attachments, structs::ApiResult}
};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use server_fn::codec::JsonEncoding;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum AppError {
    ServerFnError(ServerFnErrorErr),
    DbError(String),
}

impl FromServerFnError for AppError {
    type Encoder = JsonEncoding;

    fn from_server_fn_error(value: ServerFnErrorErr) -> Self {
        AppError::ServerFnError(value)
    }
}

/// Default Home Page
#[component]
pub fn Challenges() -> impl IntoView {
    // load once on mount
    let cwa = Resource::new(move || (), move |_| async move {
        match get_all_challenges_with_attachments().await {
            Ok(cwa) => Ok(cwa),
            Err(e) => Err(e)
        }
    });

    // spawn_local({
    //     let challenges = challenges.clone();
    //     async move {
    //         if let Ok(all) = db::structs::Challenge::get_all_with_attachments().await {
    //             challenges.set(all);
    //         }
    //     }
    // });

    view! {
        <NavBar />
        // {move || if date >= events.starting_date && date <= events.end_date}
        <div class="container grid justify-center p-8">
            <h1 class="text-4xl text-center">"Challenges"</h1>
            <div class="challenges">
                <Suspense fallback=move || view! { <div>"Loading..."</div> }>
                    {move || {
                        let challenges = cwa.get().map(move |result| match result {
                            Ok(challenges) => {
                                let ApiResult { result, details } = challenges;
                                let challenges = details;
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
                                                    key=|challenge: &db::structs::ChallengeWithAttachments| challenge.challenge.id
                                                    let(challenge)
                                                >
                                                    <div class="challenge p-2">
                                                        <Challenge
                                                            id=challenge.challenge.id
                                                            name=challenge.challenge.name.clone()
                                                            description=challenge.challenge.description.clone()
                                                            event_id=challenge.challenge.event_id
                                                            category=challenge.challenge.category.clone()
                                                            difficulty=challenge.challenge.difficulty
                                                            points=challenge.challenge.points
                                                            attachments=challenge.attachments.clone()
                                                        />
                                                    </div>
                                                </For>
                                            </div>
                                        </div>
                                    </For>
                                }.into_any()
                            }
                            Err(e) => {
                                view! {
                                    <div class="challenge p-2">
                                        <p>"Bruh" {e.to_string()}</p>
                                    </div>
                                }.into_any()
                            }
                        })
                        .collect_view()
                        .into_any();

                        view! {
                            {challenges}
                        }
                    }}
                </Suspense>
            </div>
        </div>
    }
}
