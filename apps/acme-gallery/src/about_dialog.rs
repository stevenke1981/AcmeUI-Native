//! Inline card visibility belongs to this demo; this is not a modal host.
use super::*;
use acme_ui::overlay::about_dialog;

impl Gallery {
    pub(super) fn about_dialog_examples(&self) -> WidgetNode<GalleryMessage> {
        let content: WidgetNode<GalleryMessage> = if self.about_visible {
            about_dialog("gallery_about", "AcmeUI Native")
                .version("0.2")
                .description("About card · 關於程式")
                .open(true)
                .on_close(GalleryMessage::AboutClose)
                .into()
        } else {
            button("gallery_about_show", "Show about card · 再次顯示")
                .on_click(GalleryMessage::AboutShow)
        };
        column()
            .key("gallery_about_demo")
            .gap(8.0)
            .child(content)
            .child(label(format!("About Close count: {}", self.about_closes)))
            .build()
    }
}
