use common::{language::get_local_text_args_builder, MAX_FILES_PER_MESSAGE};
use dioxus::prelude::*;
use kit::elements::{button::Button, checkbox::Checkbox, Appearance};
use warp::raygun::Location;

use super::controller::StorageController;

#[inline_props]
pub fn file_checkbox(
    cx: Scope<'a>,
    file_path: String,
    storage_controller: UseRef<StorageController>,
    select_files_to_send_mode: UseState<bool>,
) -> Element<'a> {
    if *select_files_to_send_mode.get() {
        let files_selected_to_send = storage_controller.with(|f| f.files_selected_to_send.clone());
        return cx.render(rsx!( div {
            class: "checkbox-position",
            Checkbox {
                disabled: files_selected_to_send.len() >= MAX_FILES_PER_MESSAGE,
                width: "1em".into(),
                height: "1em".into(),
                is_checked:files_selected_to_send.iter()
                .any(|location| {
                    match location {
                        Location::Constellation { path } => path == file_path,
                        Location::Disk { .. } => false,
                    }
                }),
                on_click: move |_| {
                    add_remove_file_to_send(storage_controller.clone(), file_path.clone());
                }
            }
        },));
    }
    None
}

#[inline_props]
pub fn send_files_from_chat_topbar<'a>(
    cx: Scope<'a>,
    storage_controller: UseRef<StorageController>,
    select_files_to_send_mode: UseState<bool>,
    on_press_send_files_button: EventHandler<'a, Vec<Location>>,
) -> Element<'a> {
    if *select_files_to_send_mode.get() {
        return cx.render(rsx! (div {
            class: "send-files-top-stripe",
            div {
                class: "send-files-button",
                Button {
                    text: get_local_text_args_builder("files.send-files-text-amount", |m| {
                        m.insert("amount", format!("{}/{}", storage_controller.with(|f| f.files_selected_to_send.clone()).len(), MAX_FILES_PER_MESSAGE).into());
                    }),
                    aria_label: "send_files_modal_send_button".into(),
                    appearance: Appearance::Success,
                    onpress: move |_| {
                        on_press_send_files_button.call(storage_controller.with(|f| f.files_selected_to_send.clone()));
                        select_files_to_send_mode.set(false);
                    }
                },
            }
            p {
                class: "files-selected-text",
                get_local_text_args_builder("files.files-selected-paths", |m| {
                    m.insert("files_path",storage_controller.with(|f| f.files_selected_to_send.clone()).into_iter()
                    .filter(|location| matches!(location, Location::Constellation { .. }))
                    .map(|location| {
                        if let Location::Constellation { path } = location {
                            path
                        } else {
                            String::new()
                        }
                    })
                    .collect::<Vec<String>>()
                    .join(", ").into());
                })
            }
        }));
    }
    None
}

pub fn add_remove_file_to_send(storage_controller: UseRef<StorageController>, file_path: String) {
    if let Some(index) = storage_controller.with(|f| {
        f.files_selected_to_send.iter().position(|location| {
            let path = match location {
                Location::Constellation { path } => path.clone(),
                _ => String::new(),
            };
            path == file_path.clone()
        })
    }) {
        storage_controller.with_mut(|f| f.files_selected_to_send.remove(index));
    } else if storage_controller.with(|f| f.files_selected_to_send.len() < MAX_FILES_PER_MESSAGE) {
        storage_controller.with_mut(|f| {
            f.files_selected_to_send.push(Location::Constellation {
                path: file_path.clone(),
            })
        });
    }
}
