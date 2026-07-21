//! Inputs category page builders.

use acme_widgets::{WidgetNode, column, label};

use crate::helpers::standard_component_sections;
use crate::types::*;

impl crate::Gallery {
    pub fn inputs_page(&self) -> WidgetNode<GalleryMessage> {
        if self.selected_page == 1 {
            return self.textinput_page();
        }
        let name = CATEGORIES[1].pages[self.selected_page.min(5)];
        self.component_page(name)
    }

    pub fn textinput_page(&self) -> WidgetNode<GalleryMessage> {
        let mut secs = standard_component_sections();
        secs.push((
            "States",
            column()
                .gap(8.0)
                .child(label(
                    "Click the input below to focus, then type or use IME:",
                ))
                .child(label(TEXT_INPUT_MARKER))
                .child(label(""))
                .build(),
        ));
        self.build_component_page("TextInput", secs)
    }
}
