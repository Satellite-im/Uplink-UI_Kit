#[allow(unused_imports)]
use std::collections::{BTreeMap, HashMap, HashSet};

use common::{
    icons::outline::Shape as Icon,
    language::get_local_text,
    state::{Identity, State},
    warp_runner::{RayGunCmd, WarpCmd},
    WARP_CMD_CH,
};
use dioxus::prelude::*;
use futures::{channel::oneshot, StreamExt};
use kit::{
    components::user_image::UserImage,
    elements::{
        button::Button,
        input::{Input, Options},
    },
    layout::topbar::Topbar,
};
use uuid::Uuid;
use warp::{crypto::DID, logging::tracing::log};

#[derive(PartialEq, Clone)]
enum EditGroupAction {
    Add,
    Remove,
}

enum ChanCmd {
    AddParticipants,
    RemoveParticipants,
}

#[allow(non_snake_case)]
pub fn EditGroup(cx: Scope) -> Element {
    log::trace!("rendering edit_group");
    let state = use_shared_state::<State>(cx)?;
    let friend_prefix = use_state(cx, String::new);
    let selected_friends: &UseState<HashSet<DID>> = use_state(cx, HashSet::new);
    let edit_group_action = use_state(cx, || EditGroupAction::Remove);
    let conv_id = state.read().get_active_chat().unwrap().id;
    let friends_did_already_in_group = state.read().get_active_chat().unwrap().participants;
    let friends_list = HashMap::from_iter(
        state
            .read()
            .friend_identities()
            .iter()
            .map(|id| (id.did_key(), id.clone())),
    );
    let mut friends_group_list = friends_list.clone();
    let mut friends_not_in_group_list = friends_list;

    friends_group_list.retain(|did_key, _| friends_did_already_in_group.contains(did_key));
    friends_not_in_group_list.retain(|did_key, _| !friends_did_already_in_group.contains(did_key));

    let _friends_not_in_group = State::get_friends_by_first_letter(friends_not_in_group_list);
    let _friends_in_group = State::get_friends_by_first_letter(friends_group_list);

    let add_friends = rsx!(a {
        class: "float-right-link",
        onclick: move |_| {
            edit_group_action.set(EditGroupAction::Add);
        },
        format!(
            "{} →",
            get_local_text("uplink.add-members")
        ),
    });

    let remove_friends = rsx!(a {
        class: "float-right-link",
        onclick: move |_| {
            edit_group_action.set(EditGroupAction::Remove);
        },
        format!(
            "{} →",
            get_local_text("uplink.current-members")
        ),
    });

    let friends = if *edit_group_action.get() == EditGroupAction::Add {
        _friends_not_in_group
    } else {
        _friends_in_group
    };

    cx.render(rsx!(
        div {
            id: "edit-group",
            aria_label: "edit-group",
        Topbar {
                with_back_button: false,
                div {
                    class: "search-input",
                    Input {
                        // todo: filter friends on input
                        placeholder: get_local_text("uplink.search-placeholder"),
                        disabled: false,
                        aria_label: "friend-search-input".into(),
                        icon: Icon::MagnifyingGlass,
                        options: Options {
                            with_clear_btn: true,
                            react_to_esc_key: true,
                            clear_on_submit: false,
                            ..Options::default()
                        },
                        onchange: move |(v, _): (String, _)| {
                            friend_prefix.set(v);
                        },
                    }
                    if *edit_group_action.get() == EditGroupAction::Remove {
                        rsx! {
                            add_friends,
                        }
                        } else {
                            rsx! {
                            remove_friends,
                            }
                        },
                },

            },
            rsx!(
                div {
                    class: "friend-list vertically-scrollable",
                    aria_label: "friends-list",
                    friends.iter().map(
                        |(letter, sorted_friends)| {
                            let group_letter = letter.to_string();
                            rsx!(
                                div {
                                    key: "friend-group-{group_letter}",
                                    class: "friend-group",
                                    aria_label: "friend-group",
                                    sorted_friends.iter().filter(|friend| {
                                        let name = friend.username().to_lowercase();
                                        if name.len() < friend_prefix.len() {
                                            false
                                        } else {
                                            name[..(friend_prefix.len())] == friend_prefix.to_lowercase()
                                        }
                                    } ).map(|_friend| {

                                        // let friend = _friend.clone();
                                        rsx!(
                                            friend_row {
                                                add_or_remove: "add".into(),
                                                friend: _friend.clone(),
                                                conv_id: conv_id,
                                            }
                                        )
                                        // rsx!(
                                        //     div {
                                        //         class: "friend-container",
                                        //         aria_label: "Friend Container",
                                        //         UserImage {
                                        //             platform: _friend.platform().into(),
                                        //             status: _friend.identity_status().into(),
                                        //             image: _friend.profile_picture()
                                        //         },
                                        //         div {
                                        //             class: "flex-1",
                                        //             p {
                                        //                 aria_label: "friend-username",
                                        //                 _friend.username(),
                                        //             },
                                        //         },
                                        //         Button {
                                        //             aria_label: get_local_text("uplink.remove"),
                                        //             icon: if *edit_group_action.current() == EditGroupAction::Add {
                                        //                 Icon::UserPlus
                                        //             } else {
                                        //                 Icon::UserMinus
                                        //             },
                                        //             text: if *edit_group_action.current() == EditGroupAction::Add {
                                        //                 get_local_text("uplink.add")
                                        //             } else {
                                        //                 get_local_text("uplink.remove")
                                        //             },
                                        //             onpress: move |_| {
                                        //                 let mut friends = selected_friends.get().clone();
                                        //                 friends.clear();
                                        //                 selected_friends.set(vec![friend.did_key()].into_iter().collect());
                                        //                 if *edit_group_action.current() == EditGroupAction::Add {
                                        //                     ch.send(ChanCmd::AddParticipants);
                                        //                 } else {
                                        //                     ch.send(ChanCmd::RemoveParticipants);
                                        //                 }
                                        //             }
                                        //         }
                                        //     }
                                        // )

                                    })
                                }
                            )
                        }
                    ),
                }
            )
        }
    ))
}

// #[derive(PartialEq, Props)]
pub struct FriendRowProps {
    add_or_remove: String,
    friend: Identity,
    conv_id: Uuid,
}
fn friend_row(cx: Scope<FriendRowProps>) -> Element {
    let _friend = cx.props.friend;
    let edit_group_action = use_state(cx, || EditGroupAction::Remove);
    let selected_friends: &UseState<HashSet<DID>> = use_state(cx, HashSet::new);
    let conv_id = cx.props.conv_id;
    let ch = use_coroutine(cx, |mut rx: UnboundedReceiver<ChanCmd>| {
        to_owned![selected_friends, conv_id];
        async move {
            let warp_cmd_tx = WARP_CMD_CH.tx.clone();
            while let Some(cmd) = rx.next().await {
                match cmd {
                    ChanCmd::AddParticipants => {
                        let recipients: Vec<DID> =
                            selected_friends.current().iter().cloned().collect();
                        let (tx, rx) = oneshot::channel();
                        if let Err(e) =
                            warp_cmd_tx.send(WarpCmd::RayGun(RayGunCmd::AddGroupParticipants {
                                conv_id,
                                recipients,
                                rsp: tx,
                            }))
                        {
                            log::error!("failed to send warp command: {}", e);
                            continue;
                        }
                        let res = rx.await.expect("command canceled");
                        if let Err(e) = res {
                            log::error!("failed to add new recipients to a group: {}", e);
                        }
                    }
                    ChanCmd::RemoveParticipants => {
                        let recipients: Vec<DID> =
                            selected_friends.current().iter().cloned().collect();
                        let (tx, rx) = oneshot::channel();
                        if let Err(e) =
                            warp_cmd_tx.send(WarpCmd::RayGun(RayGunCmd::RemoveGroupParticipants {
                                conv_id,
                                recipients,
                                rsp: tx,
                            }))
                        {
                            log::error!("failed to send warp command: {}", e);
                            continue;
                        }
                        let res = rx.await.expect("command canceled");
                        if let Err(e) = res {
                            log::error!("failed to remove recipients from a group: {}", e);
                        }
                    }
                }
            }
        }
    });
    cx.render(rsx!(
        // div {
        //     class: "friend-container",
        //     aria_label: "Friend Container",
        //     UserImage {
        //         platform: cx.props.friend.platform().into(),
        //         status: cx.props.friend.identity_status().into(),
        //         image: cx.props.friend.profile_picture()
        //     },
        //     div {
        //         class: "flex-1",
        //         p {
        //             aria_label: "friend-username",
        //             cx.props.friend.username(),
        //         },
        //     },
        //     if cx.props.is_creator {
        //         rsx!(
        //             div {
        //                 class: "group-creator-container",
        //                 IconElement {
        //                     icon: Icon::Satellite
        //                 }
        //                 span {
        //                     class: "group-creator-text",
        //                     get_local_text("messages.group-creator-label")
        //                 }
        //             }
        //         )

        //     }
        // }
        div {
            class: "friend-container",
            aria_label: "Friend Container",
            UserImage {
                platform: _friend.platform().into(),
                status: _friend.identity_status().into(),
                image: _friend.profile_picture()
            },
            div {
                class: "flex-1",
                p {
                    aria_label: "friend-username",
                    _friend.username(),
                },
            },
            Button {
                aria_label: get_local_text("uplink.remove"),
                icon: if *edit_group_action.current() == EditGroupAction::Add {
                    Icon::UserPlus
                } else {
                    Icon::UserMinus
                },
                text: if *edit_group_action.current() == EditGroupAction::Add {
                    get_local_text("uplink.add")
                } else {
                    get_local_text("uplink.remove")
                },
                onpress: move |_| {
                    let mut friends = selected_friends.get().clone();
                    friends.clear();
                    selected_friends.set(vec![_friend.did_key()].into_iter().collect());
                    if *edit_group_action.current() == EditGroupAction::Add {
                        ch.send(ChanCmd::AddParticipants);
                    } else {
                        ch.send(ChanCmd::RemoveParticipants);
                    }
                }
            }
        }
    ))
}
