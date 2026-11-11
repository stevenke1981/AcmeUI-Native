use super::*;

fn frame(g: &mut Gallery) {
    g.frame(FrameContext {
        window: WindowId(1),
        logical_width: 1100.0,
        logical_height: 900.0,
        scale_factor: 1.0,
    });
}

#[test]
fn anchor_pointer_dispatches_parent_and_child_hrefs_once_each() {
    let mut g = Gallery::new();
    g.selected_category = 3;
    frame(&mut g);
    for (count, index) in [1, 2, 0].into_iter().enumerate() {
        let r = g
            .button_info
            .iter()
            .find(|h| h.message == GalleryMessage::AnchorNavigate(index))
            .unwrap()
            .rect;
        let (x, y) = (r[0] + r[2] / 2.0, r[1] + r[3] / 2.0 - g.scroll);
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
        frame(&mut g);
        assert_eq!(g.anchor_selected, index);
        assert_eq!(g.anchor_navigations, count + 1);
    }
}

#[test]
fn anchor_demo_has_bounded_distinct_hit_slots_and_clipped_themed_paint() {
    let mut g = Gallery::new();
    for dark in [false, true] {
        g.dark = dark;
        let theme = g.current_theme();
        for scale in [1.0, 1.25, 2.0] {
            for selected in 0..3 {
                g.anchor_selected = selected;
                let widget = g.anchor_examples();
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
                    .compute_with_text(&layout, (400.0, 500.0), &mut g.fonts, scale)
                    .unwrap();
                let mut hits = Vec::new();
                collect_hit_regions(&widget, &layout, &snapshot, true, &mut hits);
                assert_eq!(hits.len(), 3);
                let mut bottom = 0.0;
                for (i, hit) in hits.iter().enumerate() {
                    assert_eq!(hit.message, GalleryMessage::AnchorNavigate(i));
                    let r = hit.rect;
                    assert!(r.iter().all(|n| n.is_finite()));
                    assert!(r[0] >= 0.0 && r[0] + r[2] <= 400.01 && r[2] > 0.0 && r[3] > 0.0);
                    assert!(r[1] >= bottom);
                    bottom = r[1] + r[3];
                }
                let clip = [5.0, 6.0, 380.0, 460.0];
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
                assert!(!paint.clipped_quads.is_empty());
                assert!(paint.clipped_quads.iter().all(|q| q.clip == clip));
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
