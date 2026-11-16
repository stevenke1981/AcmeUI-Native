//! Controlled CheckboxCards demo with a page-scoped keyboard host, plus the
//! single keyboard owner for the Inputs page (RadioCards F6 / CheckboxCards F7).
use super::*;
use acme_platform::PlatformKey;
use acme_ui::inputs::{CheckboxCardOption, CheckboxCardsBuilder, checkbox_cards};

/// Demo options: (value, label). The index is the message payload.
const CHANNELS: [(&str, &str); 4] = [
    ("email", "電子郵件"),
    ("sms", "簡訊"),
    ("push", "推播通知"),
    ("digest", "每週摘要"),
];

/// Sentinel for a value that is not in `CHANNELS`; the handler rejects it.
const UNKNOWN: u8 = u8::MAX;

#[derive(Default)]
pub(super) struct CheckboxCardsDemo {
    pub checked: [bool; CHANNELS.len()],
    /// Index of the keyboard-focused option.
    pub focus: Option<u8>,
    pub active: bool,
}

/// Fn pointers cannot capture, so the value maps back to its index.
pub(super) fn checkbox_message(value: &str) -> GalleryMessage {
    GalleryMessage::CheckboxCardToggle(
        CHANNELS
            .iter()
            .position(|c| c.0 == value)
            .map_or(UNKNOWN, |i| i as u8),
    )
}

pub(super) fn is_checkbox_card_hit(message: GalleryMessage) -> bool {
    matches!(message, GalleryMessage::CheckboxCardToggle(_))
}

fn cb(i: u8) -> GalleryMessage {
    GalleryMessage::CheckboxCardToggle(i)
}

impl Gallery {
    fn checkbox_cards_builder(&self) -> CheckboxCardsBuilder<GalleryMessage> {
        let mut builder = checkbox_cards::<GalleryMessage>()
            .key("gallery_checkbox_cards")
            .columns(1)
            .on_toggle(checkbox_message)
            .selected(
                CHANNELS
                    .iter()
                    .zip(self.checkbox_cards.checked)
                    .filter(|(_, on)| *on)
                    .map(|(c, _)| c.0)
                    .collect::<Vec<_>>(),
            );
        for (value, title) in CHANNELS {
            builder = builder.option(CheckboxCardOption::new(value, title));
        }
        if let Some(index) = self
            .checkbox_cards
            .focus
            .filter(|_| self.checkbox_cards.active)
        {
            builder = builder.focused(CHANNELS[index as usize].0);
        }
        builder
    }

    pub(super) fn checkbox_cards_examples(&self) -> WidgetNode<GalleryMessage> {
        let checked: Vec<&str> = CHANNELS
            .iter()
            .zip(self.checkbox_cards.checked)
            .filter(|(_, on)| *on)
            .map(|(c, _)| c.1)
            .collect();
        let checked = if checked.is_empty() {
            "尚未勾選".to_string()
        } else {
            checked.join("、")
        };
        let focus = self
            .checkbox_cards
            .focus
            .map_or("尚未聚焦", |i| CHANNELS[i as usize].1);
        column()
            .key("gallery_checkbox_cards_demo")
            .gap(8.0)
            .child(
                acme_widgets::label_builder(
                    "點選卡片切換勾選；F7 進入群組，方向鍵移動焦點，Space 切換，Esc 離開。",
                )
                .wrap(acme_widgets::LabelWrap::Word)
                .build(),
            )
            .child(column().width(360.0).child(self.checkbox_cards_builder()))
            .child(label(format!("已勾選：{checked}")))
            .child(label(format!("目前焦點：{focus}")))
            .child(label(format!(
                "Keyboard group: {}",
                if self.checkbox_cards.active {
                    "active"
                } else {
                    "inactive"
                }
            )))
            .build()
    }

    /// Toggles an option and activates the group. Unknown indices are rejected.
    pub(super) fn checkbox_card_toggle(&mut self, index: u8) -> bool {
        let Some(slot) = self.checkbox_cards.checked.get_mut(index as usize) else {
            return false;
        };
        *slot = !*slot;
        self.checkbox_cards.focus = Some(index);
        self.checkbox_cards.active = true;
        self.radio_cards.active = false;
        self.reset_hover_press();
        true
    }

    fn reveal_checkbox_focus(&mut self) {
        if let Some(index) = self.checkbox_cards.focus {
            self.reveal_if_hidden(cb(index), 80.0);
        }
    }

    /// F7 entry: when the first, last or focused card is out of view, scroll to
    /// the group top (margin keeps the section title visible).
    fn reveal_checkbox_group(&mut self) {
        let last = (CHANNELS.len() - 1) as u8;
        let mut targets = vec![cb(0), cb(last)];
        targets.extend(self.checkbox_cards.focus.map(cb));
        if targets.iter().any(|m| self.hit_is_hidden(*m) == Some(true)) {
            self.reveal_at(cb(0), 80.0);
            // A group taller than the viewport must not hide the focused card.
            self.reveal_checkbox_focus();
        }
    }

    pub(super) fn checkbox_cards_event(&mut self, event: &PlatformEvent) -> Option<bool> {
        match event {
            PlatformEvent::PointerButton { pressed: true, .. } => {
                let on_group = self
                    .hit()
                    .is_some_and(|i| is_checkbox_card_hit(self.button_info[i].message));
                self.checkbox_cards.active &= on_group;
                None // Pointer activation remains in the shared Gallery handler.
            }
            PlatformEvent::FocusChanged { gained: false, .. } => {
                self.checkbox_cards.active = false;
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
                // The RadioCards hook takes over; an ignored (composing) F6
                // must not change this group.
                if self.search.input.preedit.is_empty() {
                    self.checkbox_cards.active = false;
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
                self.checkbox_cards.active = true;
                self.radio_cards.active = false;
                self.checkbox_cards.focus.get_or_insert(0);
                self.reset_hover_press();
                self.reveal_checkbox_group();
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
            } if self.checkbox_cards.active => {
                if !matches!(
                    key,
                    PlatformKey::ArrowUp
                        | PlatformKey::ArrowDown
                        | PlatformKey::ArrowLeft
                        | PlatformKey::ArrowRight
                        | PlatformKey::Space
                        | PlatformKey::Escape
                ) {
                    return None; // Enter and other keys pass through.
                }
                if !self.search.input.preedit.is_empty() {
                    return Some(false);
                }
                match key {
                    PlatformKey::Space => {
                        // OS auto-repeat must not flicker the toggle.
                        if *repeat {
                            return Some(false);
                        }
                        let Some(index) = self.checkbox_cards.focus else {
                            return Some(false);
                        };
                        let changed = self.checkbox_card_toggle(index);
                        self.reveal_checkbox_focus();
                        Some(changed)
                    }
                    PlatformKey::Escape => {
                        // Escape leaves the group; it never scrolls.
                        self.checkbox_cards.active = false;
                        self.reset_hover_press();
                        Some(true)
                    }
                    _ => {
                        let forward =
                            matches!(key, PlatformKey::ArrowDown | PlatformKey::ArrowRight);
                        let current = self.checkbox_cards.focus.map(|i| CHANNELS[i as usize].0);
                        let builder = self.checkbox_cards_builder();
                        let next = builder
                            .navigate_focus(current, forward)
                            .and_then(|v| CHANNELS.iter().position(|c| c.0 == v));
                        let Some(next) = next else {
                            return Some(false);
                        };
                        self.checkbox_cards.focus = Some(next as u8);
                        self.reset_hover_press();
                        self.reveal_checkbox_focus();
                        Some(true)
                    }
                }
            }
            _ => None,
        }
    }

    /// Single keyboard owner of the Inputs page. The CheckboxCards hook runs
    /// first; F6 / F7 each deactivate the other group, so at most one is active.
    pub(super) fn inputs_keyboard_event(&mut self, event: &PlatformEvent) -> Option<bool> {
        if let Some(changed) = self.checkbox_cards_event(event) {
            return Some(changed);
        }
        self.radio_cards_event(event)
    }
}
