use dioxus_core::ScopeState;
use dioxus_desktop::use_window;

pub fn set_app_visible(cx: &ScopeState) {
    let window = use_window(cx);
    if !window.is_visible() {
        window.set_visible(true);
        window.set_focus();
    }
}
