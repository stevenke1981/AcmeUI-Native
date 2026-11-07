//! Inputs category page builders.

use acme_widgets::{WidgetNode, column, label, label_builder};

use crate::helpers::standard_component_sections;
use crate::render::{build_layout_context, build_theme};
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
        // The marker label stands in for the TextInput field. Its line box is the
        // control height, so layout reserves the whole field and the overlay rect
        // (taken from this node) is not one text line tall.
        let field_height = build_layout_context(&build_theme(self.dark), 1.0).control_height;
        secs.push((
            "States",
            column()
                .gap(8.0)
                .child(label(
                    "Click the input below to focus, then type or use IME:",
                ))
                .child(
                    label_builder(TEXT_INPUT_MARKER)
                        .line_height(field_height)
                        .build(),
                )
                .child(label("Undo: Ctrl+Z | Redo: Ctrl+Y / Ctrl+Shift+Z"))
                .build(),
        ));
        self.build_component_page("TextInput", secs)
    }
}
