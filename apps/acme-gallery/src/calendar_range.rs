//! Host-controlled visual range preview; calendar cells remain passive.
use super::*;
use acme_ui::foundations::calendar;

impl Gallery {
    pub(super) fn calendar_range_examples(&self) -> WidgetNode<GalleryMessage> {
        let mut preview = calendar(2026, 11).today_day(16);
        if self.calendar_range {
            preview = preview.selected_range(5, 12);
        }
        column()
            .key("gallery_calendar_range")
            .gap(8.0)
            .child(
                acme_widgets::label_builder("November 5–12 · 日期範圍預覽（日期格僅供顯示）")
                    .wrap(acme_widgets::LabelWrap::Word)
                    .build(),
            )
            .child(
                button(
                    "gallery_calendar_range_toggle",
                    if self.calendar_range {
                        "Clear range"
                    } else {
                        "Select Nov 5–12"
                    },
                )
                .on_click(GalleryMessage::CalendarRangeToggle),
            )
            .child(
                column()
                    .key("gallery_calendar_preview")
                    .width(240.0)
                    .child(preview),
            )
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gallery_calendar_range_pointer_can_clear_and_restore_the_preview() {
        let mut g = Gallery::new();
        g.selected_category = 1;
        let frame = |g: &mut Gallery| {
            g.frame(FrameContext {
                window: WindowId(1),
                logical_width: 1100.0,
                logical_height: 1000.0,
                scale_factor: 1.0,
            });
        };
        frame(&mut g);
        for expected in [false, true] {
            let r = g
                .button_info
                .iter()
                .find(|h| h.message == GalleryMessage::CalendarRangeToggle)
                .unwrap()
                .rect;
            let (x, y) = (r[0] + r[2] / 2.0, r[1] + r[3] / 2.0 - g.scroll);
            assert!(y > TOOLBAR_HEIGHT && y < 1000.0);
            println!("Calendar toggle pointer: {x}, {y}");
            g.event(PlatformEvent::PointerMoved {
                window: WindowId(1),
                x,
                y,
            });
            for pressed in [true, false] {
                assert!(g.event(PlatformEvent::PointerButton {
                    window: WindowId(1),
                    pressed,
                    x,
                    y,
                    button: 0,
                    pointer: 0,
                    shift: false,
                    ctrl: false,
                    alt: false,
                }));
            }
            assert_eq!(g.calendar_range, expected);
            frame(&mut g);
        }
    }

    #[test]
    fn gallery_calendar_range_paints_theme_accents_with_matching_hit_slots_and_clips() {
        let mut g = Gallery::new();
        for pack in ["default", "apple"] {
            g.theme_pack = pack.into();
            for dark in [false, true] {
                g.dark = dark;
                let theme = g.current_theme();
                for scale in [1.0, 1.25, 2.0] {
                    for selected in [true, false] {
                        g.calendar_range = selected;
                        let widget = g.calendar_range_examples();
                        let context = WidgetLayoutContext {
                            body_font_size: theme.typography.body,
                            body_line_height: theme.typography.body * theme.typography.line_height,
                            label_font_size: theme.typography.label,
                            control_height: 36.0,
                            scale_factor: scale,
                        };
                        let layout = widget.to_layout_with_context(NodeId::new(1), &context);
                        let snapshot = g
                            .layout
                            .compute_with_text(&layout, (400.0, 600.0), &mut g.fonts, scale)
                            .unwrap();
                        let mut hits = Vec::new();
                        collect_hit_regions(&widget, &layout, &snapshot, true, &mut hits);
                        assert_eq!(hits.len(), 1, "date cells are passive");
                        assert_eq!(hits[0].message, GalleryMessage::CalendarRangeToggle);
                        let clip = [5.0, 6.0, 380.0, 550.0];
                        let mut paint = Frame::default();
                        let mut index = 0;
                        render_content(
                            &mut paint,
                            &widget,
                            &layout,
                            &snapshot,
                            &theme,
                            scale,
                            4.0,
                            clip,
                            &mut index,
                            None,
                            None,
                            &mut g.fonts,
                            &mut g.atlas,
                        );
                        assert_eq!(index, hits.len());
                        assert!(paint.clipped_quads.iter().all(|q| q.clip == clip));
                        let range: Vec<_> = paint
                            .clipped_quads
                            .iter()
                            .filter(|q| {
                                q.quad.color == rgba(theme.colors.accent)
                                    && q.quad.border_width == 0.0
                            })
                            .collect();
                        assert_eq!(range.len(), if selected { 8 } else { 0 });
                        for cell in range {
                            let [x, y, w, h] = cell.quad.rect;
                            assert_eq!(w, 28.0);
                            assert!(x >= 0.0 && x + w <= 240.0 && y >= 0.0 && h > 0.0);
                        }
                        if selected {
                            assert!(
                                paint
                                    .text
                                    .iter()
                                    .any(|t| t.color == rgba(theme.colors.accent_foreground))
                            );
                        }
                    }
                }
            }
        }
    }
}
