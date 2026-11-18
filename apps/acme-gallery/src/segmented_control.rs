//! Controlled report period selector on the Inputs page.
use super::*;
use acme_platform::PlatformKey;
use acme_ui::inputs::{SegmentedControlBuilder, segmented_control};

const PERIODS: [(&str, &str); 3] = [("day", "今日"), ("week", "本週"), ("month", "本月")];
const UNAVAILABLE: usize = 1;

#[derive(Default)]
pub(super) struct SegmentedControlDemo {
    pub selected: usize,
    pub active: bool,
}

impl Gallery {
    fn segmented_control_builder(&self) -> SegmentedControlBuilder<GalleryMessage> {
        let mut control = segmented_control("gallery_report_period")
            .selected(self.segments.selected)
            .on_select(GalleryMessage::SegmentSelect)
            .disable_item(UNAVAILABLE);
        for (key, label) in PERIODS {
            control = control.keyed_item(key, label);
        }
        control
    }

    pub(super) fn segmented_control_example(&self) -> WidgetNode<GalleryMessage> {
        let (key, period) = PERIODS[self.segments.selected];
        let total = if key == "day" { 4 } else { 37 };
        column()
            .key("gallery_report_period_demo")
            .gap(8.0)
            .child(
                acme_widgets::label_builder(
                    "選擇報表期間。點選或按 F8 進入群組，按 ← / → 切換，按 Tab 或 Esc 離開。本週資料暫未提供。",
                )
                .wrap(acme_widgets::LabelWrap::Word)
                .build(),
            )
            .child(
                card()
                    .key("gallery_report_period_focus")
                    .variant(if self.segments.active {
                        CardVariant::Interactive
                    } else {
                        CardVariant::Plain
                    })
                    .padding(8.0)
                    .child(self.segmented_control_builder()),
            )
            .child(label(format!("報表期間：{period} · 已完成 {total} 件")))
            .build()
    }

    pub(super) fn select_segment(&mut self, index: usize) -> bool {
        if index == UNAVAILABLE || index >= PERIODS.len() {
            return false;
        }
        self.segments.selected = index;
        self.segments.active = true;
        self.radio_cards.active = false;
        self.checkbox_cards.active = false;
        self.reset_hover_press();
        true
    }

    pub(super) fn segmented_control_event(&mut self, event: &PlatformEvent) -> Option<bool> {
        match event {
            PlatformEvent::PointerButton { pressed: true, .. } => {
                self.segments.active &= self.hit().is_some_and(|index| {
                    matches!(
                        self.button_info[index].message,
                        GalleryMessage::SegmentSelect(_)
                    )
                });
                None
            }
            PlatformEvent::FocusChanged { gained: false, .. } if self.segments.active => {
                self.segments.active = false;
                Some(true)
            }
            PlatformEvent::Key {
                key,
                pressed: true,
                ctrl: false,
                alt: false,
                meta: false,
                ..
            } if self.search.input.preedit.is_empty() => {
                match key {
                    PlatformKey::Function(6) | PlatformKey::Function(7) => {
                        // Other Inputs groups handle these keys.
                        self.segments.active = false;
                        None
                    }
                    PlatformKey::Function(8) => {
                        self.search.blur();
                        self.select_segment(self.segments.selected);
                        self.reveal_minimal(GalleryMessage::SegmentSelect(self.segments.selected));
                        Some(true)
                    }
                    PlatformKey::Tab | PlatformKey::Escape if self.segments.active => {
                        self.segments.active = false;
                        self.reset_hover_press();
                        Some(true)
                    }
                    PlatformKey::ArrowLeft | PlatformKey::ArrowRight if self.segments.active => {
                        let Some(GalleryMessage::SegmentSelect(index)) = self
                            .segmented_control_builder()
                            .navigate(*key == PlatformKey::ArrowRight)
                        else {
                            return Some(false);
                        };
                        let changed = self.select_segment(index);
                        self.reveal_minimal(GalleryMessage::SegmentSelect(index));
                        Some(changed)
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }
}
