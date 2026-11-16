//! Controlled RadioCards demo with a page-scoped keyboard host.
use super::*;
use acme_platform::PlatformKey;
use acme_ui::inputs::{RadioCardOption, RadioCardsBuilder, radio_cards};

const PLANS: [(&str, &str, &str); 3] = [
    ("starter", "入門方案", "適合個人與小型專案 · Starter"),
    ("pro", "專業方案", "適合成長中的團隊 · Pro"),
    (
        "enterprise",
        "企業方案",
        "適合大型組織與進階安全 · Enterprise",
    ),
];

#[derive(Default)]
pub(super) struct RadioCardsDemo {
    pub selected: Option<u8>,
    pub confirmed: Option<u8>,
    pub active: bool,
}

/// Fn pointers cannot capture, so the option value maps back to its index.
fn plan_message(value: &str) -> GalleryMessage {
    let index = PLANS.iter().position(|p| p.0 == value).unwrap_or(0);
    GalleryMessage::RadioCardSelect(index as u8)
}

pub(super) fn is_radio_card_hit(message: GalleryMessage) -> bool {
    matches!(message, GalleryMessage::RadioCardSelect(_))
}

impl Gallery {
    fn radio_cards_builder(&self) -> RadioCardsBuilder<GalleryMessage> {
        let mut builder = radio_cards::<GalleryMessage>()
            .key("gallery_radio_cards")
            .on_select(plan_message);
        for (value, title, description) in PLANS {
            builder = builder.option(RadioCardOption::new(value, title).description(description));
        }
        if let Some(index) = self.radio_cards.selected {
            builder = builder.selected(PLANS[index as usize].0);
        }
        builder
    }

    pub(super) fn radio_cards_examples(&self) -> WidgetNode<GalleryMessage> {
        let name = |slot: Option<u8>| slot.map_or("尚未選擇", |i| PLANS[i as usize].1);
        column()
            .key("gallery_radio_cards_demo")
            .gap(8.0)
            .child(
                acme_widgets::label_builder(
                    "點選卡片選擇方案；F6 進入群組，方向鍵移動，Enter / Space 確認，Esc 離開。",
                )
                .wrap(acme_widgets::LabelWrap::Word)
                .build(),
            )
            .child(column().width(360.0).child(self.radio_cards_builder()))
            .child(label(format!(
                "目前方案：{}",
                name(self.radio_cards.selected)
            )))
            .child(label(format!(
                "已確認：{}",
                name(self.radio_cards.confirmed)
            )))
            .child(label(format!(
                "Keyboard group: {}",
                if self.radio_cards.active {
                    "active"
                } else {
                    "inactive"
                }
            )))
            .build()
    }

    pub(super) fn radio_card_select(&mut self, index: u8) -> bool {
        if index as usize >= PLANS.len() {
            return false;
        }
        self.radio_cards.selected = Some(index);
        self.radio_cards.confirmed = Some(index);
        self.radio_cards.active = true;
        self.checkbox_cards.active = false;
        self.reset_hover_press();
        true
    }

    /// F6 entry: when the first or last card is out of view, scroll to the
    /// group top; a fully visible group never moves the page.
    fn reveal_radio_group(&mut self) {
        let last = (PLANS.len() - 1) as u8;
        let first = GalleryMessage::RadioCardSelect(0);
        let last = GalleryMessage::RadioCardSelect(last);
        if self.hit_is_hidden(last) == Some(true) {
            self.reveal_at(first, 80.0);
        } else {
            self.reveal_if_hidden(first, 80.0);
        }
    }

    pub(super) fn radio_cards_event(&mut self, event: &PlatformEvent) -> Option<bool> {
        match event {
            PlatformEvent::PointerButton { pressed: true, .. } => {
                let on_card = self
                    .hit()
                    .is_some_and(|i| is_radio_card_hit(self.button_info[i].message));
                self.radio_cards.active &= on_card;
                None // Pointer activation remains in the shared Gallery handler.
            }
            PlatformEvent::FocusChanged { gained: false, .. } => {
                self.radio_cards.active = false;
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
                self.radio_cards.active = true;
                self.checkbox_cards.active = false;
                self.reset_hover_press();
                self.reveal_radio_group();
                Some(true)
            }
            PlatformEvent::Key {
                key,
                pressed: true,
                ctrl: false,
                alt: false,
                meta: false,
                ..
            } if self.radio_cards.active && self.search.input.preedit.is_empty() => {
                let forward = match key {
                    PlatformKey::ArrowDown | PlatformKey::ArrowRight => true,
                    PlatformKey::ArrowUp | PlatformKey::ArrowLeft => false,
                    PlatformKey::Enter | PlatformKey::Space => {
                        self.radio_cards.confirmed = self.radio_cards.selected;
                        let changed = self.radio_cards.selected.is_some();
                        if changed {
                            self.reset_hover_press();
                        }
                        return Some(changed);
                    }
                    PlatformKey::Escape => {
                        self.radio_cards.active = false;
                        self.reset_hover_press();
                        return Some(true);
                    }
                    _ => return None,
                };
                let Some(GalleryMessage::RadioCardSelect(index)) =
                    self.radio_cards_builder().navigate(forward)
                else {
                    return Some(false);
                };
                self.radio_cards.selected = Some(index);
                self.reset_hover_press();
                Some(true)
            }
            _ => None,
        }
    }
}
