use crate::{components::{admin::file::File, toast::{ToastMessageType, push_new_toast}, utils::{ComponentSize, Spinner}}, server::{admin::{api::{get_all_files, upload_files}}, db}};
use leptos::{prelude::*, task::spawn_local, wasm_bindgen::JsCast, web_sys::{FormData, HtmlFormElement, HtmlInputElement, SubmitEvent}};

/// Admin file browser.
/// Upload and manage standalone attachments.
#[component]
pub fn Files() -> impl IntoView {
    let refresh = RwSignal::new(0);
    let has_files_signal = RwSignal::new(false);
    let all_files = Resource::new(move || refresh.get(), move |_| async move {
        get_all_files().await.unwrap_or_default()
    });

    view! {
        <form 
            class="flex flex-col gap-4 mb-4"
            on:submit=move |ev: SubmitEvent| {
                ev.prevent_default();
                if !has_files_signal.get() {
                    return;
                } else {
                    let form = match ev.target() {
                        Some(target) => target.unchecked_into::<HtmlFormElement>(),
                        None => { push_new_toast(ToastMessageType::ErrorOccurred); return }
                    };
                    let fd = match FormData::new_with_form(&form) {
                        Ok(fd) => fd,
                        Err(_) => { push_new_toast(ToastMessageType::ErrorOccurred); return }
                    };
                    spawn_local(async move {
                        if upload_files(fd.into()).await.is_ok() {
                            refresh.update(|n| *n += 1);
                        }
                    });
                }
            }
        >
            <input 
                class=r#"p-3 bg-background rounded-lg shadow-sm"# 
                type="file" name="files" 
                multiple 
                required
                on:change=move |ev| {
                    let input = match ev.target() {
                        Some(target) => target.unchecked_into::<HtmlInputElement>(),
                        None => { push_new_toast(ToastMessageType::ErrorOccurred); return }
                    };
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
