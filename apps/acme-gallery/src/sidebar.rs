//! Controlled Sidebar demo with a page-scoped keyboard host, plus the
//! single keyboard owner for the Layout page (NavigationMenu F6 / Sidebar F7).
use super::*;
use acme_platform::PlatformKey;
use acme_ui::layout::{SidebarBuilder, SidebarItem, SidebarState};
// Imported by path: `use super::*` also brings in `acme_widgets::sidebar`.
use acme_ui::layout::sidebar as sidebar_builder;

/// Demo items: (value, label, disabled). The index is the message payload.
const ITEMS: [(&str, &str, bool); 4] = [
    ("dashboard", "儀表板", false),
    ("projects", "專案", false),
    ("reports", "報表", true),
    ("settings", "設定", false),
];

/// Sentinel for a value that is not in `ITEMS`; the handler rejects it.
const UNKNOWN: u8 = u8::MAX;

#[derive(Default)]
pub(super) struct SidebarDemo {
    /// Index of the current (keyboard-focused) item.
    pub current: u8,
    /// Index of the last confirmed item.
    pub confirmed: Option<u8>,
    pub collapsed: bool,
    pub active: bool,
}

/// Fn pointers cannot capture, so the value maps back to its index.
pub(super) fn sidebar_message(value: &str) -> GalleryMessage {
    GalleryMessage::SidebarSelect(
        ITEMS
            .iter()
            .position(|i| i.0 == value)
            .map_or(UNKNOWN, |i| i as u8),
    )
}

pub(super) fn is_sidebar_hit(message: GalleryMessage) -> bool {
    matches!(message, GalleryMessage::SidebarSelect(_))
}

impl Gallery {
    fn sidebar_demo_builder(&self) -> SidebarBuilder<GalleryMessage> {
        let mut builder = sidebar_builder::<GalleryMessage>()
            .key("gallery_layout_sidebar")
            .state(if self.side_nav.collapsed {
                SidebarState::Collapsed
            } else {
                SidebarState::Expanded
            })
            .on_select(sidebar_message);
        for (i, (value, label, disabled)) in ITEMS.into_iter().enumerate() {
            builder = builder.item(
                SidebarItem::new(label)
                    .value(value)
                    .disabled(disabled)
                    .active(i as u8 == self.side_nav.current),
            );
        }
        builder
    }

    pub(super) fn sidebar_examples(&self) -> WidgetNode<GalleryMessage> {
        let confirmed = self
            .side_nav
            .confirmed
            .map_or("尚未確認", |i| ITEMS[i as usize].1);
        column()
            .key("gallery_layout_sidebar_demo")
            .gap(8.0)
            .child(
                acme_widgets::label_builder(
                    "點選項目切換；F7 進入側邊欄，↓/↑ 移動項目，Enter / Space 確認，←/→ 收合與展開，Esc 離開。",
                )
                .wrap(acme_widgets::LabelWrap::Word)
                .build(),
            )
            .child(
                // Always present: only the variant changes, never the layout.
                card()
                    .key("gallery_layout_sidebar_focus")
                    .variant(if self.side_nav.active {
                        CardVariant::Interactive
                    } else {
                        CardVariant::Plain
                    })
                    .padding(8.0)
                    .child(self.sidebar_demo_builder()),
            )
            .child(label(format!(
                "目前項目：{}",
                ITEMS[self.side_nav.current as usize].1
            )))
            .child(label(format!("已確認：{confirmed}")))
            .child(label(format!(
                "狀態：{}",
                if self.side_nav.collapsed {
                    "收合"
                } else {
                    "展開"
                }
            )))
            .child(label(format!(
                "鍵盤群組：{}",
                if self.side_nav.active {
                    "使用中"
                } else {
                    "未使用"
                }
            )))
            .build()
    }

    /// Activates an item. Unknown and disabled items are rejected.
    pub(super) fn sidebar_select(&mut self, index: u8) -> bool {
        let Some(&(_, _, disabled)) = ITEMS.get(index as usize) else {
            return false;
        };
        if disabled {
            return false;
        }
        self.side_nav.current = index;
        self.side_nav.confirmed = Some(index);
        self.side_nav.active = true;
        self.nav_menu.active = false;
        self.reset_hover_press();
        true
    }

    /// Pointer press over the Snackbar overlay: neither Layout keyboard host
    /// owns the keyboard any more. Changes nothing else.
    pub(super) fn layout_keyboard_release(&mut self) {
        self.nav_menu.active = false;
        self.side_nav.active = false;
    }

    /// Arrow navigation: reveal the current item only when it is hidden.
    fn reveal_sidebar(&mut self) {
        self.reveal_if_hidden(GalleryMessage::SidebarSelect(self.side_nav.current), 80.0);
    }

    /// F7 entry: when the first item or the current item is out of view, scroll
    /// to the group top (margin keeps the section title visible).
    fn reveal_sidebar_group(&mut self) {
        let first = ITEMS.iter().position(|i| !i.2).unwrap_or(0) as u8;
        let first = GalleryMessage::SidebarSelect(first);
        let current = GalleryMessage::SidebarSelect(self.side_nav.current);
        if self.hit_is_hidden(first) == Some(true) || self.hit_is_hidden(current) == Some(true) {
            self.reveal_at(first, 80.0);
        }
    }

    pub(super) fn sidebar_event(&mut self, event: &PlatformEvent) -> Option<bool> {
        match event {
            PlatformEvent::PointerButton { pressed: true, .. } => {
                let on_group = self
                    .hit()
                    .is_some_and(|i| is_sidebar_hit(self.button_info[i].message));
                self.side_nav.active &= on_group;
                None
            }
            PlatformEvent::FocusChanged { gained: false, .. } => {
                self.side_nav.active = false;
                None
            }
            PlatformEvent::Key {
                key: PlatformKey::Function(6),
                pressed: true,
                ctrl: false,
                alt: false,
                meta: false,
                ..
            } => {
                // The NavigationMenu hook takes over; an ignored (composing)
                // F6 must not change this group.
                if self.search.input.preedit.is_empty() {
                    self.side_nav.active = false;
                }
                None
            }
            PlatformEvent::Key {
                key: PlatformKey::Function(7),
                pressed: true,
                ctrl: false,
                alt: false,
                meta: false,
                ..
            } => {
                if !self.search.input.preedit.is_empty() {
                    return Some(false);
                }
                self.search.blur();
                self.side_nav.active = true;
                self.nav_menu.active = false;
                self.reset_hover_press();
                self.reveal_sidebar_group();
                Some(true)
            }
            // Ctrl / Alt / Meta chords (e.g. Alt+Space, the system menu) are not ours.
            PlatformEvent::Key {
                key,
                pressed: true,
                ctrl: false,
                alt: false,
                meta: false,
                repeat,
                ..
            } if self.side_nav.active => {
                if !matches!(
                    key,
                    PlatformKey::ArrowUp
                        | PlatformKey::ArrowDown
                        | PlatformKey::ArrowLeft
                        | PlatformKey::ArrowRight
                        | PlatformKey::Enter
                        | PlatformKey::Space
                        | PlatformKey::Escape
                ) {
                    return None;
                }
                if !self.search.input.preedit.is_empty() {
                    return Some(false);
                }
                // OS auto-repeat must not re-confirm; arrows still rove.
                if *repeat && matches!(key, PlatformKey::Enter | PlatformKey::Space) {
                    return Some(false);
                }
                match key {
                    PlatformKey::ArrowDown | PlatformKey::ArrowUp => {
                        let forward = *key == PlatformKey::ArrowDown;
                        let Some(GalleryMessage::SidebarSelect(index)) =
                            self.sidebar_demo_builder().navigate(forward)
                        else {
                            return Some(false);
                        };
                        if ITEMS.get(index as usize).is_none_or(|i| i.2) {
                            return Some(false);
                        }
                        self.side_nav.current = index;
                        self.reset_hover_press();
                        self.reveal_sidebar();
                        Some(true)
                    }
                    PlatformKey::ArrowLeft | PlatformKey::ArrowRight => {
                        let collapse = *key == PlatformKey::ArrowLeft;
                        let changed = self.side_nav.collapsed != collapse;
                        self.side_nav.collapsed = collapse;
                        self.reset_hover_press();
                        Some(changed)
                    }
                    PlatformKey::Enter | PlatformKey::Space => {
                        Some(self.sidebar_select(self.side_nav.current))
                    }
                    _ => {
                        // Escape leaves the group; it never scrolls.
                        self.side_nav.active = false;
                        self.reset_hover_press();
                        Some(true)
                    }
                }
            }
            _ => None,
        }
    }

    /// Single keyboard owner of the Layout page. The sidebar hook runs first;
    /// F6 / F7 each deactivate the other group, so exactly one can be active.
    pub(super) fn layout_page_event(&mut self, event: &PlatformEvent) -> Option<bool> {
        if let Some(changed) = self.sidebar_event(event) {
            return Some(changed);
        }
        self.nav_menu_event(event)
    }
}
