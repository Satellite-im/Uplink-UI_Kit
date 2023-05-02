use std::thread;

use super::sounds::{Play, Sounds};
use notify_rust::Notification;
use std::sync::Arc;
use uuid::Uuid;
use warp::logging::tracing::log;

use once_cell::sync::Lazy;
use tokio::sync::{
    mpsc::{UnboundedReceiver, UnboundedSender},
    Mutex,
};

#[derive(Debug)]
pub enum NotificationAction {
    DisplayChat(Uuid),
    FriendListPending,
    Dummy,
}

pub struct NotificationChannel {
    pub tx: UnboundedSender<NotificationAction>,
    pub rx: Arc<Mutex<UnboundedReceiver<NotificationAction>>>,
}

pub static NOTIFICATION_LISTENER: Lazy<NotificationChannel> = Lazy::new(|| {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    NotificationChannel {
        tx,
        rx: Arc::new(Mutex::new(rx)),
    }
});

// Implementation to create and push new notifications
#[allow(non_snake_case)]
pub fn push_notification(
    title: String,
    content: String,
    notification_sound: Option<Sounds>,
    timeout: notify_rust::Timeout,
) {
    let summary = format!("Uplink - {title}");
    let _n = Notification::new()
        .summary(summary.as_ref())
        .body(&content)
        .timeout(timeout)
        .show();

    if let Some(sound) = notification_sound {
        Play(sound);
    }
}

#[allow(non_snake_case)]
pub fn push_notification_actionable(
    title: String,
    content: String,
    notification_sound: Option<Sounds>,
    timeout: notify_rust::Timeout,
    action_id: String,
    action_name: String,
    action: NotificationAction,
) {
    let summary = format!("Uplink - {title}");
    thread::spawn(move || {
        let handle = Notification::new()
            .summary(summary.as_ref())
            .body(&content)
            .timeout(timeout)
            .action(&action_id, &action_name)
            .show()
            .unwrap();
        handle.wait_for_action(|id| {
            if action_id.eq(id) {
                let tx = NOTIFICATION_LISTENER.tx.clone();
                if let Err(e) = tx.send(action) {
                    log::error!("failed to send command to initialize warp {}", e);
                }
            };
        });
    });

    if let Some(sound) = notification_sound {
        Play(sound);
    }
}

pub fn set_badge(count: u32) -> Result<(), String> {
    #[cfg(not(target_os = "macos"))]
    let _ = count;
    #[cfg(target_os = "macos")]
    unsafe {
        use cocoa::{appkit::NSApp, base::nil, foundation::NSString};
        use objc::{msg_send, sel, sel_impl};

        let label = if count == 0 {
            nil
        } else {
            NSString::alloc(nil).init_str(&format!("{}", count))
        };
        let dock_tile: cocoa::base::id = msg_send![NSApp(), dockTile];
        let _: cocoa::base::id = msg_send![dock_tile, setBadgeLabel: label];
    }
    Ok(())
}
