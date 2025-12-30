use crate::server::structs::{LeaderboardData, PivotRow};
use chrono::{DateTime, Utc};
use leptos::prelude::*;
use leptos_chartistry::*;
// use thaw::*;

#[component]
pub fn LeaderboardChart(
    series: RwSignal<Series<PivotRow, DateTime<Utc>, f64>>,
    data: RwSignal<LeaderboardData>,
) -> impl IntoView {
    view! {
        <Chart
            aspect_ratio=AspectRatio::from_outer_ratio(900.0, 400.0)
            top=RotatedLabel::middle(data.get().event_name.clone())
            left=TickLabels::aligned_floats()
            bottom=TickLabels::timestamps()
            inner=[
                AxisMarker::left_edge().into_inner(),
                AxisMarker::bottom_edge().into_inner(),
                XGridLine::default().into_inner(),
                YGridLine::default().into_inner()
            ]
            tooltip=Tooltip::left_cursor()
            series=series.get().with_y_range(0.0, data.get().y_max)
            data=data.get().rows
        />
    }
}
