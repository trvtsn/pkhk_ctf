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
        let s = match self {
            ToastMessageType::None => "",
            ToastMessageType::Custom(custom) => custom.as_ref(),
            ToastMessageType::AvatarEdited => "Successfully changed avatar",
            ToastMessageType::AvatarEditFail => "Failed to change avatar",
            ToastMessageType::EventCreated => "Successfully created new event",
            ToastMessageType::EventCreateFail => "Failed to create new event",
            ToastMessageType::EventDeleted => "Successfully deleted event",
            ToastMessageType::EventDeleteFail => "Failed to delete event",
            ToastMessageType::EventEdited => "Successfully edited event",
            ToastMessageType::EventEditFail => "Failed to edit event",
            ToastMessageType::ChallengeCreated => "Successfully created new challenge",
            ToastMessageType::ChallengeCreateFail => "Failed to create new challenge",
            ToastMessageType::ChallengeDeleted => "Successfully deleted challenge",
            ToastMessageType::ChallengeDeleteFail => "Failed to delete challenge",
            ToastMessageType::ChallengeEdited => "Successfully edited challenge",
            ToastMessageType::ChallengeEditFail => "Failed to edit challenge",
            ToastMessageType::FileRenamed => "Successfully renamed file",
            ToastMessageType::FileRenameFail => "Failed to rename file",
            ToastMessageType::InvalidCredentials => "Invalid credentials",
            ToastMessageType::NoChangesMade => "No changes were made",
            ToastMessageType::UserCreated => "Successfully created new user",
            ToastMessageType::UserCreateFail => "Failed to create new user",
            ToastMessageType::UserDeleted => "Successfully deleted user",
            ToastMessageType::UserDeleteFail => "Failed to delete user",
            ToastMessageType::UserEdited => "Successfully edited user",
            ToastMessageType::UserEditFail => "Failed to edit user",
            ToastMessageType::UserPasswordChanged => "Successfully changed password",
            ToastMessageType::UserPasswordChangeFail => "Failed to change password",
            ToastMessageType::UserUsernameChanged => "Successfully changed username",
            ToastMessageType::UserUsernameChangeFail => "Failed to change username",
            ToastMessageType::VMAddedTime => "Successfully added time to VM",
            ToastMessageType::VMAddTimeFail => "Failed to add time to VM",
            ToastMessageType::VMDestroyed => "Successfully destroyed VM",
            ToastMessageType::VMDestroyFail => "Failed to destroy VM",
            ToastMessageType::VMRestarted => "Successfully restarted VM",
            ToastMessageType::VMRestartFail => "Failed to restart VM",
            ToastMessageType::VMStarted => "Successfully started VM",
            ToastMessageType::VMStartFail => "Failed to start VM",
            
        };
        write!(f, "{s}")
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

    Effect::new(move |_| {
        if toast_message.appear.get() {
            mounted.set(true);
            set_timeout(move || entered.set(true), Duration::from_millis(16));
            set_timeout(move || toast_message.appear.set(false), Duration::from_secs(4));
        } else {
            entered.set(false);
            set_timeout(move || mounted.set(false), Duration::from_millis(1000));
            set_timeout(move || {
                toast_messages.update(|t| t.retain(|tm| tm.id != toast_message.id))
            }, Duration::from_millis(1000));
        }
    });

    view! {
        <Show when=move || mounted.get()>
            <div 
                class=move || parent_classes.get()
            >
                <div 
                    class="flex bg-toast rounded px-8 py-4 text-text transition-all duration-1000 ease-in-out shadow-sm cursor-pointer z-50 pointer-events-auto"
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
            <div class="flex flex-col gap-4 top-10 right-8 fixed h-full items-center">
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

