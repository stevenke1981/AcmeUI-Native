use super::*;

fn frame(g: &mut Gallery) {
    g.frame(FrameContext {
        window: WindowId(1),
        logical_width: 1100.0,
        logical_height: 900.0,
        scale_factor: 1.0,
    });
}
fn click(g: &mut Gallery, message: GalleryMessage) {
    let r = g
        .button_info
        .iter()
        .find(|h| h.message == message)
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
            alt: false
        }));
    }
    frame(g);
}

#[test]
fn fullscreen_close_dispatches_once_and_host_can_restore_it() {
    let mut g = Gallery::new();
    g.selected_category = 4;
    frame(&mut g);
    assert!(g.fullscreen_visible);
    assert_eq!(g.fullscreen_closes, 0);
    click(&mut g, GalleryMessage::FullscreenClose);
    assert!(!g.fullscreen_visible);
    assert_eq!(g.fullscreen_closes, 1);
    assert!(
        !g.button_info
            .iter()
            .any(|h| h.message == GalleryMessage::FullscreenClose)
    );
    click(&mut g, GalleryMessage::FullscreenShow);
    assert!(g.fullscreen_visible);
    assert_eq!(g.fullscreen_closes, 1);
    let stale = g
        .button_info
        .iter()
        .position(|h| h.message == GalleryMessage::FullscreenClose)
        .unwrap();
    assert!(g.activate(stale));
    assert!(!g.activate(stale));
    assert_eq!(g.fullscreen_closes, 2);
}

#[test]
fn fullscreen_demo_hit_and_paint_slots_match_theme_scale_and_visibility() {
    let mut g = Gallery::new();
    for dark in [false, true] {
        g.dark = dark;
        let theme = g.current_theme();
        for scale in [1.0, 1.25, 2.0] {
            for visible in [false, true] {
                g.fullscreen_visible = visible;
                let widget = g.fullscreen_examples();
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
                    .compute_with_text(&layout, (600.0, 600.0), &mut g.fonts, scale)
                    .unwrap();
                let mut hits = Vec::new();
                collect_hit_regions(&widget, &layout, &snapshot, true, &mut hits);
                assert_eq!(hits.len(), 1);
                let rect = hits[0].rect;
                assert!(rect[0] >= 0.0 && rect[0] + rect[2] <= 600.01);
                assert!(rect[2] > 0.0 && rect[3] > 0.0);
                assert_eq!(
                    hits[0].message,
                    if visible {
                        GalleryMessage::FullscreenClose
                    } else {
                        GalleryMessage::FullscreenShow
                    }
                );
                let clip = [5.0, 6.0, 580.0, 560.0];
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
                assert!(paint.text.iter().all(|t| t.clip.is_some()));
            }
        }
    }
}
