use super::*;

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
    g.scroll = (hit.rect[1] - 160.0).clamp(0.0, g.max_scroll);
    frame(g);
    let hit = g.button_info.iter().find(|h| h.message == message).unwrap();
    let (x, y) = (
        hit.rect[0] + hit.rect[2] / 2.0,
        hit.rect[1] + hit.rect[3] / 2.0 - g.scroll,
    );
    g.event(PlatformEvent::PointerMoved {
        window: WindowId(1),
        x,
        y,
    });
    for pressed in [true, false] {
        assert!(
            g.event(PlatformEvent::PointerButton {
                window: WindowId(1),
                pressed,
                x,
                y,
                button: 0,
                pointer: 0,
                shift: false,
                ctrl: false,
                alt: false
            }),
            "{message:?}, pressed={pressed}, at ({x}, {y})"
        );
    }
    frame(g);
}

#[test]
fn gallery_speed_dial_opens_selects_independent_actions_and_rejects_closed_stale_actions() {
    let mut g = Gallery::new();
    g.selected_category = 4;
    frame(&mut g);
    assert!(!g.speed_dial.open);
    assert!(
        !g.button_info
            .iter()
            .any(|h| matches!(h.message, GalleryMessage::SpeedDialAction(_)))
    );
    click(&mut g, GalleryMessage::SpeedDialToggle);
    assert!(g.speed_dial.open);
    click(&mut g, GalleryMessage::SpeedDialAction(0));
    assert_eq!(g.speed_dial.last_action, Some(0));
    assert!(g.speed_dial.open, "host controls closing separately");
    click(&mut g, GalleryMessage::SpeedDialAction(1));
    assert_eq!(g.speed_dial.last_action, Some(1));
    let stale = g
        .button_info
        .iter()
        .position(|h| h.message == GalleryMessage::SpeedDialAction(0))
        .unwrap();
    let toggle = g
        .button_info
        .iter()
        .position(|h| h.message == GalleryMessage::SpeedDialToggle)
        .unwrap();
    assert!(g.activate(toggle));
    assert!(!g.activate(stale));
    assert_eq!(g.speed_dial.last_action, Some(1));
    frame(&mut g);
    assert!(
        !g.button_info
            .iter()
            .any(|h| matches!(h.message, GalleryMessage::SpeedDialAction(_)))
    );
}

#[test]
fn speed_dial_gallery_paint_and_hit_slots_match_across_theme_scale_and_open_state() {
    let mut g = Gallery::new();
    for dark in [false, true] {
        g.dark = dark;
        let theme = g.current_theme();
        for scale in [1.0, 1.25, 2.0] {
            for open in [false, true] {
                g.speed_dial.open = open;
                for direction in [
                    acme_ui::overlay::SpeedDialDirection::Up,
                    acme_ui::overlay::SpeedDialDirection::Down,
                    acme_ui::overlay::SpeedDialDirection::Left,
                    acme_ui::overlay::SpeedDialDirection::Right,
                ] {
                    g.speed_dial.direction = direction;
                    let widget = g.speed_dial_examples();
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
                    assert_eq!(hits.len(), if open { 4 } else { 2 });
                    for hit in &hits {
                        assert!(hit.rect.iter().all(|v| v.is_finite()));
                        assert!(hit.rect[2] > 0.0 && hit.rect[3] > 0.0);
                        assert!(hit.rect[0] >= 0.0 && hit.rect[0] + hit.rect[2] <= 400.01);
                    }
                    let clip = [0.0, 5.0, 400.0, 440.0];
                    let mut paint = Frame::default();
                    let mut index = 0;
                    render_content(
                        &mut paint,
                        &widget,
                        &layout,
                        &snapshot,
                        &theme,
                        scale,
                        7.0,
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
                    assert!(paint.text.iter().all(|t| t.clip.is_some()));
                }
            }
        }
    }
}

#[test]
fn gallery_speed_dial_direction_cycle_keeps_pointer_actions_and_clears_old_press() {
    use acme_ui::overlay::SpeedDialDirection;
    let mut g = Gallery::new();
    g.selected_category = 4;
    frame(&mut g);
    click(&mut g, GalleryMessage::SpeedDialToggle);
    for direction in [
        SpeedDialDirection::Up,
        SpeedDialDirection::Down,
        SpeedDialDirection::Left,
        SpeedDialDirection::Right,
    ] {
        assert_eq!(g.speed_dial.direction, direction);
        for index in [0, 1] {
            click(&mut g, GalleryMessage::SpeedDialAction(index));
            assert_eq!(g.speed_dial.last_action, Some(index));
            assert!(g.speed_dial.open);
        }
        g.pressed = g
            .button_info
            .iter()
            .position(|h| h.message == GalleryMessage::SpeedDialAction(0));
        g.hovered = g.pressed;
        let cycle = g
            .button_info
            .iter()
            .position(|h| h.message == GalleryMessage::SpeedDialDirection)
            .unwrap();
        assert!(g.activate(cycle));
        assert_eq!(g.pressed, None);
        assert_eq!(g.hovered, None);
        frame(&mut g);
    }
    assert_eq!(g.speed_dial.direction, SpeedDialDirection::Up);
    click(&mut g, GalleryMessage::SpeedDialToggle);
    click(&mut g, GalleryMessage::SpeedDialDirection);
    assert_eq!(g.speed_dial.direction, SpeedDialDirection::Down);
    assert!(!g.speed_dial.open);
    assert!(
        !g.button_info
            .iter()
            .any(|h| matches!(h.message, GalleryMessage::SpeedDialAction(_)))
    );
}
