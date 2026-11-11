//! A controlled BottomNavigation: selection belongs to the Gallery host.
use super::*;
use acme_ui::layout::{BottomNavigationAction, bottom_navigation};

impl Gallery {
    pub(super) fn bottom_navigation_examples(&self) -> WidgetNode<GalleryMessage> {
        column()
            .key("gallery_bottom_navigation")
            .gap(8.0)
            .child(label(
                "Choose a destination; the host updates the selected marker.",
            ))
            .child(
                bottom_navigation()
                    .key("gallery_bottom_navigation_control")
                    .action(BottomNavigationAction::new("Home").icon(acme_ui::IconName::User))
                    .action(BottomNavigationAction::new("Search").icon(acme_ui::IconName::Search))
                    .action(BottomNavigationAction::new("Saved").icon(acme_ui::IconName::Star))
                    .selected_index(self.bottom_selected)
                    .on_select(GalleryMessage::BottomNavigationSelect),
            )
            .child(label(format!(
                "BottomNavigation selected: {}",
                ["Home", "Search", "Saved"][self.bottom_selected]
            )))
            .build()
    }
}
