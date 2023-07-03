#[allow(unused_imports)]
use std::path::Path;
use std::{ffi::OsStr, path::PathBuf};

use common::icons::outline::Shape as Icon;
use common::icons::Icon as IconElement;
use common::language::get_local_text;
use common::state::ToastNotification;
use common::state::{ui, Action, State};
use common::upload_file_channel::CANCEL_FILE_UPLOADLISTENER;
use common::warp_runner::thumbnail_to_base64;
use dioxus::{html::input_data::keyboard_types::Code, prelude::*};
use dioxus_desktop::{use_window, DesktopContext};
use dioxus_router::*;
use kit::layout::modal::Modal;
use kit::{
    components::{
        context_menu::{ContextItem, ContextMenu},
        nav::Nav,
    },
    elements::{
        button::Button,
        file::File,
        folder::Folder,
        tooltip::{ArrowPosition, Tooltip},
        Appearance,
    },
    layout::topbar::Topbar,
};
use rfd::FileDialog;
use uuid::Uuid;
use warp::constellation::directory::Directory;
use warp::constellation::{file::File, item::Item};

pub mod controller;
pub mod functions;

use crate::components::chat::{sidebar::Sidebar as ChatSidebar, RouteInfo};
use crate::components::files::file_preview::FilePreview;
use crate::components::files::upload_progress_bar::UploadProgressBar;

use self::controller::StorageController;

const MAX_LEN_TO_FORMAT_NAME: usize = 64;

pub const ROOT_DIR_NAME: &str = "root";

static ALLOW_FOLDER_NAVIGATION: &str = r#"
    var folders_element = document.getElementById('files-list');
    folders_element.style.pointerEvents = '$POINTER_EVENT';
    folders_element.style.opacity = '$OPACITY';
    var folders_breadcumbs_element = document.getElementById('files-breadcrumbs');
    folders_breadcumbs_element.style.pointerEvents = '$POINTER_EVENT';
    folders_breadcumbs_element.style.opacity = '$OPACITY';
"#;

fn allow_folder_navigation(window: &DesktopContext, allow_navigation: bool) {
    let new_script = if allow_navigation {
        ALLOW_FOLDER_NAVIGATION
            .replace("$POINTER_EVENT", "")
            .replace("$OPACITY", "1")
    } else {
        ALLOW_FOLDER_NAVIGATION
            .replace("$POINTER_EVENT", "none")
            .replace("$OPACITY", "0.5")
    };
    window.eval(&new_script);
}

pub enum ChanCmd {
    GetItemsFromCurrentDirectory,
    CreateNewDirectory(String),
    OpenDirectory(String),
    BackToPreviousDirectory(Directory),
    UploadFiles(Vec<PathBuf>),
    DownloadFile {
        file_name: String,
        local_path_to_save_file: PathBuf,
    },
    RenameItem {
        old_name: String,
        new_name: String,
    },
    DeleteItems(Item),
}

#[derive(PartialEq, Props)]
pub struct Props {
    route_info: RouteInfo,
}

#[allow(non_snake_case)]
pub fn FilesLayout(cx: Scope<Props>) -> Element {
    let state = use_shared_state::<State>(cx)?;
    state.write_silent().ui.current_layout = ui::Layout::Storage;

    let storage_controller = StorageController::new(cx, state.clone());

    let storage_size: &UseRef<(String, String)> = use_ref(cx, || {
        (
            functions::format_item_size(state.read().storage.max_size),
            functions::format_item_size(state.read().storage.current_size),
        )
    });
    let is_renaming_map: &UseRef<Option<Uuid>> = use_ref(cx, || None);
    let add_new_folder = use_state(cx, || false);
    let first_render = use_state(cx, || true);
    let show_file_modal: &UseState<Option<File>> = use_state(cx, || None);
    let are_files_hovering_app = use_ref(cx, || false);
    let files_been_uploaded = use_ref(cx, || false);
    let files_in_queue_to_upload: &UseRef<Vec<PathBuf>> =
        use_ref(cx, || state.read().storage.files_in_queue_to_upload.clone());
    let disable_cancel_upload_button = use_ref(cx, || false);

    let window = use_window(cx);

    let ch: &Coroutine<ChanCmd> = functions::init_coroutine(cx, storage_controller.storage_state);
    allow_folder_navigation(window, files_in_queue_to_upload.read().is_empty());

    functions::run_verifications_and_update_storage(
        first_render,
        state,
        storage_controller.clone(),
        storage_size,
        files_in_queue_to_upload,
    );

    functions::get_items_from_current_directory(cx, ch);
    #[cfg(not(target_os = "macos"))]
    functions::allow_drag_event_for_non_macos_systems(cx, drag_event, window, main_script, ch);

    let storage_state = storage_controller.storage_state.clone();
    functions::start_upload_file_listener(
        cx,
        files_been_uploaded,
        window,
        &storage_state,
        state,
        first_render,
        files_in_queue_to_upload,
        disable_cancel_upload_button,
    );

    let tx_cancel_file_upload = CANCEL_FILE_UPLOADLISTENER.tx.clone();

    cx.render(rsx!(
        div {
            id: "overlay-element",
            class: "overlay-element",
            div {id: "dash-element", class: "dash-background active-animation"},
            p {id: "overlay-text0", class: "overlay-text"},
            p {id: "overlay-text", class: "overlay-text"}
        },
        if let Some(file) = show_file_modal.current().as_ref().clone() {
            let file2 = file.clone();
            rsx!(
                get_file_modal {
                    on_dismiss: |_| {
                        show_file_modal.set(None);
                    },
                    on_download: move |_| {
                        let file_name = file2.clone().name();
                        download_file(&file_name, ch);
                    }
                    file: file.clone()
                }
            )
        }
        div {
            id: "files-layout",
            color: "red",
            aria_label: "files-layout",
            ondragover: move |_| {
                    if are_files_hovering_app.with(|i| *i == false) {
                        are_files_hovering_app.with_mut(|i| *i = true);
                    };
                },
            onclick: |_| {
                add_new_folder.set(false);
                is_renaming_map.with_mut(|i| *i = None);
            },
            ChatSidebar {
                route_info: cx.props.route_info.clone()
            },
            div {
                class: "files-body disable-select",
                aria_label: "files-body",
                Topbar {
                    with_back_button: state.read().ui.is_minimal_view() || state.read().ui.sidebar_hidden,
                    onback: move |_| {
                        let current = state.read().ui.sidebar_hidden;
                        state.write().mutate(Action::SidebarHidden(!current));
                    },
                    controls: cx.render(
                        rsx! (
                            Button {
                                icon: Icon::FolderPlus,
                                appearance: Appearance::Secondary,
                                aria_label: "add-folder".into(),
                                tooltip: cx.render(rsx!(
                                    Tooltip {
                                        arrow_position: ArrowPosition::Top,
                                        text: get_local_text("files.new-folder"),
                                    }
                                )),
                                onpress: move |_| {
                                    is_renaming_map.with_mut(|i| *i = None);
                                    add_new_folder.set(!add_new_folder);
                                },
                            },
                            Button {
                                icon: Icon::Plus,
                                appearance: Appearance::Secondary,
                                aria_label: "upload-file".into(),
                                tooltip: cx.render(rsx!(
                                    Tooltip {
                                        arrow_position: ArrowPosition::Top,
                                        text: get_local_text("files.upload"),
                                    }
                                ))
                                onpress: move |_| {
                                    is_renaming_map.with_mut(|i| *i = None);
                                    let files_local_path = match FileDialog::new().set_directory(".").pick_files() {
                                        Some(path) => path,
                                        None => return
                                    };
                                    add_files_in_queue_to_upload(files_in_queue_to_upload, files_local_path, window, ch);
                                    files_been_uploaded.with_mut(|i| *i = true);
                                },
                            }
                        )
                    ),
                    div {
                        class: "files-info",
                        aria_label: "files-info",
                        if storage_size.read().0.is_empty() {
                            rsx!(div {
                                class: "skeletal-texts",
                                div {
                                    class: "skeletal-text",
                                    div {
                                        class: "skeletal-text-content skeletal",
                                    }
                                },
                            },
                            div {
                                class: "skeletal-texts",
                                div {
                                    class: "skeletal-text",
                                    div {
                                        class: "skeletal-text-content skeletal",
                                    }
                                },
                            })
                        } else {
                            rsx!(
                                p {
                                    class: "free-space",
                                    aria_label: "free-space-max-size",
                                    format!("{}", get_local_text("files.storage-max-size")),
                                    span {
                                        class: "count",
                                       format!("{}", storage_size.read().0),
                                    }
                                },
                                p {
                                    class: "free-space",
                                    aria_label: "free-space-current-size",
                                    format!("{}", get_local_text("files.storage-current-size")),
                                    span {
                                        class: "count",
                                       format!("{}", storage_size.read().1),
                                    }
                                },
                            )
                        }
                    }
                }
                UploadProgressBar {
                    are_files_hovering_app: are_files_hovering_app,
                    files_been_uploaded: files_been_uploaded,
                    disable_cancel_upload_button: disable_cancel_upload_button,
                    on_update: move |files_to_upload: Vec<PathBuf>|  {
                        add_files_in_queue_to_upload(files_in_queue_to_upload, files_to_upload, window, ch);
                    },
                    on_cancel: move |_| {
                        let _ = tx_cancel_file_upload.send(true);
                        let _ = tx_cancel_file_upload.send(false);
                    },
                }
                div {
                    id: "files-breadcrumbs",
                    class: "files-breadcrumbs",
                    aria_label: "files-breadcrumbs",
                    storage_controller.dirs_opened_ref.read().iter().enumerate().map(|(index, dir)| {
                        let directory = dir.clone();
                        let dir_name = dir.name();
                        if dir_name == ROOT_DIR_NAME && index == 0 {
                            let home_text = get_local_text("uplink.home");
                            rsx!(div {
                                class: "crumb",
                                aria_label: "crumb",
                                onclick: move |_| {
                                    ch.send(ChanCmd::BackToPreviousDirectory(directory.clone()));
                                },
                                IconElement {
                                    icon: Icon::Home,
                                },
                                p {
                                    aria_label: "home-dir",
                                    "{home_text}",
                                }
                            })
                        } else {
                            let folder_name_formatted = functions::format_item_name(dir_name);
                            rsx!(div {
                                class: "crumb",
                                onclick: move |_| {
                                    ch.send(ChanCmd::BackToPreviousDirectory(directory.clone()));
                                },
                                aria_label: "crumb",
                                p {
                                    aria_label: "{folder_name_formatted}",
                                    "{folder_name_formatted}"
                                }
                            },)
                        }
                    })
                },
                span {
                    class: "file-parent",
                    div {
                        id: "files-list",
                        class: "files-list",
                        aria_label: "files-list",
                        add_new_folder.then(|| {
                            rsx!(
                            Folder {
                                with_rename: true,
                                onrename: |(val, key_code)| {
                                    let new_name: String = val;
                                    if storage_controller.directories_list.read().iter().any(|dir| dir.name() == new_name) {
                                        state
                                        .write()
                                        .mutate(common::state::Action::AddToastNotification(
                                            ToastNotification::init(
                                                "".into(),
                                                get_local_text("files.directory-already-with-name"),
                                                None,
                                                3,
                                            ),
                                        ));
                                        return;
                                    }
                                    if key_code == Code::Enter {
                                        ch.send(ChanCmd::CreateNewDirectory(new_name));
                                        ch.send(ChanCmd::GetItemsFromCurrentDirectory);
                                    }
                                    add_new_folder.set(false);
                                 }
                            })
                        }),
                        storage_controller.directories_list.read().iter().map(|dir| {
                            let folder_name = dir.name();
                            let folder_name2 = dir.name();
                            let key = dir.id();
                            let dir2 = dir.clone();
                            rsx!(
                                ContextMenu {
                                    key: "{key}-menu",
                                    id: dir.id().to_string(),
                                    items: cx.render(rsx!(
                                        ContextItem {
                                            icon: Icon::Pencil,
                                            aria_label: "folder-rename".into(),
                                            text: get_local_text("files.rename"),
                                            onpress: move |_| {
                                                is_renaming_map.with_mut(|i| *i = Some(key));
                                            }
                                        },
                                        hr {},
                                        ContextItem {
                                            icon: Icon::Trash,
                                            danger: true,
                                            aria_label: "folder-delete".into(),
                                            text: get_local_text("uplink.delete"),
                                            onpress: move |_| {
                                                let item = Item::from(dir2.clone());
                                                ch.send(ChanCmd::DeleteItems(item));
                                            }
                                        },
                                    )),
                                    Folder {
                                        key: "{key}-folder",
                                        text: dir.name(),
                                        aria_label: dir.name(),
                                        with_rename: *is_renaming_map.read() == Some(key),
                                        onrename: move |(val, key_code)| {
                                            if storage_controller.directories_list.read().iter().any(|dir| dir.name() == val) {
                                                state
                                                .write()
                                                .mutate(common::state::Action::AddToastNotification(
                                                    ToastNotification::init(
                                                        "".into(),
                                                        get_local_text("files.directory-already-with-name"),
                                                        None,
                                                        3,
                                                    ),
                                                ));
                                                return;
                                            }
                                            is_renaming_map.with_mut(|i| *i = None);
                                            if key_code == Code::Enter {
                                                ch.send(ChanCmd::RenameItem{old_name: folder_name2.clone(), new_name: val});
                                            }
                                        }
                                        onpress: move |_| {
                                            is_renaming_map.with_mut(|i| *i = None);
                                            ch.send(ChanCmd::OpenDirectory(folder_name.clone()));
                                        }
                                    }
                                }
                            )
                        }),
                        storage_controller.files_list.read().iter().map(|file| {
                            let file_name = file.name();
                            let file_name2 = file.name();
                            let file2 = file.clone();
                            let file3 = file.clone();
                            let key = file.id();
                            let file_id = file.id();
                            rsx!(ContextMenu {
                                        key: "{key}-menu",
                                        id: file.id().to_string(),
                                        items: cx.render(rsx!(
                                            ContextItem {
                                                icon: Icon::Pencil,
                                                aria_label: "files-rename".into(),
                                                text: get_local_text("files.rename"),
                                                onpress: move |_| {
                                                    is_renaming_map.with_mut(|i| *i = Some(key));
                                                }
                                            },
                                            ContextItem {
                                                icon: Icon::ArrowDownCircle,
                                                aria_label: "files-download".into(),
                                                text: get_local_text("files.download"),
                                                onpress: move |_| {
                                                    download_file(&file_name2, ch);
                                                },
                                            },
                                            hr {},
                                            ContextItem {
                                                icon: Icon::Trash,
                                                danger: true,
                                                aria_label: "files-delete".into(),
                                                text: get_local_text("uplink.delete"),
                                                onpress: move |_| {
                                                    let item = Item::from(file2.clone());
                                                    ch.send(ChanCmd::DeleteItems(item));
                                                }
                                            },
                                        )),
                                        File {
                                            key: "{key}-file",
                                            thumbnail: thumbnail_to_base64(file),
                                            text: file.name(),
                                            aria_label: file.name(),
                                            with_rename: *is_renaming_map.read() == Some(key),
                                            onpress: move |_| {
                                                let key = file_id;
                                                if state.read().ui.file_previews.contains_key(&key) {
                                                    state
                                                    .write()
                                                    .mutate(common::state::Action::AddToastNotification(
                                                        ToastNotification::init(
                                                            "".into(),
                                                            get_local_text("files.file-already-opened"),
                                                            None,
                                                            2,
                                                        ),
                                                    ));
                                                    return;
                                                }
                                                if file3.thumbnail().is_empty() {
                                                    state
                                                    .write()
                                                    .mutate(common::state::Action::AddToastNotification(
                                                        ToastNotification::init(
                                                            "".into(),
                                                            get_local_text("files.no-thumbnail-preview"),
                                                            None,
                                                            3,
                                                        ),
                                                    ));
                                                    return;
                                                }

                                                let file4 = file3.clone();
                                                show_file_modal.set(Some(file4));
                                            },
                                            onrename: move |(val, key_code)| {
                                                let new_name: String = val;
                                                if  storage_controller.files_list.read().iter().any(|file| file.name() == new_name) {
                                                    state
                                                    .write()
                                                    .mutate(common::state::Action::AddToastNotification(
                                                        ToastNotification::init(
                                                            "".into(),
                                                            get_local_text("files.file-already-with-name"),
                                                            None,
                                                            3,
                                                        ),
                                                    ));
                                                    return;
                                                }
                                                is_renaming_map.with_mut(|i| *i = None);
                                                if key_code == Code::Enter && !new_name.is_empty() && !new_name.chars().all(char::is_whitespace) {
                                                    ch.send(ChanCmd::RenameItem{old_name: file_name.clone(), new_name});
                                                }
                                            }
                                        }
                                    }
                              )
                        }),
                    },
                }

                (state.read().ui.sidebar_hidden && state.read().ui.metadata.minimal_view).then(|| rsx!(
                    Nav {
                        routes: cx.props.route_info.routes.clone(),
                        active: cx.props.route_info.active.clone(),
                        onnavigate: move |r| {
                            use_router(cx).replace_route(r, None, None);
                        }
                    }
                ))
            }
        }
    ))
}

// TODO: This really shouldn't be in this file
#[inline_props]
pub fn get_file_modal<'a>(
    cx: Scope<'a>,
    on_dismiss: EventHandler<'a, ()>,
    on_download: EventHandler<'a, ()>,
    file: File,
) -> Element<'a> {
    cx.render(rsx!(Modal {
        onclose: move |_| on_dismiss.call(()),
        open: true,
        children: cx.render(rsx!(FilePreview {
            file: file,
            on_download: |_| {
                on_download.call(());
            },
        }))
    }))
}

fn download_file(file_name: &str, ch: &Coroutine<ChanCmd>) {
    let file_extension = std::path::Path::new(&file_name)
        .extension()
        .and_then(OsStr::to_str)
        .map(|s| s.to_string())
        .unwrap_or_default();
    let file_stem = PathBuf::from(&file_name)
        .file_stem()
        .and_then(OsStr::to_str)
        .map(str::to_string)
        .unwrap_or_default();
    let file_path_buf = match FileDialog::new()
        .set_directory(".")
        .set_file_name(&file_stem)
        .add_filter("", &[&file_extension])
        .save_file()
    {
        Some(path) => path,
        None => return,
    };
    ch.send(ChanCmd::DownloadFile {
        file_name: file_name.to_string(),
        local_path_to_save_file: file_path_buf,
    });
}

fn add_files_in_queue_to_upload(
    files_in_queue_to_upload: &UseRef<Vec<PathBuf>>,
    files_path: Vec<PathBuf>,
    window: &DesktopContext,
    ch: &Coroutine<ChanCmd>,
) {
    allow_folder_navigation(window, false);
    files_in_queue_to_upload
        .write_silent()
        .extend(files_path.clone());
    ch.send(ChanCmd::UploadFiles(files_path));
}
