use dioxus::prelude::*;
use kit::{
    elements::{button::Button, Appearance},
    icons::Icon,
};
use shared::language::get_local_text;

use crate::{components::settings::SettingSection};

#[allow(non_snake_case)]
pub fn PrivacySettings(cx: Scope) -> Element {
    cx.render(rsx!(
        div {
            id: "settings-privacy",
            SettingSection {
                section_label: get_local_text("settings-privacy.backup-recovery-phrase"),
                section_description: get_local_text("settings-privacy.backup-phrase-description"),
                Button {
                    text: get_local_text("settings-privacy.backup-phrase"),
                    appearance: Appearance::Secondary,
                    icon: Icon::DocumentText,
                }
            },
        }
    ))
}
