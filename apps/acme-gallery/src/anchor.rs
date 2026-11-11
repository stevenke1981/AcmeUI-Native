//! Controlled navigation messages; this preview does not scroll to targets.
use super::*;
use acme_ui::layout::{AnchorItem, anchor};

pub(super) const HREFS: [&str; 3] = ["#intro", "#api", "#get"];

impl Gallery {
    pub(super) fn anchor_examples(&self) -> WidgetNode<GalleryMessage> {
        column()
            .key("gallery_anchor_demo")
            .gap(8.0)
            .child(label("Select a link · 選擇連結"))
            .child(
                anchor()
                    .key("gallery_anchor")
                    .item(AnchorItem::new(HREFS[0], "Introduction · 簡介"))
                    .item(AnchorItem::new(HREFS[1], "API").child(AnchorItem::new(HREFS[2], "GET")))
                    .active_href(HREFS[self.anchor_selected])
                    .on_navigate(|href| {
                        GalleryMessage::AnchorNavigate(
                            HREFS.iter().position(|&s| s == href).unwrap(),
                        )
                    }),
            )
            .child(label(format!(
                "Selected: {} · Navigation count: {}",
                HREFS[self.anchor_selected], self.anchor_navigations
            )))
            .build()
    }
}
