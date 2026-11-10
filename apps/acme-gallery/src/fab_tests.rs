use super::*;
#[test]
fn fab_examples_paint_live_shapes_elevation_clips_and_correct_action_indices() {
    let mut g = Gallery::new();
    for pack in ["default", "apple"] {
        g.theme_pack = pack.into();
        for dark in [false, true] {
            g.dark = dark;
            let theme = g.current_theme();
            for scale in [1.0, 1.25, 2.0] {
                let widget = g.fab_examples();
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
                    .compute_with_text(&layout, (600.0, 500.0), &mut g.fonts, scale)
                    .unwrap();
                let mut hits = Vec::new();
                collect_hit_regions(&widget, &layout, &snapshot, true, &mut hits);
                assert_eq!(
                    hits.iter().map(|h| h.message).collect::<Vec<_>>(),
                    (0..4).map(GalleryMessage::FabPress).collect::<Vec<_>>()
                );
                let clip = [5.0, 8.0, 580.0, 400.0];
                let mut frame = Frame::default();
                let mut index = 0;
                render_content(
                    &mut frame,
                    &widget,
                    &layout,
                    &snapshot,
                    &theme,
                    scale,
                    7.0,
                    clip,
                    &mut index,
                    Some(1),
                    Some(2),
                    &mut g.fonts,
                    &mut g.atlas,
                );
                assert_eq!(index, hits.len());
                assert!(frame.clipped_quads.iter().all(|q| q.clip == clip));
                let buttons: Vec<_> = frame
                    .clipped_quads
                    .iter()
                    .filter(|q| {
                        [
                            rgba(theme.colors.primary),
                            rgba(theme.colors.primary_hover),
                            rgba(theme.colors.primary_pressed),
                        ]
                        .contains(&q.quad.color)
                    })
                    .collect();
                assert_eq!(buttons.len(), 5);
                for (i, (button, radius)) in buttons
                    .iter()
                    .zip([20.0, 28.0, 28.0, 0.0, 28.0])
                    .enumerate()
                {
                    assert_eq!(button.quad.radius, radius);
                    assert_eq!(button.quad.border_width, 0.0);
                    assert_eq!(
                        button.quad.color,
                        rgba(match i {
                            1 => theme.colors.primary_hover,
                            2 => theme.colors.primary_pressed,
                            _ => theme.colors.primary,
                        })
                    );
                }
                let mut shadow_color = theme.shadows.sm.color;
                shadow_color.alpha /= 4.0;
                assert_eq!(
                    frame
                        .clipped_quads
                        .iter()
                        .filter(|q| q.quad.color == rgba(shadow_color))
                        .count(),
                    20
                );
                // Five button labels precede the ordinary status label.
                for (text, button) in frame.text.iter().take(5).zip(buttons) {
                    assert_eq!(text.color, rgba(theme.colors.primary_foreground));
                    assert_eq!(text.clip, Some(intersect_clip(button.quad.rect, clip)));
                }
            }
        }
    }
}
#[test]
fn gallery_fab_pointer_press_release_and_cancel_dispatch_once() {
    let mut g = Gallery::new();
    g.frame(FrameContext {
        window: WindowId(1),
        logical_width: 1100.0,
        logical_height: 1100.0,
        scale_factor: 1.0,
    });
    let rect = g
        .button_info
        .iter()
        .find(|h| h.message == GalleryMessage::FabPress(2))
        .unwrap()
        .rect;
    let (x, y) = (rect[0] + rect[2] / 2.0, rect[1] + rect[3] / 2.0);
    g.event(PlatformEvent::PointerMoved {
        window: WindowId(1),
        x,
        y,
    });
    g.event(PlatformEvent::PointerButton {
        window: WindowId(1),
        pressed: true,
        x,
        y,
        button: 0,
        pointer: 0,
        shift: false,
        ctrl: false,
        alt: false,
    });
    assert_eq!(g.fab_status, "none");
    g.event(PlatformEvent::PointerButton {
        window: WindowId(1),
        pressed: false,
        x,
        y,
        button: 0,
        pointer: 0,
        shift: false,
        ctrl: false,
        alt: false,
    });
    assert_eq!(g.fab_status, "create");
    g.fab_status = "none";
    g.event(PlatformEvent::PointerButton {
        window: WindowId(1),
        pressed: true,
        x,
        y,
        button: 0,
        pointer: 0,
        shift: false,
        ctrl: false,
        alt: false,
    });
    g.event(PlatformEvent::PointerMoved {
        window: WindowId(1),
        x: 1099.0,
        y: 1099.0,
    });
    g.event(PlatformEvent::PointerButton {
        window: WindowId(1),
        pressed: false,
        x,
        y,
        button: 0,
        pointer: 0,
        shift: false,
        ctrl: false,
        alt: false,
    });
    assert_eq!(g.fab_status, "none");
}

#[test]
fn passive_buttons_never_acquire_hover_or_press_from_absent_hit_indices() {
    let passive = acme_widgets::button::<GalleryMessage>("passive", "Info");
    let mut index = 7;
    for (hovered, pressed) in [(None, None), (Some(7), None), (None, Some(7))] {
        let state = button_state_for_render(&passive, &mut index, hovered, pressed);
        assert!(!state.hovered && !state.pressed);
        assert_eq!(index, 7);
    }
}

// Empty intersections draw no pixels; nonempty clips must stay in the viewport.
pub(super) fn clip_within(inner: Option<[f32; 4]>, outer: [f32; 4]) -> bool {
    inner.is_some_and(|r| {
        r[2] == 0.0
            || r[3] == 0.0
            || (r[0] >= outer[0]
                && r[1] >= outer[1]
                && r[0] + r[2] <= outer[0] + outer[2]
                && r[1] + r[3] <= outer[1] + outer[3])
    })
}
