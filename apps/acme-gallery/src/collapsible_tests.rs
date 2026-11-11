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
fn collapsible_pointer_toggles_once_and_disabled_header_rejects_stale_actions() {
    let mut g = Gallery::new();
    g.selected_category = 1;
    frame(&mut g);
    assert!(!g.collapsible_open);
    click(&mut g, GalleryMessage::CollapsibleToggle);
    assert!(g.collapsible_open);
    assert_eq!(g.collapsible_toggles, 1);
    let header = g
        .button_info
        .iter()
        .position(|h| h.message == GalleryMessage::CollapsibleToggle)
        .unwrap();
    let disable = g
        .button_info
        .iter()
        .position(|h| h.message == GalleryMessage::CollapsibleDisable)
        .unwrap();
    assert!(g.activate(disable));
    assert!(
        !g.activate(header),
        "stale header cannot toggle while disabled"
    );
    frame(&mut g);
    assert!(g.collapsible_open && g.collapsible_disabled);
    assert_eq!(g.collapsible_toggles, 1);
    assert!(
        !g.button_info
            .iter()
            .any(|h| h.message == GalleryMessage::CollapsibleToggle)
    );
    click(&mut g, GalleryMessage::CollapsibleDisable);
    click(&mut g, GalleryMessage::CollapsibleToggle);
    assert!(!g.collapsible_open && !g.collapsible_disabled);
    assert_eq!(g.collapsible_toggles, 2);
}

#[test]
fn collapsible_demo_layout_paint_and_hit_slots_follow_enabled_state_theme_and_scale() {
    let mut g = Gallery::new();
    for dark in [false, true] {
        g.dark = dark;
        let theme = g.current_theme();
        for scale in [1.0, 1.25, 2.0] {
            for (open, disabled) in [(false, false), (true, false), (true, true), (false, true)] {
                g.collapsible_open = open;
                g.collapsible_disabled = disabled;
                let widget = g.collapsible_examples();
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
                assert_eq!(hits.len(), if disabled { 1 } else { 2 });
                assert_eq!(
                    hits.last().unwrap().message,
                    GalleryMessage::CollapsibleDisable
                );
                if !disabled {
                    assert_eq!(hits[0].message, GalleryMessage::CollapsibleToggle);
                }
                let mut bottom = 0.0;
                for hit in &hits {
                    let r = hit.rect;
                    assert!(r.iter().all(|n| n.is_finite()));
                    assert!(
                        r[0] >= 0.0
                            && r[0] + r[2] <= 400.01
                            && r[2] > 0.0
                            && r[3] > 0.0
                            && r[1] >= bottom
                    );
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
                assert!(paint.text.iter().all(|t| t.clip.is_some()));
                assert!(
                    paint
                        .text
                        .iter()
                        .any(|t| t.color == rgba(theme.colors.foreground))
                );
            }
        }
    }
}
