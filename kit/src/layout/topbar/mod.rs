use common::icons;
use dioxus::prelude::*;
use warp::logging::tracing::log;

use crate::elements::{button::Button, Appearance};

#[derive(Props)]
pub struct Props<'a> {
    #[props(optional)]
    with_back_button: Option<bool>,
    #[props(optional)]
    with_currently_back: Option<bool>,
    #[props(optional)]
    onback: Option<EventHandler<'a>>,
    #[props(optional)]
    controls: Option<Element<'a>>,
    #[props(optional)]
    children: Option<Element<'a>>,
}

/// If enabled, it will render the bool
pub fn show_back_button(cx: &Scope<Props>) -> bool {
    cx.props.with_back_button.unwrap_or(false)
}

/// Emit the back button event
pub fn emit(cx: &Scope<Props>) {
    match &cx.props.onback {
        Some(f) => f.call(()),
        None => {}
    }
}

#[allow(non_snake_case)]
pub fn Topbar<'a>(cx: Scope<'a, Props<'a>>) -> Element<'a> {
    log::trace!("rendering topbar");
    let currently_back = cx.props.with_currently_back.unwrap_or(false);

    cx.render(rsx!(
        div {
            class: "topbar",
            aria_label: "Topbar",
            (show_back_button(&cx)).then(|| rsx!(
                Button {
                    aria_label: "hamburger-button".into(),
                    icon: if currently_back { icons::outline::Shape::ChevronLeft } else { icons::outline::Shape::SidebarArrowLeft },
                    onpress: move |_| emit(&cx),
                    appearance: Appearance::Secondary
                }
            )),
            div {
                class: "children",
                cx.props.children.as_ref()
            },
            div {
                class: "controls",
                cx.props.controls.as_ref()
            }
        }
    ))
}
