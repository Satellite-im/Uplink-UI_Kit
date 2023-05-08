use std::thread;

use crate::language::get_local_text;

use super::sounds::{Play, Sounds};
use notify_rust::Notification;
use std::sync::Arc;
use uuid::Uuid;
use warp::logging::tracing::log;
use windows::{runtime::IInspectable, UI::Notifications::ToastActivatedEventArgs};

use once_cell::sync::Lazy;
use tokio::sync::{
    mpsc::{UnboundedReceiver, UnboundedSender},
    Mutex,
};

#[cfg(target_os = "windows")]
pub const POWERSHELL_APP_ID: &'static str = "{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\
\\WindowsPowerShell\\v1.0\\powershell.exe";

#[derive(Debug, Clone)]
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
    action: NotificationAction,
) {
    let summary = format!("Uplink - {title}");
    thread::spawn(move || {
        show_with_action(
            Notification::new()
                .summary(summary.as_ref())
                .body(&content)
                .timeout(timeout)
                .action(&action_id, &get_local_text(&action_id))
                .finalize(),
            action_id,
            action,
        );
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

// We need to handle them all differently as there isnt a single lib that covers it for all
fn show_with_action(notification: Notification, action_id: String, action: NotificationAction) {
    #[cfg(target_os = "windows")]
    {
        // Notify-rust does not support windows actions so we use the underlying system directly
        // See https://gist.github.com/allenbenz/a0fb225aef43df4b1be1c005fb4c2811
        let duration = match notification.timeout {
            notify_rust::Timeout::Default => "duration=\"short\"",
            notify_rust::Timeout::Never => "duration=\"long\"",
            notify_rust::Timeout::Milliseconds(t) => {
                if t >= 25000 {
                    "duration=\"long\""
                } else {
                    "duration=\"short\""
                }
            }
        };
        //TODO set proper app id
        let app_id = POWERSHELL_APP_ID.to_string();
        let toast_xml = windows::Data::Xml::Dom::XmlDocument::new().unwrap();
        let template_binding = "ToastGeneric";
        /*if is_newer_than_windows81() {
            "ToastGeneric"
        } else {
            // Need to do this or an empty placeholder will be shown if no image is set
            if notification.icon.is_empty() {
                "ToastText04"
            } else {
                "ToastImageAndText04"
            }
        };*/
        let actions = format!(
            r#"<action content="{}" arguments="{}"/>"#,
            &get_local_text(&action_id),
            &action_id
        );

        toast_xml
            .LoadXml(&windows::runtime::HSTRING::from(format!(
                "<toast {} {}>
                    <visual>
                        <binding template=\"{}\">
                        {}
                        {}{}{}
                        </binding>
                    </visual>
                    {}
                    <actions>
                        {}
                    </actions>
                </toast>",
                duration,
                String::new(), //Scenario
                template_binding,
                &notification.icon,
                &notification.summary,
                notification.subtitle.as_ref().map_or("", AsRef::as_ref),
                &notification.body,
                r#"<audio silent='true'/>"#, //Already handled in uplink
                actions
            )))
            .unwrap();

        // Create the toast
        let toast_notification =
            windows::UI::Notifications::ToastNotification::CreateToastNotification(&toast_xml)
                .unwrap();

        toast_notification
            .Activated(windows::Foundation::TypedEventHandler::new(
                move |_sender, result: &Option<IInspectable>| {
                    let event: Option<windows::runtime::Result<ToastActivatedEventArgs>> =
                        result.as_ref().map(windows::runtime::Interface::cast);
                    let arguments = event
                        .and_then(|val| val.ok())
                        .and_then(|args| args.Arguments().ok());
                    if let Some(val) = arguments {
                        if val.to_string_lossy().eq(&action_id) {
                            log::trace!("windows action activated {:?}", val);
                            let tx = NOTIFICATION_LISTENER.tx.clone();
                            if let Err(e) = tx.send(action.to_owned()) {
                                log::error!("failed to send command to initialize warp {}", e);
                            }
                        }
                    };
                    Ok(())
                },
            ))
            .unwrap();

        let toast_notifier =
            windows::UI::Notifications::ToastNotificationManager::CreateToastNotifierWithId(
                &windows::runtime::HSTRING::from(&app_id),
            )
            .unwrap();
        let _n = toast_notifier.Show(&toast_notification);
    }

    #[cfg(target_os = "macos")]
    {
        // Notify-rust does not support macos actions but the underlying mac_notification library does
        let action_name = &get_local_text(&action_id);
        let response = mac_notification_sys::Notification::default()
            .title(notification.summary.as_str())
            .message(&notification.body)
            .maybe_subtitle(notification.subtitle.as_deref())
            .main_button(mac_notification_sys::MainButton::SingleAction(&action_name))
            .send()
            .unwrap();
        if let mac_notification_sys::NotificationResponse::ActionButton(id) = response {
            if action_name.eq(&id) {
                let tx = NOTIFICATION_LISTENER.tx.clone();
                if let Err(e) = tx.send(action) {
                    log::error!("failed to send command to initialize warp {}", e);
                }
            };
        }
    }

    #[cfg(target_os = "linux")]
    {
        let handle = notification.show().unwrap();
        handle.wait_for_action(|id| {
            if action_id.eq(id) {
                let tx = NOTIFICATION_LISTENER.tx.clone();
                if let Err(e) = tx.send(action) {
                    log::error!("failed to send command to initialize warp {}", e);
                }
            };
        });
    }
}
