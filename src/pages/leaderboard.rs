use crate::{components::{leaderboard_chart::LeaderboardChart, navbar::NavBar, utils::{ComponentSize, Spinner}}, server::{api::build_leaderboard_data, enums::ServerEventPayload, structs::PivotRow}, utils::{use_server_events, OrToast}};
use leptos::prelude::*;
use leptos_chartistry::*;

/// Live leaderboard chart. Refreshes when challenges are created, edited, or solved.
#[component]
pub fn Leaderboard() -> impl IntoView {
    let refresh = RwSignal::new(0);
    let leaderboard_data_resource = Resource::new(move || refresh.get(), move |_| async move {
        build_leaderboard_data().await.or_toast_and_default("Failed to load leaderboard")
    });

    use_server_events("/events", move |payload| match payload {
        ServerEventPayload::NewChallengeCreated(_) |
        ServerEventPayload::ChallengeDeleted(_) |
        ServerEventPayload::ChallengeEdited(_) |
        ServerEventPayload::ChallengeSolved => refresh.update(|n| *n += 1),
        _ => {},
    });

    view! {
        <NavBar />
        <div class=r#"p-4 bg-background text-text min-h-screen"#>
            <h3 class=r#"m-2 text-4xl text-center"#>"Leaderboard"</h3>
            <Transition fallback=move || {
                view! { <Spinner component_size=ComponentSize::Small /> }
            }>
                {move || {
                    let data = leaderboard_data_resource.get().unwrap_or_default();
                    let base = Series::new(|r: &PivotRow| r.ts);
                    let series = data.users
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
                        <div class="grid justify-center mt-8">
                            <LeaderboardChart series=RwSignal::new(series) data=RwSignal::new(data) />
                        </div>
                    }
                }}
            </Transition>
        </div>
    }
}
