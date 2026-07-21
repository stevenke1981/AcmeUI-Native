//! Stress tests category page builders.

use acme_widgets::WidgetNode;

use crate::types::*;

impl crate::Gallery {
    pub fn stress_tests_page(&self) -> WidgetNode<GalleryMessage> {
        let name = CATEGORIES[7].pages[self.selected_page.min(3)];
        self.component_page(name)
    }
}
