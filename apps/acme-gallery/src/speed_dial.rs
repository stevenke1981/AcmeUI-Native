//! A controlled SpeedDial; selecting an action leaves closing to the host.
use super::*;
use acme_ui::overlay::{SpeedDialAction, speed_dial};

#[derive(Default)]
pub(super) struct SpeedDialDemo {
    pub open: bool,
    pub last_action: Option<usize>,
}

impl Gallery {
    pub(super) fn speed_dial_examples(&self) -> WidgetNode<GalleryMessage> {
        column()
            .key("gallery_speed_dial")
            .gap(8.0)
            .child(label(
                "Expand the dial, then choose an action. Close with the same toggle.",
            ))
            .child(
                speed_dial()
                    .key("gallery_speed_dial_control")
                    .open(self.speed_dial.open)
                    .action(SpeedDialAction::new("Edit · 編輯").icon(acme_ui::IconName::Plus))
                    .action(SpeedDialAction::new("Share · 分享"))
                    .on_toggle(GalleryMessage::SpeedDialToggle)
                    .on_action(GalleryMessage::SpeedDialAction),
            )
            .child(label(format!(
                "SpeedDial action: {}",
                match self.speed_dial.last_action {
                    Some(0) => "Edit · 編輯",
                    Some(_) => "Share · 分享",
                    None => "none",
                }
            )))
            .build()
    }
}
