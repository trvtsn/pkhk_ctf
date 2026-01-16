use leptos::{prelude::*};

/// Default Home Page
#[component]
pub fn TruncatedDesc(description_signal: RwSignal<Option<String>>) -> impl IntoView {
    let description = Memo::new(move |_| {
        description_signal.get().unwrap_or_default()
    });

    let desc_max_len = 200usize;
    let desc_expanded = RwSignal::new(false);

    let needs_truncate =  Memo::new(move |_| {
        description.get().clone().chars().count() > desc_max_len
    });
    let truncated_desc = Memo::new(move |_| {
        if needs_truncate.get() && !desc_expanded.get() {
            format!("{}...", description.get().clone().chars().take(desc_max_len).collect::<String>())
        } else {
            description.get()
        }
    });
    let show_more_less_text = Memo::new(move |_| {
        if desc_expanded.get() { "Show Less" } else { "Show More" }
    });

    view! {
        <Show when=move || desc_expanded.get() || !needs_truncate.get()>
            {description.get()}
        </Show>

        <Show when=move || !desc_expanded.get() && needs_truncate.get()>
            {truncated_desc.get()}
        </Show>

        <Show when=move || needs_truncate.get()>
            <button
                type="button"
                class="ml-2 text-base underline text-blue-600 cursor-pointer"
                on:click=move |_| {
                    desc_expanded.set(!desc_expanded.get());
                }
            >
                {show_more_less_text.get()}
            </button>
        </Show>
    }
}
