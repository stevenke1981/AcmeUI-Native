//! Controlled Collapsible preview; the host owns open/disabled state.
use super::*;
use acme_ui::foundations::collapsible;

pub(super) fn row_action(widget: &WidgetNode<GalleryMessage>) -> Option<GalleryMessage> {
    action_examples::row_action(widget).or_else(|| {
        if let WidgetNode::Row(row) = widget {
            row.message
                .filter(|m| matches!(m, GalleryMessage::CollapsibleToggle))
        } else {
            None
        }
    })
}

impl Gallery {
    pub(super) fn collapsible_examples(&self) -> WidgetNode<GalleryMessage> {
        column()
            .key("gallery_collapsible_demo")
            .gap(8.0)
            .child(label("Click the details header · 點選標題展開內容"))
            .child(
                collapsible("gallery_collapsible")
                    .title("Details · 詳細資料")
                    .open(self.collapsible_open)
                    .disabled(self.collapsible_disabled)
                    .on_toggle(GalleryMessage::CollapsibleToggle)
                    .child(label("Content is visible · 內容已展開")),
            )
            .child(
                button(
                    "gallery_collapsible_disable",
                    if self.collapsible_disabled {
                        "Enable header · 啟用標題"
                    } else {
                        "Disable header · 停用標題"
                    },
                )
                .on_click(GalleryMessage::CollapsibleDisable),
            )
            .child(label(format!(
                "Header: {} · Toggle count: {}",
                if self.collapsible_disabled {
                    "disabled"
                } else {
                    "enabled"
                },
                self.collapsible_toggles
            )))
            .build()
    }
}
