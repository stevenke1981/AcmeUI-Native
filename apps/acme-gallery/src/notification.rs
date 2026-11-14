//! Host-controlled desktop notification preview.
use super::*;
use acme_ui::overlay::notification;

impl Gallery {
    pub(super) fn desktop_notification_examples(&self) -> WidgetNode<GalleryMessage> {
        let content = if self.notification_visible {
            notification("Saved · 已儲存")
                .key("gallery_notification")
                .description("Undo the last change or close this notification.")
                .action("Undo", GalleryMessage::NotificationAction)
                .on_close(GalleryMessage::NotificationClose)
                .into()
        } else {
            button("gallery_notification_show", "Show notification")
                .on_click(GalleryMessage::NotificationShow)
        };
        column()
            .key("gallery_notification_demo")
            .gap(8.0)
            .child(content)
            .child(label(format!(
                "Actions: {} · Closes: {}",
                self.notification_actions, self.notification_closes
            )))
            .build()
    }
}
