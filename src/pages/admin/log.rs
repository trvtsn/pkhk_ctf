// use super::AdminNavBar;
// use crate::components::navbar::NavBar;
use leptos::prelude::*;
use leptos::server::codee::string::FromToStringCodec;
use leptos::web_sys::{EventSource, MessageEvent};
use leptos_use::{use_event_source_with_options, UseEventSourceMessage, UseEventSourceOptions, UseEventSourceReturn};

/// Default Home Page
#[component]
pub fn Log() -> impl IntoView {
    let logs = RwSignal::new(Vec::<String>::new());
    let UseEventSourceReturn { message, .. } = 
        use_event_source_with_options::<String, FromToStringCodec>(
            "/logs".to_string(), 
            UseEventSourceOptions::default().immediate(true)
        );

    Effect::new(move |_| {
        if let Some(msg) = message.get() {
            let text = format!("[{}] {}\n", msg.event_type, msg.data);
            logs.update(|v| v.push(text));
        }
    });

    view! {
        <textarea class="server-logs w-full border-1 border-black" readonly rows="20">
        {move || {
            logs.get()
        }}
        </textarea>
        <button on:click=move |_| logs.set(Vec::new())>"Clear"</button>
        <button>"Export"</button>
    }
}
