use crate::{components::{leaderboard_chart::LeaderboardChart, navbar::NavBar}, server::{build_leaderboard_data, db::structs::{Submission, SubmissionWithData}, structs::{LeaderboardData, PivotRow}}};
use chrono::{DateTime, Utc};
use leptos::{logging::log, prelude::*};
use leptos_chartistry::*;
use std::collections::HashMap;

/// Default Home Page
#[component]
pub fn Leaderboard() -> impl IntoView {
    let data = Resource::new(move || (), move |_| async move {
        match build_leaderboard_data().await {
            Ok(Some(leaderboard_data)) => Ok(leaderboard_data),
            Ok(None) => {
                Ok(LeaderboardData {
                    event_name: "Bruh".to_string(),
                    x_min: DateTime::from_timestamp_nanos(1000),
                    x_max: DateTime::from_timestamp_nanos(1000),
                    y_max: 1000 as f64,
                    users: vec!["bruh_user".to_string()],
                    rows: vec![PivotRow::default()]
                })
            }
            Err(e) => Err(e)
        }
    });

    view! { 
        <NavBar />
        <div class="container p-8 inline justify-center">
            <h3 class="text-4xl text-center">"Leaderboard"</h3>
            //<div class="w-screen h-screen">
                <Suspense fallback=move || view! { <div>"Loading... bruh"</div> }>
                    {move || {
                        let result_view = data.get().map(|j| match j {
                            Ok(dataa) => { 
                                let usernames = dataa.users.clone();
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
                                log!("{:?}", dataa.rows);

                                view! {
                                    <LeaderboardChart
                                        series=RwSignal::new(series)
                                        data=RwSignal::new(dataa)
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
