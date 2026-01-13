use crate::{components::{leaderboard_chart::LeaderboardChart, navbar::NavBar}, server::{build_leaderboard_data, enums::AdminEventPayloadKind, structs::{LeaderboardData, PivotRow}}};
use leptos::prelude::*;
use leptos_chartistry::*;
use leptos_use::{UseEventSourceOptions, UseEventSourceReturn, use_event_source_with_options};
use leptos::server::codee::string::FromToStringCodec;

/// Default Home Page
#[component]
pub fn Leaderboard() -> impl IntoView {
    let refresh = RwSignal::new(0);
    let leaderboard_data = RwSignal::new(LeaderboardData::default());
    let leaderboard_data_resource = Resource::new(move || refresh.get(), move |_| async move {
        build_leaderboard_data().await.unwrap_or_default()
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
                Ok(AdminEventPayloadKind::ChallengeDeleted) |
                Ok(AdminEventPayloadKind::ChallengeEdited) |
                Ok(AdminEventPayloadKind::ChallengeSolved) => refresh.update(|n| *n += 1),
                Ok(_) => {},
                Err(e) => tracing::warn!("failed to parse AdminEventPayloadKind: {}", e)
            }
        }

        let leaderboard_data_result = leaderboard_data_resource.get().unwrap_or_default();
        leaderboard_data.set(leaderboard_data_result);
    });

    view! { 
        <NavBar />
        <div class="p-4 justify-center grid">
            <h3 class="text-4xl text-center m-2">"Leaderboard"</h3>
            <Transition fallback=move || view! { <div>"Loading..."</div> }>
                {move || {
                    let data = leaderboard_data.get();
                    let usernames = data.users.clone();
                    let base = Series::new(|r: &PivotRow| r.ts);
                    let series = usernames.iter().cloned().fold(base, |s, name| {
                        let nm = name.clone();
                        s.line(
                            Line::new(move |r: &PivotRow| {
                                r.values.get(&nm).cloned().unwrap_or(f64::NAN)
                            })
                            .with_name(name),
                        )
                    });

                    view! {
                        <LeaderboardChart
                            series=RwSignal::new(series)
                            data=RwSignal::new(data)
                        />
                    }
                }}
            </Transition>
        </div>
    }
}
