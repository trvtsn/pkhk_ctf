use crate::{
    components::{challenge::Challenge, navbar::NavBar},
    server::{db, get_all_challenges_with_attachments}
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
        <div class="container p-8 inline justify-center">
            <h1 class="text-4xl text-center">"Challenges"</h1>
            <div class="challenges grid-cols-4 p-4 m-4 flex">
                <Suspense fallback=move || view! { <div>"Loading..."</div> }>
                    {move || {
                        let challenges = cwa.get().map(move |result| match result {
                            Ok(challenges) => {
                                view! {
                                    <For
                                        each=move || challenges.clone()
                                        key=|challenge: &db::structs::ChallengeWithAttachments| challenge.challenge.id
                                        let(challenge)
                                    >
                                        <div class="challenge p-2">
                                            // <p>{foobar}</p>
                                            <Challenge
                                                title=challenge.challenge.name
                                                description=challenge.challenge.description
                                                difficulty=challenge.challenge.difficulty
                                                points=challenge.challenge.points
                                                attachments=challenge.attachments
                                            />
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
