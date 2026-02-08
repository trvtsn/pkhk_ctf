use icondata as i;
use leptos::{prelude::*};
use leptos_icons::Icon;

/// Default Home Page
#[component]
pub fn TruncatedDesc(description: RwSignal<Option<String>>) -> impl IntoView {
    let description = Memo::new(move |_| {
        description.get().unwrap_or_default()
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
        <Show when=move || desc_expanded.get() || !needs_truncate.get()>{description.get()}</Show>

        <Show when=move || {
            !desc_expanded.get() && needs_truncate.get()
        }>{truncated_desc.get()}</Show>

        <Show when=move || needs_truncate.get()>
            <button
                type="button"
                class=r#"ml-2 text-base text-blue-600 underline cursor-pointer"#
                on:click=move |_| {
                    desc_expanded.set(!desc_expanded.get());
                }
            >
                {show_more_less_text.get()}
            </button>
        </Show>
    }
}

#[component]
pub fn HidePasswordButton(hidden: RwSignal<bool>) -> impl IntoView {
    view! {
        <Show when=move || hidden.get()>
            <button
                type="button"
                class=r#"ml-2 text-base text-blue-600 underline cursor-pointer"#
                on:click=move |_| {
                    hidden.set(false);
                }
            >
                <Icon icon=i::LuEye />
            </button>
        </Show>

        <Show when=move || !hidden.get()>
            <button
                type="button"
                class=r#"ml-2 text-base text-blue-600 underline cursor-pointer"#
                on:click=move |_| {
                    hidden.set(true);
                }
            >
                <Icon icon=i::LuEyeOff />
            </button>
        </Show>
    }
}

#[component]
pub fn DimmingOverlay(overlay_triggered: RwSignal<bool>) -> impl IntoView {
    let classes = Memo::new(move |_| {
        let base = "absolute inset-0 bg-black/45 backdrop-blur-[1px] z-10";
        if overlay_triggered.get() {
            base.to_string()
        } else {
            format!("{base} hidden")
        }
    });

    view! {
        <div
            class=move || classes.get()
            aria-hidden=move || !overlay_triggered.get()
            on:click=move |_| overlay_triggered.set(false)
        ></div>
    }
}
