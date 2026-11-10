//! Gallery owns notification visibility and deadlines; the component is stateless.
use super::*;
use std::time::{Duration, Instant};

pub(super) struct NotificationDemo {
    pub(super) visible: bool,
    pub(super) deadline: Option<Instant>,
    pub(super) status: &'static str,
}
impl NotificationDemo {
    pub(super) fn new() -> Self {
        Self {
            visible: true,
            deadline: None,
            status: "shown until dismissed",
        }
    }
    pub(super) fn show_timed(&mut self, now: Instant) {
        self.visible = true;
        self.deadline = Some(now + Duration::from_secs(5));
        self.status = "shown for 5 seconds";
    }
    pub(super) fn dismiss(&mut self) {
        self.visible = false;
        self.deadline = None;
        self.status = "dismissed";
    }
    pub(super) fn expire(&mut self, now: Instant) -> bool {
        if self.deadline.is_none_or(|deadline| now < deadline) {
            return false;
        }
        self.visible = false;
        self.deadline = None;
        self.status = "automatically dismissed";
        true
    }
    pub(super) fn poll_interval(&self) -> Option<Duration> {
        self.deadline.map(|_| Duration::from_millis(100))
    }
}

impl Gallery {
    pub(super) fn notification_examples(&self) -> WidgetNode<GalleryMessage> {
        let mut examples = column().key("gallery_notifications").gap(8.0);
        if self.notification.visible {
            examples = examples.child(
                mobile_notification("Message received · 收到新訊息")
                    .key("gallery_notification_active")
                    .body("Your changes are ready to review. 你的變更已可供檢閱。")
                    .action("Open", GalleryMessage::MobileNotificationAction)
                    .on_dismiss(GalleryMessage::MobileNotificationDismiss),
            );
        }
        examples
            .child(row().gap(8.0)
                .child(button("gallery_notification_show", "Show for 5 seconds")
                    .on_click(GalleryMessage::MobileNotificationShow))
                .child(label(format!("Notification: {}", self.notification.status))))
            .child(label("Show again to restart the timer; Dismiss closes immediately."))
            .child(mobile_notification("Title only · 僅標題").key("gallery_notification_title"))
            .child(column().width(320.0).child(
                mobile_notification("Long message · 長訊息").key("gallery_notification_long")
                    .body("Please save your work before continuing. 請先儲存工作，再繼續下一步，避免遺失尚未儲存的內容。")))
            .build()
    }
}
