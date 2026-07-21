//! Component page template builder.

use acme_widgets::{
    WidgetNode, column, label, label_with_size, separator,
};

use crate::helpers::{spacing, standard_component_sections};
use crate::types::*;

impl crate::Gallery {
    /// Page dispatcher — routes to the correct category builder.
    pub fn render_page(&self) -> WidgetNode<GalleryMessage> {
        match self.selected_category {
            0 => self.foundations_page(),
            1 => self.inputs_page(),
            2 => self.navigation_page(),
            3 => self.overlay_page(),
            4 => self.data_page(),
            5 => self.patterns_page(),
            6 => self.accessibility_page(),
            7 => self.stress_tests_page(),
            _ => label("Unknown category"),
        }
    }

    /// Build a quick component page with standard sections.
    pub fn component_page(&self, title: &str) -> WidgetNode<GalleryMessage> {
        let secs = standard_component_sections();
        self.build_component_page(title, secs)
    }

    pub fn build_component_page(
        &self,
        title: &str,
        sections: Vec<(&'static str, WidgetNode<GalleryMessage>)>,
    ) -> WidgetNode<GalleryMessage> {
        let mut page = column::<GalleryMessage>()
            .gap(spacing(self.density, 28.0))
            .padding(spacing(self.density, 24.0));
        page = page.child(label_with_size(title, 24.0));
        page = page.child(separator());
        for (section_title, content) in sections {
            page = page.child(self.page_section(section_title, content));
        }
        page.build()
    }

    pub fn page_section(
        &self,
        title: &str,
        content: WidgetNode<GalleryMessage>,
    ) -> WidgetNode<GalleryMessage> {
        column()
            .gap(spacing(self.density, 10.0))
            .child(label_with_size(title, 16.0))
            .child(separator())
            .child(content)
            .build()
    }
}
