//! Private Gallery host for the mobile list's existing Row messages.
use super::*;
use acme_platform::PlatformKey;

/// Classify only this demo's row actions; other components own their routing.
pub(super) fn row_action(widget: &WidgetNode<GalleryMessage>) -> Option<GalleryMessage> {
    if let WidgetNode::Row(row) = widget {
        row.message
            .filter(|m| matches!(m, GalleryMessage::MobileListItemTap(_)))
    } else {
        None
    }
}

impl Gallery {
    pub(super) fn list_item_examples(&self) -> WidgetNode<GalleryMessage> {
        let item = |index| {
            let (key, title, detail) = if index == 0 {
                (
                    "gallery_list_wifi",
                    "Wi-Fi · 無線網路",
                    "Connected · 已連線",
                )
            } else {
                (
                    "gallery_list_notifications",
                    "Notifications · 通知",
                    "Messages enabled · 已啟用訊息",
                )
            };
            let mut node: WidgetNode<GalleryMessage> =
                mobile_list_item(title, self.list_details.then_some(detail))
                    .key(key)
                    .trailing_icon(acme_ui::IconName::ChevronRight)
                    .on_tap(GalleryMessage::MobileListItemTap(index))
                    .into();
            if self.list_focus == Some(index) {
                if let WidgetNode::Row(row) = &mut node {
                    row.style.background = Some(acme_widgets::style::ColorToken::Accent);
                }
            }
            node
        };
        let order = if self.list_reverse { [1, 0] } else { [0, 1] };
        column().key("gallery_list_items").gap(8.0)
            .child(item(order[0])).child(item(order[1]))
            .child(mobile_list_item("Information · 資訊", Some("Read-only row · 唯讀項目")).key("gallery_list_passive"))
            .child(column().width(320.0).child(mobile_list_item("A longer title · 較長的清單標題", Some("Long details wrap inside the row. 詳細資訊會在項目內換行，請閱讀後再繼續下一步。"))
                .key("gallery_list_long").trailing_icon(acme_ui::IconName::Info)))
            .child(row().gap(8.0)
                .child(button("gallery_list_reverse", "Reverse items").on_click(GalleryMessage::MobileListItemReverse))
                .child(button("gallery_list_details", "Toggle details").on_click(GalleryMessage::MobileListItemDetails)))
            .child(label(format!("Last opened: {}", match self.last_list_tap { Some(0) => "Wi-Fi", Some(1) => "Notifications", _ => "none" })))
            .child(acme_widgets::label_builder("Keyboard: F6 enters the list; Tab / Shift+Tab or arrows move; Enter / Space opens; Escape leaves.")
                .wrap(acme_widgets::LabelWrap::Character).build())
            .build()
    }
    fn reveal_list_focus(&mut self) {
        if let Some(index) = self.list_focus {
            if let Some(hit) = self
                .button_info
                .iter()
                .find(|h| h.message == GalleryMessage::MobileListItemTap(index))
            {
                self.scroll = (hit.rect[1] - TOOLBAR_HEIGHT - 8.0).clamp(0.0, self.max_scroll);
            }
        }
    }
    pub(super) fn list_item_event(&mut self, event: &PlatformEvent) -> Option<bool> {
        match event {
            PlatformEvent::PointerButton { pressed: true, .. } => {
                self.list_focus = self.hit().and_then(|i| match self.button_info[i].message {
                    GalleryMessage::MobileListItemTap(index) => Some(index),
                    _ => None,
                });
                None // Pointer activation remains in the shared Gallery handler.
            }
            PlatformEvent::FocusChanged { gained: false, .. } => {
                self.list_focus = None;
                None
            }
            PlatformEvent::Key {
                key: PlatformKey::Function(6),
                pressed: true,
                ..
            } => {
                // Do not steal a navigation key from an active composition.
                if !self.search.input.preedit.is_empty() {
                    return Some(false);
                }
                self.search.blur();
                self.list_focus = Some(if self.list_reverse { 1 } else { 0 });
                self.reveal_list_focus();
                Some(true)
            }
            PlatformEvent::Key {
                key,
                pressed: true,
                shift,
                ..
            } if self.list_focus.is_some() => {
                let order = if self.list_reverse { [1, 0] } else { [0, 1] };
                let current = self.list_focus.unwrap();
                let position = order.iter().position(|&i| i == current).unwrap();
                match key {
                    PlatformKey::Tab => {
                        self.list_focus = if *shift {
                            position.checked_sub(1).map(|i| order[i])
                        } else {
                            order.get(position + 1).copied()
                        };
                    }
                    PlatformKey::ArrowDown | PlatformKey::ArrowUp => {
                        self.list_focus = Some(order[(position + 1) % 2])
                    }
                    PlatformKey::Enter | PlatformKey::Space => {
                        let target = self
                            .button_info
                            .iter()
                            .position(|h| h.message == GalleryMessage::MobileListItemTap(current));
                        return Some(target.is_some_and(|i| self.activate(i)));
                    }
                    PlatformKey::Escape => self.list_focus = None,
                    _ => return None,
                }
                self.reveal_list_focus();
                Some(true)
            }
            _ => None,
        }
    }
}
