use super::*;

fn frame(g: &mut Gallery) {
    g.frame(FrameContext {
        window: WindowId(1),
        logical_width: 1100.0,
        logical_height: 1000.0,
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
fn context_menu_item_dispatches_once_and_host_can_restore_it() {
    let mut g = Gallery::new();
    g.selected_category = 4; // Overlay category
    frame(&mut g);
    assert!(g.context_menu_open);
    assert_eq!(g.context_menu_cuts, 0);
    assert_eq!(g.context_menu_copies, 0);
    click(&mut g, GalleryMessage::ContextMenuCut);
    assert!(!g.context_menu_open);
    assert_eq!(g.context_menu_cuts, 1);
    assert_eq!(g.context_menu_copies, 0);
    assert!(
        !g.button_info
            .iter()
            .any(|h| h.message == GalleryMessage::ContextMenuCut
                || h.message == GalleryMessage::ContextMenuCopy)
    );
    click(&mut g, GalleryMessage::ContextMenuShow);
    assert!(g.context_menu_open);
    assert_eq!(g.context_menu_cuts, 1);
    click(&mut g, GalleryMessage::ContextMenuCopy);
    assert!(!g.context_menu_open);
    assert_eq!(g.context_menu_cuts, 1);
    assert_eq!(g.context_menu_copies, 1);

    // Stale click after close should be rejected
    click(&mut g, GalleryMessage::ContextMenuShow);
    let stale = g
        .button_info
        .iter()
        .position(|h| h.message == GalleryMessage::ContextMenuCut)
        .unwrap();
    assert!(g.activate(stale));
    assert!(!g.activate(stale));
    assert_eq!(g.context_menu_cuts, 2);
    assert!(!g.context_menu_open);
}

#[test]
fn context_menu_demo_hit_and_paint_slots_match_theme_scale_and_visibility() {
    let mut g = Gallery::new();
    for dark in [false, true] {
        g.dark = dark;
        let theme = g.current_theme();
        for scale in [1.0, 1.25, 2.0] {
            for open in [false, true] {
                g.context_menu_open = open;
                let widget = g.context_menu_examples();
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
                if open {
                    assert_eq!(hits.len(), 2);
                    let messages: Vec<_> = hits.iter().map(|h| h.message).collect();
                    assert!(messages.contains(&GalleryMessage::ContextMenuCut));
                    assert!(messages.contains(&GalleryMessage::ContextMenuCopy));
                    for hit in &hits {
                        assert!(
                            hit.rect[2] > 0.0 && hit.rect[3] > 0.0,
                            "hit rect must be positive: {:?}",
                            hit.rect
                        );
                    }
                    assert!(
                        !(hits[0].rect[0] < hits[1].rect[0] + hits[1].rect[2]
                            && hits[1].rect[0] < hits[0].rect[0] + hits[0].rect[2]
                            && hits[0].rect[1] < hits[1].rect[1] + hits[1].rect[3]
                            && hits[1].rect[1] < hits[0].rect[1] + hits[0].rect[3]),
                        "Cut and Copy hit rects must not overlap"
                    );
                } else {
                    assert_eq!(hits.len(), 1);
                    assert_eq!(hits[0].message, GalleryMessage::ContextMenuShow);
                }
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
