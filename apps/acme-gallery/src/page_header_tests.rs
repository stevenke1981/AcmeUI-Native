use super::*;

fn frame(g: &mut Gallery) {
    g.frame(FrameContext {
        window: WindowId(1),
        logical_width: 1100.0,
        logical_height: 760.0,
        scale_factor: 1.0,
    });
}

#[test]
fn gallery_page_header_actions_survive_pointer_description_changes() {
    let mut g = Gallery::new();
    g.selected_category = 3;
    frame(&mut g);
    for (message, saves, description) in [
        (GalleryMessage::HeaderSave, 1, true),
        (GalleryMessage::HeaderToggleDescription, 1, false),
        (GalleryMessage::HeaderSave, 2, false),
        (GalleryMessage::HeaderToggleDescription, 2, true),
        (GalleryMessage::HeaderReset, 0, true),
    ] {
        let hit = g.button_info.iter().find(|h| h.message == message).unwrap();
        let (x, y) = (
            hit.rect[0] + hit.rect[2] / 2.0,
            hit.rect[1] + hit.rect[3] / 2.0 - g.scroll,
        );
        assert!((40.0..760.0).contains(&y));
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
                alt: false
            }));
        }
        frame(&mut g);
        assert_eq!(g.header_saves, saves);
        assert_eq!(g.header_description, description);
        assert!(g.pressed.is_none());
    }
}

#[test]
fn page_header_gallery_keeps_action_paint_hits_and_bounds_across_subtitle_theme_and_scale() {
    let mut g = Gallery::new();
    for dark in [false, true] {
        g.dark = dark;
        let theme = g.current_theme();
        for scale in [1.0, 1.25, 2.0] {
            for description in [false, true] {
                g.header_description = description;
                let widget = g.page_header_examples();
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
                    .compute_with_text(&layout, (400.0, 300.0), &mut g.fonts, scale)
                    .unwrap();
                let mut hits = Vec::new();
                collect_hit_regions(&widget, &layout, &snapshot, true, &mut hits);
                assert_eq!(
                    hits.iter().map(|h| h.message).collect::<Vec<_>>(),
                    [
                        GalleryMessage::HeaderSave,
                        GalleryMessage::HeaderReset,
                        GalleryMessage::HeaderToggleDescription
                    ]
                );
                assert!(hits.iter().all(|h| h.rect[0] >= 0.0
                    && h.rect[0] + h.rect[2] <= 400.01
                    && h.rect[2] > 0.0
                    && h.rect[3] > 0.0));
                let clip = [5.0, 6.0, 380.0, 260.0];
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
                assert!(
                    paint
                        .clipped_quads
                        .iter()
                        .any(|q| q.quad.color == rgba(theme.colors.primary))
                );
                assert!(
                    paint
                        .clipped_quads
                        .iter()
                        .any(|q| q.quad.color == rgba(theme.colors.secondary))
                );
                assert!(paint.text.iter().all(|t| t.clip.is_some()));
            }
        }
    }
}
