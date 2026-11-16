//! Controlled NavigationMenu demo with a page-scoped keyboard host.
use super::*;
use acme_platform::PlatformKey;
use acme_ui::layout::{NavMenuItem, NavigationMenuBuilder, nav_menu_item, navigation_menu};

/// Flat demo entries: (value, label, parent value, disabled).
/// The flat index is the payload of `GalleryMessage::NavMenuSelect`.
const ENTRIES: [(&str, &str, Option<&str>, bool); 7] = [
    ("home", "首頁", None, false),
    ("products", "產品", None, false),
    ("desktop", "桌面版", Some("products"), false),
    ("mobile", "行動版", Some("products"), false),
    ("enterprise", "企業版", Some("products"), true),
    ("docs", "文件", None, false),
    ("support", "支援", None, true),
];

/// Sentinel for a value that is not in `ENTRIES`; the handler rejects it.
const UNKNOWN: u8 = u8::MAX;

pub(super) struct NavMenuDemo {
    /// Selected top-level value.
    pub item: &'static str,
    /// Keyboard-focused / active child of the selected item.
    pub child: Option<&'static str>,
    /// Flat index of the last confirmed destination.
    pub destination: Option<u8>,
    pub active: bool,
}

impl Default for NavMenuDemo {
    fn default() -> Self {
        Self {
            item: "home",
            child: None,
            destination: None,
            active: false,
        }
    }
}

fn flat_index(value: &str) -> Option<u8> {
    ENTRIES.iter().position(|e| e.0 == value).map(|i| i as u8)
}

/// Fn pointers cannot capture, so the value maps back to its flat index.
fn nav_message(value: &str) -> GalleryMessage {
    GalleryMessage::NavMenuSelect(flat_index(value).unwrap_or(UNKNOWN))
}

pub(super) fn is_nav_menu_hit(message: GalleryMessage) -> bool {
    matches!(message, GalleryMessage::NavMenuSelect(_))
}

impl Gallery {
    fn nav_menu_builder(&self) -> NavigationMenuBuilder<GalleryMessage> {
        let mut builder = navigation_menu::<GalleryMessage>("gallery_nav_menu")
            .on_select(nav_message)
            .selected(self.nav_menu.item);
        for (value, label, parent, disabled) in ENTRIES {
            if parent.is_some() {
                continue;
            }
            let mut item: NavMenuItem = nav_menu_item(label, value).disabled(disabled);
            for (cv, cl, cp, cd) in ENTRIES {
                if cp == Some(value) {
                    item = item.child(nav_menu_item(cl, cv).disabled(cd));
                }
            }
            builder = builder.item(item);
        }
        if let Some(child) = self.nav_menu.child {
            builder = builder.active_child(child);
        }
        builder
    }

    pub(super) fn nav_menu_examples(&self) -> WidgetNode<GalleryMessage> {
        let destination = self
            .nav_menu
            .destination
            .map_or("尚未前往", |i| ENTRIES[i as usize].1);
        column()
            .key("gallery_nav_menu_demo")
            .gap(8.0)
            .child(
                acme_widgets::label_builder(
                    "點選項目前往；F6 進入選單，←/→ 移動項目，↓/↑ 移動子項目，Enter / Space 確認，Esc 離開。",
                )
                .wrap(acme_widgets::LabelWrap::Word)
                .build(),
            )
            .child(self.nav_menu_builder())
            .child(label(format!("目前目的地：{destination}")))
            .child(label(format!(
                "Keyboard menu: {}",
                if self.nav_menu.active {
                    "active"
                } else {
                    "inactive"
                }
            )))
            .build()
    }

    /// Activates a flat entry. Unknown and disabled entries are rejected.
    pub(super) fn nav_menu_select(&mut self, index: u8) -> bool {
        let Some(&(value, _, parent, disabled)) = ENTRIES.get(index as usize) else {
            return false;
        };
        if disabled {
            return false;
        }
        match parent {
            Some(parent) => {
                self.nav_menu.item = ENTRIES[flat_index(parent).unwrap() as usize].0;
                self.nav_menu.child = Some(value);
            }
            None => {
                self.nav_menu.item = value;
                self.nav_menu.child = None;
            }
        }
        self.nav_menu.destination = Some(index);
        self.nav_menu.active = true;
        self.reset_hover_press();
        true
    }

    fn reveal_nav_menu(&mut self) {
        let Some(index) = flat_index(self.nav_menu.item) else {
            return;
        };
        if let Some(hit) = self
            .button_info
            .iter()
            .find(|h| h.message == GalleryMessage::NavMenuSelect(index))
        {
            self.scroll = (hit.rect[1] - TOOLBAR_HEIGHT - 8.0).clamp(0.0, self.max_scroll);
        }
    }

    pub(super) fn nav_menu_event(&mut self, event: &PlatformEvent) -> Option<bool> {
        match event {
            PlatformEvent::PointerButton { pressed: true, .. } => {
                let on_menu = self
                    .hit()
                    .is_some_and(|i| is_nav_menu_hit(self.button_info[i].message));
                self.nav_menu.active &= on_menu;
                None // Pointer activation remains in the shared Gallery handler.
            }
            PlatformEvent::FocusChanged { gained: false, .. } => {
                self.nav_menu.active = false;
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
                self.nav_menu.active = true;
                self.reveal_nav_menu();
                Some(true)
            }
            PlatformEvent::Key {
                key, pressed: true, ..
            } if self.nav_menu.active => {
                if !self.search.input.preedit.is_empty() {
                    return Some(false);
                }
                match key {
                    PlatformKey::ArrowLeft | PlatformKey::ArrowRight => {
                        let forward = *key == PlatformKey::ArrowRight;
                        let Some(GalleryMessage::NavMenuSelect(index)) =
                            self.nav_menu_builder().navigate(forward)
                        else {
                            return Some(false);
                        };
                        let Some(&(value, _, None, false)) = ENTRIES.get(index as usize) else {
                            return Some(false);
                        };
                        self.nav_menu.item = value;
                        self.nav_menu.child = None;
                    }
                    PlatformKey::ArrowDown | PlatformKey::ArrowUp => {
                        let forward = *key == PlatformKey::ArrowDown;
                        let next = self
                            .nav_menu_builder()
                            .navigate_child(self.nav_menu.child, forward)
                            .and_then(flat_index);
                        let Some(index) = next else {
                            return Some(false);
                        };
                        self.nav_menu.child = Some(ENTRIES[index as usize].0);
                    }
                    PlatformKey::Enter | PlatformKey::Space => {
                        let target = self
                            .nav_menu
                            .child
                            .and_then(flat_index)
                            .or_else(|| flat_index(self.nav_menu.item));
                        return Some(target.is_some_and(|i| self.nav_menu_select(i)));
                    }
                    PlatformKey::Escape => {
                        if self.nav_menu.child.is_some() {
                            self.nav_menu.child = None;
                        } else {
                            self.nav_menu.active = false;
                        }
                    }
                    _ => return None,
                }
                self.reset_hover_press();
                self.reveal_nav_menu();
                Some(true)
            }
            _ => None,
        }
    }
}
