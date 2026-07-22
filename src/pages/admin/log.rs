use crate::components::toast::{ToastMessageType, push_new_toast};
use crate::utils::use_sse;
use base64::Engine;
use chrono::Local;
use leptos::{prelude::*, wasm_bindgen::JsCast};

/// Live server log viewer, streamed via SSE.
#[component]
pub fn Log() -> impl IntoView {
    let logs = RwSignal::new(Vec::<String>::new());
    use_sse("/admin/logs", move |event_type, data| {
        logs.update(|v| v.push(format!("[{event_type}] {data}\n")));
    });

    view! {
        <textarea class=r#"p-4 w-full rounded-lg shadow-sm server-logs bg-background"# readonly rows="20">
            {move || logs.get()}
        </textarea>
        <div class=r#"flex gap-3 mt-2"#>
            <button
                class=r#"inline-flex gap-2 items-center py-2 px-4 text-sm font-medium text-white 
                rounded-lg transition focus:ring-2 focus:outline-none active:scale-95 
                bg-yale-blue-600 hover:bg-yale-blue-700 focus:ring-yale-blue-400"#
                on:click=move |_| logs.set(Vec::new())
            >
                "Clear"
            </button>
            <button
                class=r#"inline-flex gap-2 items-center py-2 px-4 text-sm font-medium text-white 
                rounded-lg transition focus:ring-2 focus:outline-none active:scale-95 
                bg-yale-blue-600 hover:bg-yale-blue-700 focus:ring-yale-blue-400"#
                on:click=move |_| {
                    let text = logs.get_untracked().join("");
                    let current_datetime = Local::now().format("%d_%m_%Y-%H_%M_%S");
                    let file_name = format!("pkhkctf-logs-{}.txt", current_datetime);
                    let b64 = base64::engine::general_purpose::STANDARD.encode(text.as_bytes());
                    let href = format!("data:text/plain;charset=utf-8;base64,{}", b64);
                    let document = match leptos::web_sys::window().and_then(|window| window.document()) {
                        Some(document) => document,
                        None => { push_new_toast(ToastMessageType::ErrorOccurred); return }
                    };
                    let a = match document.create_element("a") {
                        Ok(el) => el.unchecked_into::<leptos::web_sys::HtmlAnchorElement>(),
                        Err(_) => { push_new_toast(ToastMessageType::ErrorOccurred); return }
                    };
                    a.set_href(&href);
                    a.set_download(&file_name);
                    a.click();
                }
            >
                "Export"
            </button>
        </div>
    }
}
