//! Typed Sidenav demo with a page-scoped keyboard host (F8). Part of the
//! Layout keyboard owner: NavigationMenu F6 / Sidebar F7 / Sidenav F8.
use super::*;
use acme_platform::PlatformKey;
use acme_ui::desktop::{SidenavBuilder, sidenav_item, sidenav_section};
// Imported by path: `use super::*` may also bring in a `sidenav` module/widget.
use acme_ui::desktop::sidenav as sidenav_builder;

/// Demo items: (value, label, disabled, section index). The flat index is
/// the message payload.
const ITEMS: [(&str, &str, bool, u8); 4] = [
    ("profile", "Profile", false, 0),
    ("appearance", "Appearance", false, 0),
    ("privacy", "Privacy", false, 1),
    ("security", "Security", true, 1),
];
const SECTIONS: [&str; 2] = ["General", "Advanced"];

/// Pixels kept above a revealed group so the section title stays visible.
const GROUP_REVEAL_MARGIN: f32 = 80.0;

/// Sentinel for a value that is not in `ITEMS`; the handler rejects it.
const UNKNOWN: u8 = u8::MAX;

#[derive(Default)]
pub(super) struct SidenavDemo {
    /// Index of the current (keyboard-focused, selected) item.
    pub current: u8,
    /// Index of the last confirmed item.
    pub confirmed: Option<u8>,
    pub active: bool,
}

/// Fn pointers cannot capture, so the value maps back to its index.
pub(super) fn sidenav_message(value: &str) -> GalleryMessage {
    GalleryMessage::SidenavSelect(
        ITEMS
            .iter()
            .position(|i| i.0 == value)
            .map_or(UNKNOWN, |i| i as u8),
    )
}

pub(super) fn is_sidenav_hit(message: GalleryMessage) -> bool {
    matches!(message, GalleryMessage::SidenavSelect(_))
}

impl Gallery {
    fn sidenav_demo_builder(&self) -> SidenavBuilder<GalleryMessage> {
        let mut builder = sidenav_builder::sidenav::<GalleryMessage>("gallery_layout_sidenav")
            .on_select(sidenav_message)
            .selected(ITEMS[self.sidenav.current as usize].0);
        for (s, title) in SECTIONS.into_iter().enumerate() {
            let mut section = sidenav_section::<GalleryMessage>(title);
            for (value, label, disabled, _) in ITEMS.into_iter().filter(|i| i.3 as usize == s) {
                let mut item = sidenav_item(label, value);
                if disabled {
                    item = item.disabled();
                }
                section = section.item(item);
            }
            builder = builder.section(section);
        }
        builder
    }

    pub(super) fn sidenav_examples(&self) -> WidgetNode<GalleryMessage> {
        let confirmed = self
            .sidenav
            .confirmed
            .map_or("尚未確認", |i| ITEMS[i as usize].1);
        column()
            .key("gallery_layout_sidenav_demo")
            .gap(8.0)
            .child(
                acme_widgets::label_builder(
                    "點選項目切換；F8 進入側邊導覽，↓/↑ 移動項目（跨區段），Enter / Space 確認，Esc 離開。",
                )
                .wrap(acme_widgets::LabelWrap::Word)
                .build(),
            )
            .child(
                // Always present: only the variant changes, never the layout.
                card()
                    .key("gallery_layout_sidenav_focus")
                    .variant(if self.sidenav.active {
                        CardVariant::Interactive
                    } else {
                        CardVariant::Plain
                    })
                    .padding(8.0)
                    .child(self.sidenav_demo_builder()),
            )
            .child(label(format!(
                "Sidenav selected: {}",
                ITEMS[self.sidenav.current as usize].1
            )))
            .child(label(format!("已確認：{confirmed}")))
            .child(label(format!(
                "鍵盤導覽：{}",
                if self.sidenav.active {
                    "使用中"
                } else {
                    "未使用"
                }
            )))
            .build()
    }

    /// Activates an item. Unknown and disabled items are rejected.
    pub(super) fn sidenav_select(&mut self, index: u8) -> bool {
        let Some(&(_, _, disabled, _)) = ITEMS.get(index as usize) else {
            return false;
        };
        if disabled {
            return false;
        }
        self.sidenav.current = index;
        self.sidenav.confirmed = Some(index);
        self.sidenav.active = true;
        self.nav_menu.active = false;
        self.side_nav.active = false;
        self.reset_hover_press();
        true
    }

    /// F8 entry: when the first item or the current item is out of view, scroll
    /// to the group top (margin keeps the section title visible).
    fn reveal_sidenav_group(&mut self) {
        let first = ITEMS.iter().position(|i| !i.2).unwrap_or(0) as u8;
        let first = GalleryMessage::SidenavSelect(first);
        let current = GalleryMessage::SidenavSelect(self.sidenav.current);
        if self.hit_is_hidden(first) == Some(true) || self.hit_is_hidden(current) == Some(true) {
            self.reveal_at(first, GROUP_REVEAL_MARGIN);
            // A short viewport: the group reveal may have hidden the current item.
            if self.hit_is_hidden(current) == Some(true) {
                self.reveal_minimal(current);
            }
        }
    }

    pub(super) fn sidenav_event(&mut self, event: &PlatformEvent) -> Option<bool> {
        match event {
            PlatformEvent::PointerButton { pressed: true, .. } => {
                let on_group = self
                    .hit()
                    .is_some_and(|i| is_sidenav_hit(self.button_info[i].message));
                self.sidenav.active &= on_group;
                None
            }
            PlatformEvent::FocusChanged { gained: false, .. } => {
                self.sidenav.active = false;
                None
            }
            PlatformEvent::Key {
                key: PlatformKey::Function(8),
                pressed: true,
                ctrl: false,
                alt: false,
                meta: false,
                repeat,
                ..
            } => {
                // Auto-repeat must not reset hover/press or reveal again.
                if *repeat || !self.search.input.preedit.is_empty() {
                    return Some(false);
                }
                self.search.blur();
                self.sidenav.active = true;
                self.nav_menu.active = false;
                self.side_nav.active = false;
                self.reset_hover_press();
                self.reveal_sidenav_group();
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
            } if self.sidenav.active => {
                if !matches!(
                    key,
                    PlatformKey::ArrowUp
                        | PlatformKey::ArrowDown
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
                        let Some(GalleryMessage::SidenavSelect(index)) =
                            self.sidenav_demo_builder().navigate(forward)
                        else {
                            return Some(false);
                        };
                        if ITEMS.get(index as usize).is_none_or(|i| i.2) {
                            return Some(false);
                        }
                        self.sidenav.current = index;
                        self.reset_hover_press();
                        self.reveal_minimal(GalleryMessage::SidenavSelect(index));
                        Some(true)
                    }
                    PlatformKey::Enter | PlatformKey::Space => {
                        Some(self.sidenav_select(self.sidenav.current))
                    }
                    _ => {
                        // Escape leaves the group; it never scrolls.
                        self.sidenav.active = false;
                        self.reset_hover_press();
                        Some(true)
                    }
                }
            }
            _ => None,
        }
    }
}
