use leptos::prelude::*;
use std::time::Duration;

pub type ToastAppear = bool;

#[derive(Clone, PartialEq, Debug)]
pub struct ToastMessage {
    pub id: u32,
    pub message_type: RwSignal<ToastMessageType>,
    pub appear: RwSignal<ToastAppear>,
}

#[derive(Clone, Default, PartialEq)]
pub enum ToastMessageType {
    #[default]
    None,
    Custom(String),
    AvatarEdited,
    AvatarEditFail,
    EventCreated,
    EventCreateFail,
    EventDeleted,
    EventDeleteFail,
    EventEdited,
    EventEditFail,
    ChallengeCreated,
    ChallengeCreateFail,
    ChallengeDeleted,
    ChallengeDeleteFail,
    ChallengeEdited,
    ChallengeEditFail,
    ErrorOccurred,
    FileRenamed,
    FileRenameFail,
    InvalidCredentials,
    NoChangesMade,
    UserCreated,
    UserCreateFail,
    UserDeleted,
    UserDeleteFail,
    UserEdited,
    UserEditFail,
    UserPasswordChanged,
    UserPasswordChangeFail,
    UserUsernameChanged,
    UserUsernameChangeFail,
    VMAddedTime,
    VMAddTimeFail,
    VMDestroyed,
    VMDestroyFail,
    VMRestarted,
    VMRestartFail,
    VMStarted,
    VMStartFail,
}

impl std::fmt::Display for ToastMessageType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ToastMessageType::None => write!(f, ""),
            ToastMessageType::Custom(custom) => write!(f, "{custom}"),
            ToastMessageType::AvatarEdited => write!(f, "Successfully changed avatar"),
            ToastMessageType::AvatarEditFail => write!(f, "Failed to change avatar"),
            ToastMessageType::EventCreated => write!(f, "Successfully created new event"),
            ToastMessageType::EventCreateFail => write!(f, "Failed to create new event"),
            ToastMessageType::EventDeleted => write!(f, "Successfully deleted event"),
            ToastMessageType::EventDeleteFail => write!(f, "Failed to delete event"),
            ToastMessageType::EventEdited => write!(f, "Successfully edited event"),
            ToastMessageType::EventEditFail => write!(f, "Failed to edit event"),
            ToastMessageType::ChallengeCreated => write!(f, "Successfully created new challenge"),
            ToastMessageType::ChallengeCreateFail => write!(f, "Failed to create new challenge"),
            ToastMessageType::ChallengeDeleted => write!(f, "Successfully deleted challenge"),
            ToastMessageType::ChallengeDeleteFail => write!(f, "Failed to delete challenge"),
            ToastMessageType::ChallengeEdited => write!(f, "Successfully edited challenge"),
            ToastMessageType::ChallengeEditFail => write!(f, "Failed to edit challenge"),
            ToastMessageType::ErrorOccurred => write!(f, "An error occurred"),
            ToastMessageType::FileRenamed => write!(f, "Successfully renamed file"),
            ToastMessageType::FileRenameFail => write!(f, "Failed to rename file"),
            ToastMessageType::InvalidCredentials => write!(f, "Invalid credentials"),
            ToastMessageType::NoChangesMade => write!(f, "No changes were made"),
            ToastMessageType::UserCreated => write!(f, "Successfully created new user"),
            ToastMessageType::UserCreateFail => write!(f, "Failed to create new user"),
            ToastMessageType::UserDeleted => write!(f, "Successfully deleted user"),
            ToastMessageType::UserDeleteFail => write!(f, "Failed to delete user"),
            ToastMessageType::UserEdited => write!(f, "Successfully edited user"),
            ToastMessageType::UserEditFail => write!(f, "Failed to edit user"),
            ToastMessageType::UserPasswordChanged => write!(f, "Successfully changed password"),
            ToastMessageType::UserPasswordChangeFail => write!(f, "Failed to change password"),
            ToastMessageType::UserUsernameChanged => write!(f, "Successfully changed username"),
            ToastMessageType::UserUsernameChangeFail =>  write!(f, "Failed to change username"),
            ToastMessageType::VMAddedTime => write!(f, "Successfully added time to VM"),
            ToastMessageType::VMAddTimeFail => write!(f, "Failed to add time to VM"),
            ToastMessageType::VMDestroyed => write!(f, "Successfully destroyed VM"),
            ToastMessageType::VMDestroyFail => write!(f, "Failed to destroy VM"),
            ToastMessageType::VMRestarted => write!(f, "Successfully restarted VM"),
            ToastMessageType::VMRestartFail => write!(f, "Failed to restart VM"),
            ToastMessageType::VMStarted => write!(f, "Successfully started VM"),
            ToastMessageType::VMStartFail => write!(f, "Failed to start VM"),
        }
    }
}

#[component]
pub fn Toast(
    toast_message: ToastMessage,
) -> impl IntoView {
    let toast_messages = expect_context::<RwSignal<Vec<ToastMessage>>>();
    let entered = RwSignal::new(false);
    let mounted = RwSignal::new(false);

    let parent_classes = Memo::new(move |_| {
        let base = "flex transition-all duration-1000 ease-in-out pointer-events-none";
        if entered.get() {
            format!("{base} translate-x-0")
        } else {
            format!("{base} translate-x-72")
        }
    });

    Effect::watch(
        move || toast_message.appear.get(),
        move |appearing, prev, _| {
            if *appearing {
                mounted.set(true);
                set_timeout(move || entered.set(true), Duration::from_millis(16));
                set_timeout(move || toast_message.appear.set(false), Duration::from_secs(4));
            } else if prev.is_some() {
                entered.set(false);
                set_timeout(move || mounted.set(false), Duration::from_millis(1000));
                set_timeout(move || {
                    toast_messages.update(|t| t.retain(|tm| tm.id != toast_message.id))
                }, Duration::from_millis(1000));
            }
        },
        true
    );

    view! {
        <Show when=move || mounted.get()>
            <div 
                class=move || parent_classes.get()
            >
                <div 
                    class="flex bg-toast rounded px-8 py-4 text-text transition-all duration-1000 ease-in-out shadow-sm cursor-pointer pointer-events-auto"
                    on:click=move |_| {
                        toast_message.appear.set(false);
                    }
                >
                    {move || toast_message.message_type.get().to_string()}
                </div>
            </div>
        </Show>
    }
}

#[component]
pub fn Toasts() -> impl IntoView {
    let toast_messages = expect_context::<RwSignal<Vec<ToastMessage>>>();

    view! {
        <Show when=move || !toast_messages.get().is_empty()>
            <div class="flex flex-col gap-4 top-10 right-8 fixed h-full items-end z-50">
                <For
                    each=move || toast_messages.get()
                    key=|toast_message: &ToastMessage| toast_message.id
                    let(toast_message)
                >
                    <Toast toast_message />
                </For>
            </div>
        </Show>
    }
}

pub fn push_new_toast(message_type: ToastMessageType) {
    let toast_messages = expect_context::<RwSignal<Vec<ToastMessage>>>();

    let max_id = toast_messages.get_untracked().iter().max_by_key(|t| t.id).map(|t| t.id).unwrap_or_default() + 1;
    toast_messages.update(|t| t.push(ToastMessage { 
        id: max_id, 
        message_type: RwSignal::new(message_type), 
        appear: RwSignal::new(true) 
    }));
}
