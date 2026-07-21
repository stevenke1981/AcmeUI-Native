//! Overlay category page builders.

use acme_widgets::WidgetNode;

use crate::types::*;

impl crate::Gallery {
    pub fn overlay_page(&self) -> WidgetNode<GalleryMessage> {
        let name = CATEGORIES[3].pages[self.selected_page.min(3)];
        self.component_page(name)
    }
}
