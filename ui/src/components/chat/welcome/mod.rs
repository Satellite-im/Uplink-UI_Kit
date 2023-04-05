use dioxus::prelude::*;

use crate::UPLINK_ROUTES;
use common::language::get_local_text;
use common::{icons::outline::Shape as Icon, STATIC_ARGS};
use dioxus_router::use_router;
use kit::elements::{button::Button, Appearance};

#[allow(non_snake_case)]
pub fn Welcome(cx: Scope) -> Element {
    let router = use_router(cx).clone();
    let cta_text = get_local_text("friends.cta-text");
    let image_path = STATIC_ARGS
        .extras_path
        .join("images")
        .join("mascot")
        .join("better_with_friends.webp")
        .to_str()
        .map(|x| x.to_string())
        .unwrap_or_default();
    cx.render(rsx! {
        div {
            id: "welcome",
            aria_label: "welcome-screen",
            img {
                class: "image",
                src:"{image_path}"
            },
            p {
                class: "muted",
                "{cta_text}"
            },
            Button {
                icon: Icon::Plus,
                aria_label: "add-friends-button".into(),
                text: get_local_text("friends.add"),
                appearance: Appearance::Secondary,
                onpress: move |_| {
                    router.replace_route(UPLINK_ROUTES.friends, None, None);
                }
            },
        }
    })
}
