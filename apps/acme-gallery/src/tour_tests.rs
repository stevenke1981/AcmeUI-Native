use super::*;

#[test]
fn tour_scroll_does_not_shrink_the_gallery_toolbar() {
    let mut g = Gallery::new();
    g.selected_category = 4;
    for height in [760.0, 900.0] {
        for finished in [false, true] {
            g.tour.finished = finished;
            g.scroll = f32::MAX;
            g.frame(FrameContext {
                window: WindowId(1),
                logical_width: 1100.0,
                logical_height: height,
                scale_factor: 1.0,
            });
            let (root, snapshot) = g.last_frame_layout.as_ref().unwrap();
            let ids = extract_gallery_ids(root);
            let toolbar = snapshot.get(ids.toolbar).unwrap();
            let viewport = snapshot.get(ids.scroll_view).unwrap();
            assert_eq!(toolbar.y, 0.0);
            assert_eq!(
                toolbar.height, TOOLBAR_HEIGHT,
                "long Tour page must retain toolbar controls"
            );
            assert_eq!(viewport.y, TOOLBAR_HEIGHT);
            assert!((viewport.height - (height - TOOLBAR_HEIGHT)).abs() < 0.01);
        }
    }
}

fn frame(g: &mut Gallery) {
    g.frame(FrameContext {
        window: WindowId(1),
        logical_width: 1100.0,
        logical_height: 760.0,
        scale_factor: 1.0,
    });
}

fn click(g: &mut Gallery, message: GalleryMessage) {
    let hit = g.button_info.iter().find(|h| h.message == message).unwrap();
    let center_y = hit.rect[1] + hit.rect[3] / 2.0;
    g.scroll = (center_y - 500.0).clamp(0.0, g.max_scroll);
    frame(g);
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
    frame(g);
}

#[test]
fn gallery_tour_pointer_previous_next_finish_restart_and_stale_messages_are_controlled() {
    let mut g = Gallery::new();
    g.selected_category = 4;
    frame(&mut g);
    assert!(!g.tour.apply(GalleryMessage::TourPrevious));
    assert!(!g.tour.apply(GalleryMessage::TourFinish));
    click(&mut g, GalleryMessage::TourNext);
    assert_eq!(g.tour.current, 1);
    click(&mut g, GalleryMessage::TourPrevious);
    assert_eq!(g.tour.current, 0);
    click(&mut g, GalleryMessage::TourNext);
    click(&mut g, GalleryMessage::TourNext);
    assert_eq!(g.tour.current, 2);
    assert!(!g.tour.apply(GalleryMessage::TourNext));
    click(&mut g, GalleryMessage::TourFinish);
    assert!(g.tour.finished);
    for stale in [
        GalleryMessage::TourPrevious,
        GalleryMessage::TourNext,
        GalleryMessage::TourFinish,
    ] {
        assert!(!g.tour.apply(stale));
        assert!(!g.button_info.iter().any(|h| h.message == stale));
    }
    click(&mut g, GalleryMessage::TourRestart);
    assert!(!g.tour.finished);
    assert_eq!(g.tour.current, 0);
    assert!(!g.tour.apply(GalleryMessage::TourRestart));
    assert!(g.pressed.is_none());
}

#[test]
fn tour_gallery_keeps_themed_clipped_paint_and_hit_slots_for_every_stage() {
    let mut g = Gallery::new();
    for dark in [false, true] {
        g.dark = dark;
        let theme = g.current_theme();
        for scale in [1.0, 1.25, 2.0] {
            for stage in 0..4 {
                g.tour.current = stage.min(2);
                g.tour.finished = stage == 3;
                let widget = g.tour_examples();
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
                    .compute_with_text(&layout, (500.0, 300.0), &mut g.fonts, scale)
                    .unwrap();
                let mut hits = Vec::new();
                collect_hit_regions(&widget, &layout, &snapshot, true, &mut hits);
                let expected = match stage {
                    0 => vec![GalleryMessage::TourNext],
                    1 => vec![GalleryMessage::TourPrevious, GalleryMessage::TourNext],
                    2 => vec![GalleryMessage::TourPrevious, GalleryMessage::TourFinish],
                    _ => vec![GalleryMessage::TourRestart],
                };
                assert_eq!(hits.iter().map(|h| h.message).collect::<Vec<_>>(), expected);
                let clip = [5.0, 6.0, 480.0, 260.0];
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
                if stage != 3 {
                    assert!(
                        paint
                            .clipped_quads
                            .iter()
                            .any(|q| q.quad.color == rgba(theme.colors.primary))
                    );
                }
                assert!(paint.text.iter().all(|t| t.clip.is_some()));
            }
        }
    }
}
