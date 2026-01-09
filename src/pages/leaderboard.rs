use crate::{components::{leaderboard_chart::LeaderboardChart, navbar::NavBar}, server::{build_leaderboard_data, structs::PivotRow}};
use leptos::{logging::log, prelude::*};
use leptos_chartistry::*;

/// Default Home Page
#[component]
pub fn Leaderboard() -> impl IntoView {
    let leaderboard_data = Resource::new(move || (), move |_| async move {
        build_leaderboard_data().await.unwrap_or_default()
    });

    view! { 
        <NavBar />
        <div class="p-4 justify-center grid">
            <h3 class="text-4xl text-center m-2">"Leaderboard"</h3>
            <Suspense fallback=move || view! { <div>"Loading..."</div> }>
                {move || {
                    let result_view = match leaderboard_data.get() {
                        Some(data) => { 
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
                            }.into_any()
                        }
                        None => view! { "No events currently active" }.into_any()
                    };

                    view! {
                        { result_view }
                    }
                }}
            </Suspense>
        </div>
    }
}
