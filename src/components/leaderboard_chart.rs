use crate::server::structs::{LeaderboardData, PivotRow};
use chrono::{DateTime, Local};
use leptos::prelude::*;
use leptos_chartistry::*;
// use thaw::*;

#[component]
pub fn LeaderboardChart(
    series: RwSignal<Series<PivotRow, DateTime<Local>, f64>>,
    data: RwSignal<LeaderboardData>,
) -> impl IntoView {
    let event_name = Memo::new(move |_| {
        data.get().event_name.clone()
    });

    let data = Memo::new(move |_| {
        data.get()
    });

    let data_rows = Memo::new(move |_| {
        data.get().rows
    });

    view! {
        <Transition fallback=move || view! { <p>"Loading..."</p> }>
            {move || {
                let event_name = event_name.get();
                let series = series.get().with_y_range(0.0, data.get().y_max);
                view! {
                    <Chart
                        aspect_ratio=AspectRatio::from_outer_ratio(1000.0, 500.0)
                        top=RotatedLabel::middle(event_name)
                        left=TickLabels::aligned_floats()
                        bottom=TickLabels::timestamps()
                        inner=[
                            AxisMarker::left_edge().into_inner(),
                            AxisMarker::bottom_edge().into_inner(),
                            XGridLine::default().into_inner(),
                            YGridLine::default().into_inner()
                        ]
                        tooltip=Tooltip::left_cursor()
                        series=series
                        data=data_rows
                    />
                }
            }}
        </Transition>
    }
}
