use std::{path::PathBuf, rc::Rc};

use crate::{
    components::chat::{compose::Compose, sidebar::Sidebar as ChatSidebar, welcome::Welcome},
    layouts::slimbar::SlimbarLayout,
    utils::{
        get_drag_event,
        verify_valid_paths::{decoded_pathbufs, verify_if_are_valid_paths},
    },
};

use common::{
    language::get_local_text,
    state::{ui, Action, State},
};
use dioxus::prelude::*;
use dioxus_desktop::{use_window, wry::webview::FileDropEvent, DesktopContext};
use uuid::Uuid;

type UseEvalFn = Rc<dyn Fn(&str) -> Result<UseEval, EvalError>>;

pub const FEEDBACK_TEXT_SCRIPT: &str = r#"
    const feedback_element = document.getElementById('overlay-text');
    feedback_element.textContent = '$TEXT';
"#;

pub const ANIMATION_DASH_SCRIPT: &str = r#"
    var dashElement = document.getElementById('dash-element')
    dashElement.style.animation = "border-dance 0.5s infinite linear"
"#;

pub const SELECT_CHAT_BAR: &str = r#"
    var chatBar = document.getElementsByClassName('chatbar')[0].getElementsByClassName('input_textarea')[0]
    chatBar.focus()
"#;

pub const OVERLAY_SCRIPT: &str = r#"
    var chatLayout = document.getElementById('compose')

    var IS_DRAGGING = $IS_DRAGGING

    var overlayElement = document.getElementById('overlay-element')

    if (IS_DRAGGING) {
    chatLayout.classList.add('hover-effect')
    overlayElement.style.display = 'block'
    } else {
    chatLayout.classList.remove('hover-effect')
    overlayElement.style.display = 'none'
    }
"#;

#[allow(non_snake_case)]
pub fn ChatLayout(cx: Scope) -> Element {
    let state = use_shared_state::<State>(cx)?;
    let first_render = use_state(cx, || true);

    state.write_silent().ui.current_layout = ui::Layout::Welcome;

    let is_minimal_view = state.read().ui.is_minimal_view();
    let sidebar_hidden = state.read().ui.sidebar_hidden;
    let show_welcome = state.read().chats().active.is_none();

    if *first_render.get() && is_minimal_view {
        state.write().mutate(Action::SidebarHidden(true));
        first_render.set(false);
    }
    let drag_event: &UseRef<Option<FileDropEvent>> = use_ref(cx, || None);
    let window = use_window(cx);
    let eval: &UseEvalFn = use_eval(cx);

    // #[cfg(target_os = "windows")]
    use_future(cx, (), |_| {
        to_owned![state, window, drag_event, eval];
        async move {
            // ondragover function from div does not work on windows
            loop {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                if let FileDropEvent::Hovered { .. } = get_drag_event::get_drag_event() {
                    drop_and_attach_files(eval.clone(), &window, &drag_event, state.clone()).await;
                }
            }
        }
    });

    cx.render(rsx!(
        div {
            id: "chat-layout",
            aria_label: "chat-layout",
            div {
                id: "drag-drop-element",
            }
            div {
                id: "overlay-element",
                class: "overlay-element",
                div {id: "dash-element", class: "dash-background active-animation"},
                p {id: "overlay-text0", class: "overlay-text"},
                p {id: "overlay-text", class: "overlay-text"}
            },
            SlimbarLayout { active: crate::UplinkRoute::ChatLayout{} },
            // todo: consider showing a welcome screen if the sidebar is to be shown but there are no conversations in the sidebar. this case arises when
            // creating a new account on a mobile device.
            ChatSidebar {
                active_route: crate::UplinkRoute::ChatLayout {},
            },
            show_welcome.then(|| rsx!(Welcome {})),
            (!show_welcome && (sidebar_hidden  || !state.read().ui.is_minimal_view())).then(|| rsx!(Compose {}))
        }
    ))
}

async fn drop_and_attach_files(
    eval: UseEvalFn,
    window: &DesktopContext,
    drag_event: &UseRef<Option<FileDropEvent>>,
    state: UseSharedState<State>,
) {
    let new_files = drag_and_drop_function(eval.clone(), &window, &drag_event).await;
    let mut new_files_to_upload: Vec<_> = state
        .read()
        .get_active_chat()
        .map(|f| f.files_attached_to_send)
        .unwrap_or_default()
        .iter()
        .filter(|file_name| !new_files.contains(file_name))
        .cloned()
        .collect();
    new_files_to_upload.extend(new_files);
    let chat_uuid = state
        .read()
        .get_active_chat()
        .map(|f| f.id)
        .unwrap_or(Uuid::nil());
    state
        .write()
        .mutate(Action::SetChatAttachments(chat_uuid, new_files_to_upload));
}

// Like ui::src:layout::storage::drag_and_drop_function
async fn drag_and_drop_function(
    eval: UseEvalFn,
    window: &DesktopContext,
    drag_event: &UseRef<Option<FileDropEvent>>,
) -> Vec<PathBuf> {
    *drag_event.write_silent() = Some(get_drag_event::get_drag_event());
    let mut new_files_to_upload = Vec::new();
    loop {
        let file_drop_event = get_drag_event::get_drag_event();
        match file_drop_event {
            FileDropEvent::Hovered { paths, .. } => {
                if verify_if_are_valid_paths(&paths) {
                    let mut script = OVERLAY_SCRIPT.replace("$IS_DRAGGING", "true");
                    if paths.len() > 1 {
                        script.push_str(&FEEDBACK_TEXT_SCRIPT.replace(
                            "$TEXT",
                            &format!(
                                "{} {}!",
                                paths.len(),
                                get_local_text("files.files-to-upload")
                            ),
                        ));
                    } else {
                        script.push_str(&FEEDBACK_TEXT_SCRIPT.replace(
                            "$TEXT",
                            &format!(
                                "{} {}!",
                                paths.len(),
                                get_local_text("files.one-file-to-upload")
                            ),
                        ));
                    }
                    let _ = eval(&script);
                }
            }
            FileDropEvent::Dropped { paths, .. } => {
                if verify_if_are_valid_paths(&paths) {
                    *drag_event.write_silent() = None;
                    new_files_to_upload = decoded_pathbufs(paths);
                    let mut script = OVERLAY_SCRIPT.replace("$IS_DRAGGING", "false");
                    script.push_str(ANIMATION_DASH_SCRIPT);
                    script.push_str(SELECT_CHAT_BAR);
                    window.set_focus();
                    let _ = eval(&script);
                    break;
                }
            }
            _ => {
                *drag_event.write_silent() = None;
                let script = OVERLAY_SCRIPT.replace("$IS_DRAGGING", "false");
                let _ = eval(&script);
                break;
            }
        };
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    *drag_event.write_silent() = None;
    new_files_to_upload
}
