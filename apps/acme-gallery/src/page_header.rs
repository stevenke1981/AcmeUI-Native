//! Optional subtitles must not replace PageHeader's keyed actions.
use super::*;
use acme_ui::layout::{HeaderAction, page_header};

impl Gallery {
    pub(super) fn page_header_examples(&self) -> WidgetNode<GalleryMessage> {
        let mut header = page_header("gallery_page_header_control", "Draft")
            .action(HeaderAction::primary(
                "gallery_header_save",
                "Save",
                GalleryMessage::HeaderSave,
            ))
            .action(HeaderAction::secondary(
                "gallery_header_reset",
                "Reset",
                GalleryMessage::HeaderReset,
            ));
        if self.header_description {
            header = header.description("Optional document description");
        }
        column()
            .key("gallery_page_header")
            .gap(8.0)
            .child(header)
            .child(
                button(
                    "gallery_header_description",
                    if self.header_description {
                        "Hide description"
                    } else {
                        "Show description"
                    },
                )
                .on_click(GalleryMessage::HeaderToggleDescription),
            )
            .child(label(format!("PageHeader saves: {}", self.header_saves)))
            .build()
    }
}
