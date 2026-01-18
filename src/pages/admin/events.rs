use crate::{components::admin::event::Event, pages::admin::Actions, server::{admin::get_all_events, db, enums::ResultStatus, structs::ApiResult}, utils::html_local_to_datetime};
use leptos::{prelude::*, task:: spawn_local};

/// Default Home Page
#[component]
pub fn Events() -> impl IntoView {
    let creating = RwSignal::new(false);
    let section = RwSignal::new(Actions::None);
    let refresh = RwSignal::new(0);

    let name_signal = RwSignal::new("".to_string());
    let description_signal = RwSignal::new("".to_string());
    let start_at_signal = RwSignal::new("".to_string());
    let end_at_signal = RwSignal::new("".to_string());
    
    let events_resource = Resource::new(move || refresh.get(), move |_| async move {
        get_all_events().await.unwrap_or_default()
    });

    view! {
        <div class=r#"flex gap-2 mb-4"#>
            <button
                class=r#"py-1 px-3 text-sm rounded-md border border-gray-300 hover:bg-gray-50"#
                on:click=move |_| {
                    if creating.get() {
                        creating.set(false);
                        section.set(Actions::None);
                    } else {
                        creating.set(true);
                        section.set(Actions::Create);
                    }
                }
            >
                "Create"
            </button>
        </div>

        <div class=r#"flex flex-col gap-4"#>
            <Show when=move || section.get() == Actions::Create>
                <label class=r#"block mb-1 text-sm font-medium text-gray-700"#>"Name"</label>
                <input
                    class=r#"py-2 px-3 w-full text-sm rounded-md border border-gray-300 
                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                    name="name"
                    bind:value=name_signal
                />

                <label class=r#"block mb-1 text-sm font-medium text-gray-700"#>"Description"</label>
                <input
                    class=r#"py-2 px-3 w-full text-sm rounded-md border border-gray-300 
                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                    name="description"
                    bind:value=description_signal
                />

                <label class=r#"block mb-1 text-sm font-medium text-gray-700"#>"Start Date"</label>
                <input
                    class=r#"py-2 px-3 w-full text-sm rounded-md border border-gray-300 
                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                    type="datetime-local"
                    name="start_at"
                    bind:value=start_at_signal
                />

                <label class=r#"block mb-1 text-sm font-medium text-gray-700"#>"End Date"</label>
                <input
                    class=r#"py-2 px-3 w-full text-sm rounded-md border border-gray-300 
                    focus:ring-2 focus:outline-none focus:ring-yale-blue-500"#
                    type="datetime-local"
                    name="end_at"
                    bind:value=end_at_signal
                />

                <div class=r#"flex gap-3 mt-2"#>
                    <button
                        type="button"
                        class=r#"py-2 px-4 text-sm rounded-md border border-gray-300 hover:bg-gray-50"#
                        on:click=move |_| { section.set(Actions::None) }
                    >
                        "Cancel"
                    </button>
                    <button
                        type="button"
                        class=r#"inline-flex items-center py-2 px-4 ml-auto text-sm font-semibold 
                        text-white rounded-md shadow-sm focus:ring-2 focus:outline-none 
                        bg-yale-blue-600 hover:bg-yale-blue-500 focus:ring-yale-blue-500"#
                        on:click=move |_| {
                            let name = name_signal.get();
                            let description = description_signal.get();
                            let start_at = html_local_to_datetime(start_at_signal.get());
                            let end_at = html_local_to_datetime(end_at_signal.get());
                            spawn_local(async move {
                                tracing::debug!("creating event...");
                                if let Ok(ApiResult { result, .. }) = crate::server::admin::event(crate::server::admin::EventAction::Create {
                                        name,
                                        description,
                                        start_at,
                                        end_at,
                                    })
                                    .await && result == ResultStatus::Success
                                {
                                    refresh.update(|n| *n += 1);
                                }
                            });
                        }
                    >
                        "Create"
                    </button>
                </div>
            </Show>
        </div>

        <div class=r#"events"#>
            <div class=r#"grid grid-cols-4 m-4 content-stretch"#>
                <Transition fallback=move || view! { <div>"Loading..."</div> }>
                    <For
                        each=move || events_resource.get().unwrap_or_default()
                        key=|event: &db::structs::Event| event.id.clone()
                        let(event)
                    >
                        <div class=r#"p-2 event"#>
                            <Event event refresh />
                        </div>
                    </For>
                </Transition>
            </div>
        </div>
    }
}
