use base64::Engine;
use chrono::Local;
use leptos::{prelude::*, wasm_bindgen::JsCast};
use leptos::server::codee::string::FromToStringCodec;
use leptos_use::{use_event_source_with_options, UseEventSourceOptions, UseEventSourceReturn};

/// Default Home Page
#[component]
pub fn Log() -> impl IntoView {
    let logs = RwSignal::new(Vec::<String>::new());
    let UseEventSourceReturn { message, .. } = 
        use_event_source_with_options::<String, FromToStringCodec>(
            "/admin/logs".to_string(), 
            UseEventSourceOptions::default().immediate(true)
        );

    Effect::new(move |_| {
        if let Some(msg) = message.get() {
            let text = format!("[{}] {}\n", msg.event_type, msg.data);
            logs.update(|v| v.push(text));
        }
    });

    view! {
        <textarea class="server-logs w-full rounded-lg shadow-sm p-4" readonly rows="20">
        {move || {
            logs.get()
        }}
        </textarea>
        <div class="flex gap-3 mt-2">
            <button 
                class="bg-indigo-600 hover:bg-indigo-700 focus:ring-indigo-400 inline-flex items-center gap-2 rounded-lg text-white px-4 py-2 text-sm font-medium focus:outline-none focus:ring-2 active:scale-95 transition" 
                on:click=move |_| logs.set(Vec::new())
                >"Clear"</button>
            <button 
                class="bg-indigo-600 hover:bg-indigo-700 focus:ring-indigo-400 inline-flex items-center gap-2 rounded-lg text-white px-4 py-2 text-sm font-medium focus:outline-none focus:ring-2 active:scale-95 transition"
                on:click=move |_| {
                    let text = logs.get().join("");

                    let current_datetime = Local::now().format("%d_%m_%Y-%H_%M_%S");
                    let file_name = format!("pkhkctf-logs-{}.txt", current_datetime);

                    let b64 = base64::engine::general_purpose::STANDARD.encode(text.as_bytes());
                    let href = format!("data:text/plain;charset=utf-8;base64,{}", b64);

                    let window = leptos::web_sys::window().unwrap();
                    let document = window.document().unwrap();
                    let a = document
                        .create_element("a")
                        .unwrap()
                        .unchecked_into::<leptos::web_sys::HtmlAnchorElement>();

                    a.set_href(&href);
                    a.set_download(&file_name);
                    a.click();
                }
            >"Export"</button>
        </div>
    }
}
