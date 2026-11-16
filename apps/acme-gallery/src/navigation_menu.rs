//! Controlled NavigationMenu demo with a page-scoped keyboard host.
use super::*;
use acme_platform::PlatformKey;
use acme_ui::layout::{NavMenuItem, NavigationMenuBuilder, nav_menu_item, navigation_menu};

/// Flat demo entries: (value, label, parent value, disabled).
/// The flat index is the payload of `GalleryMessage::NavMenuSelect`.
const ENTRIES: [(&str, &str, Option<&str>, bool); 8] = [
    ("home", "首頁", None, false),
    ("products", "產品", None, false),
    ("desktop", "桌面版", Some("products"), false),
    ("mobile", "行動版", Some("products"), false),
    ("enterprise", "企業版", Some("products"), true),
    ("docs", "文件", None, false),
    ("support", "支援", None, true),
    // Child of a disabled parent: only reachable by a direct call, never by keys.
    ("priority", "優先支援", Some("support"), false),
];

/// Pixels kept above a revealed top-level item so the section heading stays
/// visible (same value as the Sidebar group reveal).
const GROUP_REVEAL_MARGIN: f32 = 80.0;
/// Pixels kept below a child revealed from under the viewport bottom.
const CHILD_REVEAL_GAP: f32 = 8.0;

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

pub(super) fn flat_index(value: &str) -> Option<u8> {
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
            .child(
                // Always present: only the variant changes, never the layout.
                card()
                    .key("gallery_nav_menu_focus")
                    .variant(if self.nav_menu.active {
                        CardVariant::Interactive
                    } else {
                        CardVariant::Plain
                    })
                    .padding(8.0)
                    .child(self.nav_menu_builder()),
            )
            .child(label(format!("目前目的地：{destination}")))
            .child(label(format!(
                "鍵盤選單：{}",
                if self.nav_menu.active {
                    "使用中"
                } else {
                    "未使用"
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
        // A child of a disabled parent is unreachable by pointer; reject it here too.
        if parent
            .and_then(flat_index)
            .and_then(|p| ENTRIES.get(p as usize))
            .is_some_and(|p| p.3)
        {
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
        self.side_nav.active = false;
        self.reset_hover_press();
        true
    }

    /// Scrolls so the hit region of `message` sits `margin` px below the
    /// viewport top, but only when that region is not fully in view.
    pub(super) fn reveal_if_hidden(&mut self, message: GalleryMessage, margin: f32) {
        if self.hit_is_hidden(message) == Some(true) {
            self.reveal_at(message, margin);
        }
    }

    /// `Some(true)` when the hit region of `message` is not fully in view,
    /// `None` when it has no hit region.
    pub(super) fn hit_is_hidden(&self, message: GalleryMessage) -> Option<bool> {
        let hit = self.button_info.iter().find(|h| h.message == message)?;
        let [_, view_top, _, view_height] = self.viewport_clip;
        let top = hit.rect[1] - self.scroll;
        Some(!(top >= view_top && top + hit.rect[3] <= view_top + view_height))
    }

    /// Unconditionally scrolls `message` to `margin` px below the toolbar.
    pub(super) fn reveal_at(&mut self, message: GalleryMessage, margin: f32) {
        if let Some(hit) = self.button_info.iter().find(|h| h.message == message) {
            self.scroll = (hit.rect[1] - TOOLBAR_HEIGHT - margin).clamp(0.0, self.max_scroll);
        }
    }

    /// Minimal scroll that brings `message` fully into view (top edge wins).
    fn reveal_minimal(&mut self, message: GalleryMessage) {
        let Some(hit) = self.button_info.iter().find(|h| h.message == message) else {
            return;
        };
        let [_, view_top, _, view_height] = self.viewport_clip;
        let top = hit.rect[1] - self.scroll;
        let bottom = top + hit.rect[3];
        if top < view_top {
            self.scroll -= view_top - top;
        } else if bottom > view_top + view_height {
            self.scroll += bottom - (view_top + view_height);
        }
        self.scroll = self.scroll.clamp(0.0, self.max_scroll);
    }

    /// Reveals the focused child with the smallest scroll that keeps its parent
    /// row in view; without a child (or before its dropdown has a hit region)
    /// the selected item is revealed under the group margin.
    fn reveal_nav_menu(&mut self) {
        let parent = flat_index(self.nav_menu.item).map(GalleryMessage::NavMenuSelect);
        let child = self
            .nav_menu
            .child
            .and_then(flat_index)
            .map(GalleryMessage::NavMenuSelect)
            .filter(|m| self.hit_is_hidden(*m).is_some());
        if let Some(child) = child {
            if self.hit_is_hidden(child) == Some(true) {
                let hit = self.button_info.iter().find(|h| h.message == child);
                let [_, view_top, _, view_height] = self.viewport_clip;
                let top = hit.map_or(0.0, |h| h.rect[1] - self.scroll);
                let bottom = hit.map_or(0.0, |h| h.rect[1] + h.rect[3] - self.scroll);
                if top < view_top {
                    if let Some(parent) = parent {
                        self.reveal_at(parent, GROUP_REVEAL_MARGIN);
                    }
                } else {
                    self.scroll = (self.scroll + bottom - (view_top + view_height)
                        + CHILD_REVEAL_GAP)
                        .clamp(0.0, self.max_scroll);
                }
                // A viewport too short for both: the child wins.
                if self.hit_is_hidden(child) == Some(true) {
                    self.reveal_minimal(child);
                }
                return;
            }
        }
        if let Some(parent) = parent {
            self.reveal_if_hidden(parent, GROUP_REVEAL_MARGIN);
        }
        // A short viewport: revealing the parent may have hidden the child.
        if let Some(child) = child
            && self.hit_is_hidden(child) == Some(true)
        {
            self.reveal_minimal(child);
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
                ctrl: false,
                alt: false,
                meta: false,
                ..
            } => {
                // Do not steal a navigation key from an active composition.
                if !self.search.input.preedit.is_empty() {
                    return Some(false);
                }
                self.search.blur();
                self.nav_menu.active = true;
                self.side_nav.active = false;
                self.reveal_nav_menu();
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
            } if self.nav_menu.active => {
                // Keys the host does not handle are never consumed.
                if !matches!(
                    key,
                    PlatformKey::ArrowLeft
                        | PlatformKey::ArrowRight
                        | PlatformKey::ArrowDown
                        | PlatformKey::ArrowUp
                        | PlatformKey::Enter
                        | PlatformKey::Space
                        | PlatformKey::Escape
                ) {
                    return None;
                }
                if !self.search.input.preedit.is_empty() {
                    return Some(false);
                }
                // OS auto-repeat must not re-confirm or leave; arrows still rove.
                if *repeat
                    && matches!(
                        key,
                        PlatformKey::Enter | PlatformKey::Space | PlatformKey::Escape
                    )
                {
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
                            // Leaving never scrolls.
                            self.nav_menu.active = false;
                            self.reset_hover_press();
                            return Some(true);
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
