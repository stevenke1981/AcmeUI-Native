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
fn gallery_bottom_navigation_pointer_selects_and_preserves_controlled_selection() {
    let mut g = Gallery::new();
    g.selected_category = 3;
    frame(&mut g);
    assert_eq!(g.bottom_selected, 0);
    for selection in [1, 2, 2, 0] {
        let hit = g
            .button_info
            .iter()
            .find(|h| h.message == GalleryMessage::BottomNavigationSelect(selection))
            .expect("configured navigation action hit");
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
                alt: false,
            }));
        }
        frame(&mut g);
        assert_eq!(g.bottom_selected, selection);
        assert_eq!(
            g.button_info
                .iter()
                .filter(|h| matches!(h.message, GalleryMessage::BottomNavigationSelect(_)))
                .count(),
            3
        );
    }
    g.button_info.push(HitRegion {
        rect: [0.0; 4],
        message: GalleryMessage::BottomNavigationSelect(usize::MAX),
        scrolled: true,
    });
    assert!(!g.activate(g.button_info.len() - 1));
    assert_eq!(g.bottom_selected, 0);
}

#[test]
fn bottom_navigation_gallery_paint_and_hits_match_across_theme_scale_and_selection() {
    let mut g = Gallery::new();
    for dark in [false, true] {
        g.dark = dark;
        let theme = g.current_theme();
        for scale in [1.0, 1.25, 2.0] {
            for selected in 0..3 {
                g.bottom_selected = selected;
                let widget = g.bottom_navigation_examples();
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
                assert_eq!(hits.len(), 3);
                for (index, hit) in hits.iter().enumerate() {
                    assert_eq!(hit.message, GalleryMessage::BottomNavigationSelect(index));
                }
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
                        .any(|q| q.quad.color == rgba(theme.colors.secondary))
                );
                assert!(paint.text.iter().all(|t| t.clip.is_some()));
            }
        }
    }
}
