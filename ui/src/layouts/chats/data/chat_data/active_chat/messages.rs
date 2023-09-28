use chrono::{DateTime, Utc};
use common::warp_runner::ui_adapter;
use std::collections::{HashMap, VecDeque};
use uuid::Uuid;

use crate::layouts::chats::data::{MsgView, PartialMessage};

#[derive(Debug, Default, Clone)]
pub struct Messages {
    pub messages: VecDeque<ui_adapter::Message>,
    pub displayed_messages: MsgView,
    // used for displayed_messages
    pub message_times: HashMap<Uuid, DateTime<Utc>>,
}

impl Messages {
    pub fn new(mut m: VecDeque<ui_adapter::Message>) -> Self {
        let mut message_times = HashMap::new();
        let mut messages = VecDeque::new();
        for msg in m.drain(..) {
            message_times.insert(msg.inner.id(), msg.inner.date());
            messages.push_back(msg);
        }

        Self {
            messages,
            displayed_messages: MsgView::default(),
            message_times,
        }
    }

    pub fn append_messages(&mut self, mut m: Vec<ui_adapter::Message>) {
        for msg in m.drain(..) {
            self.message_times.insert(msg.inner.id(), msg.inner.date());
            self.messages.push_back(msg);
        }
    }

    pub fn prepend_messages(&mut self, mut m: Vec<ui_adapter::Message>) {
        for msg in m.drain(..).rev() {
            self.message_times.insert(msg.inner.id(), msg.inner.date());
            self.messages.push_front(msg);
        }
    }

    pub fn add_message_to_view(&mut self, message_id: Uuid) {
        let date = match self.message_times.get(&message_id).cloned() {
            Some(time) => time,
            None => return,
        };
        self.displayed_messages
            .insert(PartialMessage { message_id, date });
    }

    pub fn remove_message_from_view(&mut self, message_id: Uuid) {
        self.displayed_messages.remove(message_id);
    }
}
