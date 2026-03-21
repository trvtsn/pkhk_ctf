use crate::{components::utils::Gauge, server::enums::ServerEventPayload};
use leptos::prelude::*;
use leptos::server::codee::string::FromToStringCodec;
use leptos_use::{use_event_source_with_options, UseEventSourceOptions, UseEventSourceReturn};

#[component]
pub fn Status() -> impl IntoView {
    let cpu_percent = RwSignal::new(0_f32);
    let ram_percent = RwSignal::new(0_f32);
    let ram_mb = RwSignal::new(0.0_f32);
    let uptime = RwSignal::new(String::from("0s"));
    let active_users = RwSignal::new(0_u32);
    let traffic = RwSignal::new(String::from("0 B"));

    let UseEventSourceReturn { message, .. } =
        use_event_source_with_options::<String, FromToStringCodec>(
            "/admin/status".to_string(),
            UseEventSourceOptions::default().immediate(true),
        );

    Effect::new(move |_| {
        if let Some(msg) = message.get() {
            match serde_json::from_str::<ServerEventPayload>(&msg.data) {
                Ok(ServerEventPayload::StatusData(data)) => {
                    uptime.set(data.uptime);
                    active_users.set(data.active_users);
                    cpu_percent.set(data.cpu_usage);
                    ram_percent.set(data.ram_usage);
                    ram_mb.set(data.ram_usage_mb);
                    traffic.set(data.traffic);
                },
                Ok(_) => {},
                Err(e) => tracing::warn!("failed to parse ServerEventPayload: {}", e)
            }
        }
    });

    view! {
        <div class="grid gap-8">
            <div class="grid grid-cols-2 gap-4">
                <div class="flex flex-col items-center gap-2">
                    <h3 class="text-lg font-semibold">"CPU Usage"</h3>
                    <Gauge percent=cpu_percent />
                    <p class="text-sm">{move || format!("{:.1}%", cpu_percent.get())}</p>
                </div>

                <div class="flex flex-col items-center gap-2">
                    <h3 class="text-lg font-semibold">"RAM Usage"</h3>
                    <Gauge percent=ram_percent />
                    <p class="text-sm">{move || format!("{:.1} MB", ram_mb.get())}</p>
                </div>
            </div>

            <div class="grid grid-cols-3 gap-4">
                <div class="flex flex-col items-center gap-2">
                    <h3 class="text-lg font-semibold">"Uptime"</h3>
                    <p class="text-2xl">{move || uptime.get()}</p>
                </div>

                <div class="flex flex-col items-center gap-2">
                    <h3 class="text-lg font-semibold">"Active Users"</h3>
                    <p class="text-2xl">{move || active_users.get()}</p>
                </div>

                <div class="flex flex-col items-center gap-2">
                    <h3 class="text-lg font-semibold">"Traffic"</h3>
                    <p class="text-2xl">{move || traffic.get()}</p>
                </div>
            </div>
        </div>
    }
}
