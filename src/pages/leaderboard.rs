use crate::{components::{leaderboard_chart::LeaderboardChart, navbar::NavBar}, server::{build_leaderboard_data, db::structs::{Submission, SubmissionWithData}, structs::{ApiResult, LeaderboardData, PivotRow}}};
use chrono::{DateTime, Utc};
use leptos::{logging::log, prelude::*};
use leptos_chartistry::*;
use std::collections::HashMap;

/// Default Home Page
#[component]
pub fn Leaderboard() -> impl IntoView {
    let leaderboard_data = Resource::new(move || (), move |_| async move {
        match build_leaderboard_data().await {
            Ok(leaderboard_data) => Ok(leaderboard_data),
            Err(e) => Err(e)
        }
    });

    view! { 
        <NavBar />
        <div class="p-4 justify-center grid">
            <h3 class="text-4xl text-center m-2">"Leaderboard"</h3>
            //<div class="w-screen h-screen">
                <Suspense fallback=move || view! { <div>"Loading..."</div> }>
                    {move || {
                        let result_view = leaderboard_data.get().map(|ld| match ld {
                            Ok(data) => { 
                                let usernames = data.users.clone();
                                log!("let usernames");

                                let base = Series::new(|r: &PivotRow| r.ts);
                                log!("let base");

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
                            Err(e) => view! { {e.to_string()} }.into_any()
                        });

                        view! {
                            { result_view }
                        }
                    }}
                </Suspense>
            //</div>
        </div>
    }
}
