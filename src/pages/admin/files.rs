use crate::{components::{admin::file::File, utils::{ComponentSize, Spinner}}, server::{admin::{get_all_files, upload_files}, db}};
use leptos::{prelude::*, web_sys::{FormData, HtmlFormElement, HtmlInputElement, SubmitEvent}, wasm_bindgen::JsCast};

/// Default Home Page
#[component]
pub fn Files() -> impl IntoView {
    let refresh = RwSignal::new(0);
    let has_files_signal = RwSignal::new(false);
    let all_files = Resource::new(move || refresh.get(), move |_| async move {
        get_all_files().await.unwrap_or_default()
    });

    let upload_action = Action::new_local(move |data: &FormData| {
        let data = data.clone();
        async move {
            if let Ok(_) = upload_files(data.clone().into()).await {
                refresh.update(|n| *n += 1);
            }
        }
    });

    let upload_action_text = Memo::new(move |_| {
        if upload_action.pending().get() { "Uploading..." } else { "" }
    });

    view! {
        <form 
            class="flex flex-col gap-4 mb-4"
            on:submit=move |ev: SubmitEvent| {
                ev.prevent_default();
                if !has_files_signal.get() {
                    return;
                } else {
                    let target = ev.target().unwrap().unchecked_into::<HtmlFormElement>();
                    let fd = FormData::new_with_form(&target).unwrap();
                    upload_action.dispatch_local(fd);
                }
            }
        >
            <input 
                class=r#"p-3 bg-background rounded-lg shadow-sm"# 
                type="file" name="files" 
                multiple 
                on:change=move |ev| {
                    let input = ev.target().unwrap().unchecked_into::<HtmlInputElement>();
                    if input.files().is_some() {
                        has_files_signal.set(true);
                    }
                }
            />
            <input
                class=r#"inline-flex items-center py-2 px-4 ml-auto text-sm font-semibold text-white 
                rounded-md shadow-sm focus:ring-2 focus:outline-none bg-yale-blue-600 
                hover:bg-yale-blue-500 focus:ring-yale-blue-500"#
                disabled=move || !has_files_signal.get()
                type="submit"
                value="Upload"
            />
        </form>
        <p>{move || upload_action_text.get()}</p>
        <Transition fallback=move || {
            view! { <Spinner component_size=ComponentSize::Big /> }
        }>
            {move || {
                let all_files = all_files.get().unwrap_or_default();
                view! {
                    <div class=r#"grid grid-cols-4 m-2 files items-start gap-4"#>
                        <For
                            each=move || all_files.clone()
                            key=|file: &db::structs::AttachmentWithoutBlob| file.id.clone()
                            let(file)
                        >
                            <File file refresh />
                        </For>
                    </div>
                }
            }}
        </Transition>
    }
}
