//! Accessibility category page builders.

use crate::types::*;
use acme_style::{ColorToken, Styled};
use acme_widgets::{WidgetNode, button, column, label, row, stack};

impl crate::Gallery {
    pub fn accessibility_page(&self) -> WidgetNode<GalleryMessage> {
        if self.selected_page == 0 {
            let demo = column()
                .gap(16.0)
                .child(label("Tab / Shift+Tab moves focus; Enter / Space activates the exact focused control."))
                .child(
                    row()
                        .key("focus_action_row")
                        .padding(16.0)
                        .gap(12.0)
                        .bg(ColorToken::Muted)
                        .on_click(GalleryMessage::ToggleDensity)
                        .child(
                            column()
                                .child(label("Change density · 切換密度"))
                                .child(label("Row action; child button is independent")),
                        )
                        .child(button("focus_child", "Toggle focus rings").on_click(GalleryMessage::ToggleFocusRings)),
                )
                .child(
                    column()
                        .key("focus_action_column")
                        .padding(16.0)
                        .bg(ColorToken::Surface)
                        .on_click(GalleryMessage::ToggleTheme)
                        .child(label("Change theme · 切換主題")),
                )
                .child(
                    stack()
                        .key("focus_action_stack")
                        .size(360.0, 44.0)
                        .bg(ColorToken::Muted)
                        .on_click(GalleryMessage::FocusDemo)
                        .child(label("Stack action · 堆疊動作")),
                )
                .child(
                    row()
                        .key("focus_passive")
                        .padding(16.0)
                        .child(label("Read only · 唯讀：skipped by Tab")),
                )
                .child(button("focus_after", "Next control").on_click(GalleryMessage::DpiInfo))
                .child(label(format!("Current density: {}", self.density.label())))
                .build();
            return self.build_component_page(
                "Retained container focus",
                vec![("Keyboard and typed actions", demo)],
            );
        }
        let name = CATEGORIES[6].pages[self.selected_page.min(3)];
        self.component_page(name)
    }
}
