//! Private host routing for the Mobile action examples.
use super::*;
pub(super) fn row_action(widget: &WidgetNode<GalleryMessage>) -> Option<GalleryMessage> {
    list_examples::row_action(widget).or_else(|| {
        if let WidgetNode::Row(row) = widget {
            row.message
                .filter(|m| matches!(m, GalleryMessage::MobileActionPress(_)))
        } else {
            None
        }
    })
}
impl Gallery {
    pub(super) fn action_examples(&self) -> WidgetNode<GalleryMessage> {
        column()
            .key("gallery_actions")
            .gap(8.0)
            .child(
                mobile_action("Share · 分享")
                    .key("gallery_action_share")
                    .icon(acme_ui::IconName::Star)
                    .on_press(GalleryMessage::MobileActionPress(false)),
            )
            .child(
                mobile_action("Delete · 刪除")
                    .key("gallery_action_delete")
                    .icon(acme_ui::IconName::Close)
                    .destructive(true)
                    .on_press(GalleryMessage::MobileActionPress(true)),
            )
            .child(
                mobile_action("Disabled delete · 停用刪除")
                    .key("gallery_action_disabled")
                    .destructive(true)
                    .disabled(true)
                    .on_press(GalleryMessage::MobileActionPress(true)),
            )
            .child(mobile_action("No handler · 未設定動作").key("gallery_action_passive"))
            .child(
                column().width(260.0).child(
                    mobile_action("Long action description · 請先閱讀內容，再執行這個操作")
                        .key("gallery_action_long")
                        .icon(acme_ui::IconName::Info),
                ),
            )
            .child(label(format!("Action: {}", self.action_status)))
            .build()
    }
}
