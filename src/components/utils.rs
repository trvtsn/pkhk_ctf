use leptos::{prelude::*};

/// Default Home Page
#[component]
pub fn TruncatedDesc(description_signal: RwSignal<Option<String>>) -> impl IntoView {
    let description = Memo::new(move |_| {
        description_signal.get().unwrap_or_default()
    });
    let desc_max_len = 200usize;
    let needs_truncate =  Memo::new(move |_| {
        description.get().clone().chars().count() > desc_max_len
    });
    let desc_expanded = RwSignal::new(false);

    let truncated_desc = Memo::new(move |_| {
        if needs_truncate.get() && !desc_expanded.get() {
            description.get().clone().chars().take(desc_max_len).collect::<String>()
        } else {
            description.get()
        }
    });

    view! {
        {move || {
            if desc_expanded.get() || !needs_truncate.get() {
                description.get().clone()
            } else {
                format!("{}...", truncated_desc.get())
            }
        }}

        {move || {
            if needs_truncate.get() {
                view! {
                    <button
                        class="ml-2 text-base underline text-blue-600"
                        on:click=move |_| {
                            desc_expanded.set(!desc_expanded.get());
                        }
                    >
                        { move || if desc_expanded.get() { "Show Less" } else { "Show More" } }
                    </button>
                }.into_any()
            } else {
                view! {}.into_any()
            }
        }}
    }
}
