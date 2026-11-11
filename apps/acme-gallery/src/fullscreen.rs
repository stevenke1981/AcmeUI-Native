//! Inline preview demo; this host controls visibility, without a modal overlay.
use super::*;
use acme_ui::overlay::fullscreen;

impl Gallery {
    pub(super) fn fullscreen_examples(&self) -> WidgetNode<GalleryMessage> {
        let content: WidgetNode<GalleryMessage> = if self.fullscreen_visible {
            fullscreen("gallery_fullscreen")
                .on_close(GalleryMessage::FullscreenClose)
                .child(label("Preview content · 預覽內容"))
                .build()
        } else {
            button("gallery_fullscreen_show", "Reopen preview · 重新開啟")
                .on_click(GalleryMessage::FullscreenShow)
        };
        column()
            .key("gallery_fullscreen_demo")
            .gap(8.0)
            .child(label("Close and reopen the inline preview."))
            .child(content)
            .child(label(format!(
                "Preview Close count: {}",
                self.fullscreen_closes
            )))
            .build()
    }
}
