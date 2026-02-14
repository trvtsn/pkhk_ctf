use crate::{components::{leaderboard_chart::LeaderboardChart, navbar::NavBar}, server::{build_leaderboard_data, enums::AdminEventPayloadKind, structs::PivotRow}};
use leptos::prelude::*;
use leptos_chartistry::*;
use leptos_use::{UseEventSourceOptions, UseEventSourceReturn, use_event_source_with_options};
use leptos::server::codee::string::FromToStringCodec;

/// Default Home Page
#[component]
pub fn Leaderboard() -> impl IntoView {
    let refresh = RwSignal::new(0);
    let leaderboard_data_resource = Resource::new(move || refresh.get(), move |_| async move {
        build_leaderboard_data().await.unwrap_or_default()
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
                Ok(AdminEventPayloadKind::ChallengeDeleted) |
                Ok(AdminEventPayloadKind::ChallengeEdited) |
                Ok(AdminEventPayloadKind::ChallengeSolved) => refresh.update(|n| *n += 1),
                Ok(_) => {},
                Err(e) => tracing::warn!("failed to parse AdminEventPayloadKind: {}", e)
            }
        }
    });

    view! {
        <NavBar />
        <div class=r#"grid justify-center p-4 bg-background text-text min-h-screen"#>
            <h3 class=r#"m-2 text-4xl text-center"#>"Leaderboard"</h3>
            <Transition fallback=move || {
                view! { <div>"Loading..."</div> }
            }>
                {move || {
                    let data = leaderboard_data_resource.get().unwrap_or_default();
                    let usernames = data.users.clone();
                    let base = Series::new(|r: &PivotRow| r.ts);
                    let series = usernames
                        .iter()
                        .cloned()
                        .fold(
                            base,
                            |s, name| {
                                let nm = name.clone();
                                s.line(
                                    Line::new(move |r: &PivotRow| {
                                            r.values.get(&nm).cloned().unwrap_or(f64::NAN)
                                        })
                                        .with_name(name),
                                )
                            },
                        );

                    view! {
                        <LeaderboardChart series=RwSignal::new(series) data=RwSignal::new(data) />
                    }
                }}
            </Transition>
        </div>
    }
}
