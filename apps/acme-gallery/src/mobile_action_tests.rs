use super::*;
fn frame(g: &mut Gallery) {
    g.frame(FrameContext {
        window: WindowId(1),
        logical_width: 1100.0,
        logical_height: 1100.0,
        scale_factor: 1.0,
    });
}
#[test]
fn pointer_actions_dispatch_and_disabled_rows_have_no_hit() {
    let mut g = Gallery::new();
    frame(&mut g);
    assert_eq!(
        g.button_info
            .iter()
            .filter(|h| matches!(h.message, GalleryMessage::MobileActionPress(_)))
            .count(),
        2
    );
    for destructive in [false, true] {
        let message = GalleryMessage::MobileActionPress(destructive);
        let r = g
            .button_info
            .iter()
            .find(|h| h.message == message)
            .unwrap()
            .rect;
        let (x, y) = (r[0] + r[2] / 2.0, r[1] + r[3] / 2.0);
        g.event(PlatformEvent::PointerMoved {
            window: WindowId(1),
            x,
            y,
        });
        for pressed in [true, false] {
            g.event(PlatformEvent::PointerButton {
                window: WindowId(1),
                pressed,
                x,
                y,
                button: 0,
                pointer: 0,
                shift: false,
                ctrl: false,
                alt: false,
            });
        }
        assert_eq!(
            g.action_status,
            if destructive {
                "delete requested"
            } else {
                "share requested"
            }
        );
    }
    let widget = g.action_examples();
    let layout = widget.to_layout_with_context(
        NodeId::new(1),
        &WidgetLayoutContext {
            body_font_size: 14.0,
            body_line_height: 20.0,
            label_font_size: 13.0,
            control_height: 36.0,
            scale_factor: 1.0,
        },
    );
    let snapshot = g.layout.compute(&layout, (600.0, 1000.0)).unwrap();
    let r = snapshot.get(layout.children[2].id).unwrap();
    let mut hits = Vec::new();
    collect_hit_regions(&widget, &layout, &snapshot, true, &mut hits);
    assert!(hits.iter().all(|h| h.rect[1] != r.y));
    let WidgetNode::Row(disabled) = &widget.children()[2] else {
        panic!("Row")
    };
    assert!(disabled.message.is_none());
}
#[test]
fn action_paint_indices_semantic_label_colours_and_clips_follow_live_theme() {
    use acme_widgets::style::ColorToken;
    let mut g = Gallery::new();
    for pack in ["default", "apple"] {
        g.theme_pack = pack.into();
        for dark in [false, true] {
            g.dark = dark;
            let theme = g.current_theme();
            for scale in [1.0, 1.25, 2.0] {
                let widget = column()
                    .width(320.0)
                    .child(g.action_examples())
                    .child(button("after", "After").on_click(GalleryMessage::DpiInfo))
                    .build();
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
                    .compute_with_text(&layout, (320.0, 2000.0), &mut g.fonts, scale)
                    .unwrap();
                let mut hits = Vec::new();
                collect_hit_regions(&widget, &layout, &snapshot, true, &mut hits);
                assert_eq!(hits.len(), 3);
                let mut f = Frame::default();
                let mut index = 0;
                let clip = [0.0, 40.0, 320.0, 900.0];
                render_content(
                    &mut f,
                    &widget,
                    &layout,
                    &snapshot,
                    &theme,
                    scale,
                    5.0,
                    clip,
                    &mut index,
                    Some(2),
                    Some(1),
                    &mut g.fonts,
                    &mut g.atlas,
                );
                assert_eq!(index, 3);
                assert!(f.clipped_quads.iter().all(|q| q.clip == clip));
                assert_eq!(
                    f.clipped_quads[2].quad.color,
                    rgba(ColorToken::DisabledBg.resolve(&theme))
                );
                // Text order: share icon/label, delete icon/label, disabled label.
                assert_eq!(f.text[2].color, rgba(theme.colors.danger));
                assert_eq!(f.text[3].color, rgba(theme.colors.danger));
                assert_eq!(f.text[4].color, rgba(theme.colors.disabled_text));
                assert!(f.text.iter().all(|t| t.clip == Some(clip)));
            }
        }
    }
}
