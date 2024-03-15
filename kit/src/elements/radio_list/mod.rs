use crate::elements::button::Button;
use crate::elements::Appearance;
use common::icons::outline::Shape as Icon;
use dioxus::prelude::*;

#[derive(Props, Clone)]
pub struct Props {
    initial_value: String,
    values: Vec<String>,
    onchange: EventHandler<String>,
}

#[allow(non_snake_case)]
pub fn RadioList(props: Props) -> Element {
    let internal_state = use_signal(|| props.initial_value.clone());

    rsx!(
        div {
            class: "radio-list",
            for option in &props.values {
                Button {
                    icon: if internal_state.get() == option { Icon::RadioSelected } else { Icon::Radio },
                    appearance: if internal_state.get() == option { Appearance::Primary } else { Appearance::Secondary },
                    text: option.clone(),
                    aria_label: format!("radio-option-{}", option),
                    onpress: move |_| {
                        internal_state.set(option.clone());
                        props.onchange.call(option.clone());
                    },
                }
            }
        }
    )
}
