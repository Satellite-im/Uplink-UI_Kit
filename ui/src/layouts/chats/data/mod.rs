mod active_chat;
mod chat_behavior;
mod chat_data;
mod js_msg;
mod message_view;
mod misc;
mod msg_group;
mod msg_range;
mod partial_message;
mod scroll_behavior;
mod view_init;

pub use active_chat::*;
pub use chat_behavior::*;
pub use chat_data::*;
pub use js_msg::*;
pub use message_view::*;
pub use misc::*;
pub use msg_group::*;
pub use msg_range::*;
pub use partial_message::*;
pub use scroll_behavior::*;
pub use view_init::*;

pub const DEFAULT_MESSAGES_TO_TAKE: usize = 10;
