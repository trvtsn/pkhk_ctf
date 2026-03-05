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
        description.get().chars().count() > desc_max_len
    });
    let truncated_desc = Memo::new(move |_| {
        if needs_truncate.get() && !desc_expanded.get() {
            format!("{}...", description.get().chars().take(desc_max_len).collect::<String>())
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
    let result_view = move || if !overlay_triggered.get() {
        "".into_any()
    } else {
        view! {
            <div
                class="fixed inset-0 bg-black/45 backdrop-blur-[1px] z-10"
                on:click=move |_| overlay_triggered.set(false)
            ></div>
        }.into_any()
    };

    view! {
        {result_view}
    }
}

pub enum ComponentSize {
    Small,
    Medium,
    Big,
}

/// Thanks to devAaus (https://github.com/devAaus)
/// https://uiverse.io/devAaus/funny-catfish-94
#[component]
pub fn Spinner(component_size: ComponentSize) -> impl IntoView {
    let blue_indicator_classes_base = "border-4 border-transparent text-blue-400 text-4xl animate-spin flex items-center justify-center border-t-blue-400 rounded-full".to_string();
    let blue_indicator_classes = match component_size {
        ComponentSize::Small => Memo::new(move |_| format!("w-5 h-5 {blue_indicator_classes_base}")),
        ComponentSize::Medium => Memo::new(move |_| format!("w-10 h-10 {blue_indicator_classes_base}")),
        ComponentSize::Big => Memo::new(move |_| format!("w-20 h-20 {blue_indicator_classes_base}")),
    };
    let red_indicator_classes_base = "border-4 border-transparent text-red-400 text-2xl animate-spin flex items-center justify-center border-t-red-400 rounded-full".to_string();
    let red_indicator_classes = match component_size {
        ComponentSize::Small => Memo::new(move |_| format!("w-4 h-4 {red_indicator_classes_base}")),
        ComponentSize::Medium => Memo::new(move |_| format!("w-8 h-8 {red_indicator_classes_base}")),
        ComponentSize::Big => Memo::new(move |_| format!("w-16 h-16 {red_indicator_classes_base}")),
    };
    view! {
        <div class="flex-col gap-4 w-full flex items-center justify-center">
            <div 
                class=move || blue_indicator_classes.get()
            >
                <div
                    class=move || red_indicator_classes.get()
                ></div>
            </div>
        </div>
    }
}