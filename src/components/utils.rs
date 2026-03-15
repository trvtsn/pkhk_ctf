use crate::server::db::structs::DbUser;
use chrono::{DateTime, Local};
use icondata as i;
use leptos::prelude::*;
use leptos_icons::Icon;

#[component]
pub fn TruncatedDesc(
    #[prop(into)] 
    description: Signal<Option<String>>
) -> impl IntoView {
    const MAX_LEN: usize = 200;
    let expanded = RwSignal::new(false);

    let display_text = Memo::new(move |_| {
        let desc = description.get().unwrap_or_default();
        if !expanded.get() && desc.chars().count() > MAX_LEN {
            format!("{}...", desc.chars().take(MAX_LEN).collect::<String>())
        } else {
            desc
        }
    });

    let needs_truncate = Memo::new(move |_| {
        description.get().unwrap_or_default().chars().count() > MAX_LEN
    });

    view! {
        {move || display_text.get()}

        <Show when=move || needs_truncate.get()>
            <button
                type="button"
                class=r#"ml-2 text-base text-blue-600 underline cursor-pointer"#
                on:click=move |_| {
                    expanded.set(!expanded.get_untracked());
                }
            >
                {move || if expanded.get() { "Show Less" } else { "Show More" }}
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

#[component]
pub fn Difficulty(difficulty: i8) -> impl IntoView {
    view! {
        {move || {
            view! {
                <div
                    class=r#"difficulty"#
                    role="img"
                    aria-label=format!("Difficulty: {} of 5", difficulty)
                >
                    <span class=r#"label"#>
                        <b class=r#"text-lg/8"#>"Difficulty: "</b>
                        {"⭐".repeat(difficulty as usize)}
                    </span>
                </div>
            }
        }}
    }
}

#[component]
pub fn FileTooltip(
    file_name: String,
    id: String,
    #[prop(into)] on_download: String,
    on_remove: Callback<()>,
) -> impl IntoView {
    let show_tooltip = RwSignal::new(false);
    view! {
        <div class="flex gap-2 items-center">
            <span
                class="relative inline-block"
                on:mouseenter=move |_| show_tooltip.set(true)
                on:mouseleave=move |_| show_tooltip.set(false)
                on:focus=move |_| show_tooltip.set(true)
                on:blur=move |_| show_tooltip.set(false)
                tabindex="0"
            >
                {file_name}
                <Show when=move || show_tooltip.get()>
                    <div
                        role="tooltip"
                        class=r#"absolute left-1/2 bottom-full -translate-x-1/2 whitespace-nowrap
                            rounded p-1 text-xs bg-card-hover shadow-sm z-1"#
                    >
                        {format!("ID: {}", id)}
                    </div>
                </Show>
            </span>
            <a
                download
                href=on_download
            >
                <Icon icon=i::LuDownload />
            </a>
            <button
                class="cursor-pointer"
                on:click=move |_| on_remove.run(())
            >
                <Icon icon=i::LuX />
            </button>
        </div>
    }
}

#[component]
pub fn UserTooltip(db_user: DbUser) -> impl IntoView {
    let show_tooltip = RwSignal::new(false);
    view! {
        <div class="flex gap-2 items-center">
            <span
                class="relative inline-block"
                on:mouseenter=move |_| show_tooltip.set(true)
                on:mouseleave=move |_| show_tooltip.set(false)
                on:focus=move |_| show_tooltip.set(true)
                on:blur=move |_| show_tooltip.set(false)
                tabindex="0"
            >
                {db_user.username}
                <Show when=move || show_tooltip.get()>
                    <div
                        role="tooltip"
                        class=r#"absolute left-1/2 bottom-full -translate-x-1/2 whitespace-nowrap
                            rounded p-1 text-xs bg-card-hover shadow-sm z-1"#
                    >
                        <div>
                            <b>"ID: "</b>{format!("{}", db_user.id)}
                        </div>
                        <div>
                            <b>"E-mail: "</b>{format!("{}", db_user.email)}
                        </div>
                        <div>
                            <b>"Role: "</b>{format!("{}", db_user.role.to_string())}
                        </div>
                    </div>
                </Show>
            </span>
        </div>
    }
}

#[component]
pub fn VMTooltip(vm_id: u32, href: String, created_at: DateTime<Local>, end_at: DateTime<Local>) -> impl IntoView {
    let show_tooltip = RwSignal::new(false);
    view! {
        <div class="flex gap-2 items-center">
            <span
                class="relative inline-block"
                on:mouseenter=move |_| show_tooltip.set(true)
                on:mouseleave=move |_| show_tooltip.set(false)
                on:focus=move |_| show_tooltip.set(true)
                on:blur=move |_| show_tooltip.set(false)
                tabindex="0"
            >
                <a href=href target="_blank" class="flex gap-1 text-yale-blue-600 hover:text-yale-blue-300">
                    {vm_id}
                    <Icon icon=i::LuExternalLink width="0.8em" height="0.8em" />
                </a>
                <Show when=move || show_tooltip.get()>
                    <div
                        role="tooltip"
                        class=r#"absolute left-1/2 bottom-full -translate-x-1/2 whitespace-nowrap
                            rounded p-1 text-xs bg-card-hover shadow-sm z-1"#
                    >
                        <div>
                            <b>"Created At: "</b>{created_at.format("%Y-%m-%d %H:%M:%S").to_string()}
                        </div>
                        <div>
                            <b>"Expires At: "</b>{end_at.format("%Y-%m-%d %H:%M:%S").to_string()}
                        </div>
                    </div>
                </Show>
            </span>
        </div>
    }
}

#[component]
pub fn Gauge(percent: RwSignal<f32>) -> impl IntoView {
    view! {
        <div class="w-40 h-20 overflow-hidden">
            <div
                // for some reason we can't break this class into multiple lines
                // or else it will break and tailwind won't be able to generate
                // the required style
                class="w-40 h-40 rounded-full bg-[conic-gradient(from_270deg,var(--color-yale-blue-500)_0deg,var(--color-yale-blue-800)_calc(1.8deg*var(--percent)),var(--color-gauge-meter)_calc(1.8deg*var(--percent)),var(--color-gauge-meter)_180deg,transparent_180deg)]"
                style=move || format!("--percent: {:.2}", percent.get())
            >
                <div class="w-full h-full flex items-center justify-center">
                <div class="w-28 h-28 rounded-full bg-gauge-bg"></div>
                </div>
            </div>
        </div>
    }
}
