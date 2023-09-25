// although it's annoying to not immediately see the contents of the script when you ctrl+click the variable,
// it is easier to read and write the script when it's in a separate .js file. Having a folder of .js scripts
// also makes it easier to see what scripts are available.

pub const SETUP_CONTEXT_PARENT: &str = include_str!("./setup_context_parent.js");
pub const SCROLL_TO: &str = include_str!("./scroll_to.js");
pub const SCROLL_UNREAD: &str = include_str!("./scroll_unread.js");
pub const SCROLL_BOTTOM: &str = include_str!("./scroll_bottom.js");
pub const READ_SCROLL: &str = include_str!("./read_scroll.js");
pub const SHOW_CONTEXT: &str = include_str!("./show_context.js");
pub const SCROLL_TO_MESSAGE: &str = include_str!("./scroll_to_message.js");
pub const SCROLL_TO_TOP: &str = include_str!("./scroll_to_top.js");
pub const SCROLL_TO_BOTTOM: &str = include_str!("./scroll_to_bottom.js");
pub const SCROLL_TO_END: &str = include_str!("./scroll_to_end.js");
pub const OBSERVER_SCRIPT: &str = include_str!("./observer_script.js");
