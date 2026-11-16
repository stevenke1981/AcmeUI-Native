//! Modal demo for the Gallery overlay page.
use super::*;
use acme_ui::overlay::modal;

impl Gallery {
    pub(super) fn modal_examples(&self) -> WidgetNode<GalleryMessage> {
        let content: WidgetNode<GalleryMessage> = if self.modal_visible {
            modal("gallery_modal", "Modal Demo")
                .width(480.0)
                .closable(true)
                .on_close(GalleryMessage::ModalClose)
                .child(label("This is a modal dialog with a close button."))
                .build()
        } else {
            button("gallery_modal_show", "Show modal · 顯示對話框")
                .on_click(GalleryMessage::ModalShow)
        };
        column()
            .key("gallery_modal_demo")
            .gap(8.0)
            .child(content)
            .child(label(format!("Modal Close count: {}", self.modal_closes)))
            .build()
    }
}
