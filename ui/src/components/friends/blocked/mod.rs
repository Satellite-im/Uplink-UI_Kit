use crate::{
    components::friends::friend::Friend,
    logger,
    state::State,
    utils::convert_status,
    warp_runner::{commands::MultiPassCmd, WarpCmd},
    WARP_CMD_CH,
};
use dioxus::prelude::*;
use futures::{channel::oneshot, StreamExt};
use kit::{
    components::{
        context_menu::{ContextItem, ContextMenu},
        indicator::Platform,
        user_image::UserImage,
    },
    elements::label::Label,
    icons::Icon,
};
use shared::language::get_local_text;
use warp::{crypto::DID, error::Error, multipass::identity::Relationship};

#[allow(non_snake_case)]
pub fn BlockedUsers(cx: Scope) -> Element {
    let state = use_shared_state::<State>(cx).unwrap();
    let block_list = state.read().friends.blocked.clone();

    let ch = use_coroutine(cx, |mut rx: UnboundedReceiver<DID>| {
        //to_owned![];
        async move {
            let warp_cmd_tx = WARP_CMD_CH.tx.clone();
            while let Some(did) = rx.next().await {
                let (tx, rx) = oneshot::channel::<Result<(), warp::error::Error>>();
                warp_cmd_tx
                    .send(WarpCmd::MultiPass(MultiPassCmd::Unblock { did, rsp: tx }))
                    .expect("failed to send cmd");

                let rsp = rx.await.expect("command canceled");
                if let Err(e) = rsp {
                    match e {
                        Error::PublicKeyIsntBlocked => {}
                        _ => {
                            logger::error(&format!("failed to unblock user: {}", e));
                        }
                    }
                }
            }
        }
    });

    cx.render(rsx! (
        div {
            class: "friends-list",
            aria_label: "Blocked List",
            Label {
                text: get_local_text("friends.blocked"),
            },
            block_list.into_iter().map(|blocked_user| {
                let did = blocked_user.did_key();
                let did_suffix: String = did.to_string().chars().rev().take(6).collect();
                let unblock_user = blocked_user.clone();
                let unblock_user_clone = unblock_user.clone();
                let platform = match blocked_user.platform() {
                    warp::multipass::identity::Platform::Desktop => Platform::Desktop,
                    warp::multipass::identity::Platform::Mobile => Platform::Mobile,
                    _ => Platform::Headless //TODO: Unknown
                };
                let mut relationship = Relationship::default();
                relationship.set_blocked(true);
                rsx!(
                    ContextMenu {
                        id: format!("{}-friend-listing", did),
                        key: "{did}-friend-listing",
                        items: cx.render(rsx!(
                            ContextItem {
                                danger: true,
                                icon: Icon::XMark,
                                text: get_local_text("friends.unblock"),
                                onpress: move |_| {
                                    ch.send(unblock_user.clone().did_key());
                                }
                            },
                        )),
                        Friend {
                            username: blocked_user.username(),
                            suffix: did_suffix,
                            status_message: blocked_user.status_message().unwrap_or_default(),
                            relationship: relationship,
                            user_image: cx.render(rsx! (
                                UserImage {
                                    platform: platform,
                                    status: convert_status(&blocked_user.identity_status()),
                                    image: blocked_user.graphics().profile_picture()
                                }
                            )),
                            onremove: move |_| {
                                ch.send(unblock_user_clone.clone().did_key());
                            }
                        }
                    }
                )
            })
        }
    ))
}
