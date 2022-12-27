use dioxus::{prelude::*, desktop::use_window};
use kit::{User as UserInfo, elements::{input::{Input, Options}, label::Label}, icons::Icon, components::{nav::Nav, context_menu::{ContextMenu, ContextItem}, user::User, user_image::UserImage, indicator::{Platform, Status}, user_image_group::UserImageGroup}, layout::sidebar::Sidebar as ReusableSidebar};
use warp::raygun::Message;

use crate::{components::{chat::RouteInfo, media::remote_control::RemoteControls}, state::{State, Action, Chat, Identity}, utils::language::get_local_text};

#[derive(PartialEq, Props)]
pub struct Props {
    route_info: RouteInfo,
}

pub fn build_participants(identities: &Vec<Identity>) -> Vec<UserInfo> {
    // Create a vector of UserInfo objects to store the results
    let mut user_info: Vec<UserInfo> = vec![];

    // Iterate over the identities vector
    for identity in identities {
        // For each identity, create a new UserInfo object and set its fields
        // to the corresponding values from the identity object
        user_info.push(UserInfo {
            platform: Platform::Mobile,
            status: Status::Online,
            username: identity.username(),
            photo: identity.graphics().profile_picture(),
        })
    }

    // Return the resulting user_info vector
    user_info
}

pub fn build_participants_names(identities: &Vec<Identity>) -> String {
    let mut participants_name = String::from("");

    // Iterate over the identities vector
    for identity in identities {
        // Create a string with the username of the current identity and a comma
        let name = format!("{}, ", identity.username());
        // Append the name string to the participants_name string
        participants_name.push_str(&name);
    }

    // Remove the last two characters from the participants_name string (the trailing comma and space)
    participants_name.pop();
    participants_name.pop();

    // Return the resulting participants_name string
    participants_name
}


#[allow(non_snake_case)]
pub fn Sidebar(cx: Scope<Props>) -> Element {
    let state: UseSharedState<State> = use_context::<State>(&cx)?;

    let sidebar_chats = state.read().chats.in_sidebar.clone();

    let favorites = state.read().chats.favorites.clone();

    let binding = state.read();
    let active_media_chat = binding.get_active_media_chat();

    let desktop = use_window(&cx);

    cx.render(rsx!(
        ReusableSidebar {
            with_search: cx.render(rsx!(
                div {
                    class: "search-input",
                    Input {
                        placeholder: get_local_text("uplink.search-placeholder"),
                        // TODO: Pending implementation
                        disabled: true,
                        icon: Icon::MagnifyingGlass,
                        options: Options {
                            with_clear_btn: true,
                            ..Options::default()
                        }
                    }
                }
            ))
            with_nav: cx.render(rsx!(
                Nav {
                    routes: cx.props.route_info.routes.clone(),
                    active: cx.props.route_info.active.clone(),
                    onnavigate: move |r| {
                        use_router(&cx).replace_route(r, None, None);
                    }
                },
            )),
            div {
                class: "drag-handle",
                onmousedown: move |_| desktop.drag(),
            },
            // Only display favorites if we have some.
            (!favorites.is_empty()).then(|| rsx!(
                div {
                    id: "favorites",
                    Label {
                        text: get_local_text("favorites.favorites"),
                    },
                    div {
                        class: "vertically-scrollable",
                        favorites.iter().cloned().map(|chat_id| {
                            let default_chat = Chat::default();
                            let chat = state.read().chats.all.get(&chat_id).unwrap_or(&default_chat).clone();
                            let favorites_chat = chat.clone();
                            let remove_favorite = chat.clone();
                            let without_me = state.read().get_without_me(chat.participants.clone());
                            let participants_name = build_participants_names(&without_me);
                            
                            rsx! (
                                ContextMenu {
                                    key: "{chat_id}-favorite",
                                    id: chat_id.to_string(),
                                    items: cx.render(rsx!(
                                        ContextItem {
                                            icon: Icon::ChatBubbleBottomCenterText,
                                            text: get_local_text("uplink.chat"),
                                            onpress: move |_| {
                                                state.write().mutate(Action::ChatWith(favorites_chat.clone()));
                                                if cx.props.route_info.active.to != "/" {
                                                    use_router(&cx).replace_route("/", None, None);
                                                }
                                            }
                                        },
                                        ContextItem {
                                            icon: Icon::XMark,
                                            text: get_local_text("favorites.remove"),
                                            onpress: move |_| {
                                                state.write().mutate(Action::ToggleFavorite(remove_favorite.clone()));
                                            }
                                        }
                                    )),
                                    UserImageGroup {
                                        // loading: true,
                                        participants: build_participants(&chat.participants.clone()),
                                        with_username: participants_name,
                                        onpress: move |_| {
                                            state.write().mutate(Action::ChatWith(chat.clone()));
                                            if cx.props.route_info.active.to != "/" {
                                                use_router(&cx).replace_route("/", None, None);
                                            }
                                        }
                                    }
                                }
                            )
                        })
                    }
                }
            )),
            div {
                id: "chats",
                (!sidebar_chats.is_empty()).then(|| rsx!(
                    Label {
                        text: get_local_text("uplink.chats"),
                    }
                )),
                sidebar_chats.iter().cloned().map(|chat_id| {
                    let chat = state.read().chats.all.get(&chat_id).unwrap().clone();
                    let without_me = state.read().get_without_me(chat.participants.clone());
                    let user = without_me.first();
                    let default_message = Message::default();
                    let parsed_user = match user {
                        Some(u) => u.clone(),
                        None => Identity::default(),
                    };

                    let last_message = chat.messages.last();
                    let unwrapped_message = match last_message {
                        Some(m) => m,
                        None => &default_message,
                    };

                    let val = unwrapped_message.value();
                    let timestamp = unwrapped_message.date().timestamp_millis() as u64;

                    let badge = if chat.unreads > 0 {
                        chat.unreads.to_string()
                    } else { "".into() };
                    
                    let key = chat.id;

                    let active = state.read().get_active_chat().unwrap_or_default().id == chat.id;
                    let chat_with = chat.clone();
                    let clear_unreads = chat.clone();

                    let participants = without_me.clone();
                    let participants_name = if participants.len() > 2 { build_participants_names(&participants) } else { parsed_user.username() };

                    // TODO:
                    // let _block_user_text = LOCALES
                    //     .lookup(&*APP_LANG.read(), "friends.block")
                    //     .unwrap_or_default();

                    rsx!(
                        ContextMenu {
                            key: "{key}-chat",
                            id: format!("{}-chat", key),
                            items: cx.render(rsx!(
                                ContextItem {
                                    icon: Icon::BellSlash,
                                    text: get_local_text("uplink.clear-unreads"),
                                    onpress: move |_| {
                                        state.write().mutate(Action::ClearUnreads(clear_unreads.clone()));
                                    }
                                },
                                hr{ },
                                ContextItem {
                                    icon: Icon::PhoneArrowUpRight,
                                    text: get_local_text("uplink.call"),
                                    //TODO: Wire to state

                                },
                                hr{ }
                                ContextItem {
                                    icon: Icon::EyeSlash,
                                    text: get_local_text("uplink.hide-chat"),
                                    onpress: move |_| {
                                        state.write().mutate(Action::RemoveFromSidebar(chat.clone()));
                                    }
                                },
                            )),
                            User {
                                // loading: true,
                                username: participants_name,
                                subtext: val.join("\n"),
                                timestamp: timestamp,
                                active: active,
                                user_image: cx.render(rsx!(
                                    if participants.len() <= 2 {rsx! (
                                        UserImage {
                                            platform: Platform::Mobile,
                                            status: Status::Online
                                            image: parsed_user.graphics().profile_picture(),
                                        }
                                    )} else {rsx! (
                                        UserImageGroup {
                                            participants: build_participants(&participants)
                                        }
                                    )}
                                )),
                                with_badge: badge,
                                onpress: move |_| {
                                    state.write().mutate(Action::ChatWith(chat_with.clone()));
                                    if cx.props.route_info.active.to != "/" {
                                        use_router(&cx).replace_route("/", None, None);
                                    }
                                }
                            }
                        }
                    )}
                )
            },
            active_media_chat.is_some().then(|| rsx!(
                RemoteControls {
                    in_call_text: get_local_text("remote-controls.in-call"),
                    mute_text: get_local_text("remote-controls.mute"),
                    unmute_text: get_local_text("remote-controls.unmute"),
                    listen_text: get_local_text("remote-controls.listen"),
                    silence_text: get_local_text("remote-controls.silence"),
                   end_text: get_local_text("remote-controls.end"),
                }
            )),
        }
    ))
}
