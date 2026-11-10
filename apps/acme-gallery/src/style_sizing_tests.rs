use super::*;

#[test]
fn full_sizing_example_fills_only_the_padded_parent_and_keeps_theme_and_clip() {
    let mut gallery = Gallery::new();
    for dark in [false, true] {
        gallery.dark = dark;
        let theme = gallery.current_theme();
        for scale in [1.0, 1.25, 2.0] {
            let widget = gallery.full_sizing_example();
            let context = WidgetLayoutContext {
                body_font_size: theme.typography.body,
                body_line_height: theme.typography.body * theme.typography.line_height,
                label_font_size: theme.typography.label,
                control_height: 36.0,
                scale_factor: scale,
            };
            let layout = widget.to_layout_with_context(NodeId::new(1), &context);
            let snapshot = gallery
                .layout
                .compute_with_text(&layout, (800.0, 600.0), &mut gallery.fonts, scale)
                .unwrap();
            let parent = snapshot.get(layout.id).unwrap();
            let child = snapshot.get(layout.children[0].id).unwrap();
            assert_eq!((parent.width, parent.height), (240.0, 96.0));
            assert_eq!(
                (child.x, child.y, child.width, child.height),
                (12.0, 12.0, 216.0, 72.0)
            );
            let mut frame = Frame::default();
            let mut index = 0;
            let clip = [0.0, 0.0, 240.0, 96.0];
            render_content(
                &mut frame,
                &widget,
                &layout,
                &snapshot,
                &theme,
                scale,
                0.0,
                clip,
                &mut index,
                None,
                None,
                &mut gallery.fonts,
                &mut gallery.atlas,
            );
            assert_eq!(index, 0);
            assert_eq!(frame.clipped_quads.len(), 2);
            assert!(frame.clipped_quads.iter().all(|q| q.clip == clip));
            assert_eq!(frame.clipped_quads[1].quad.rect, [12.0, 12.0, 216.0, 72.0]);
            assert_eq!(
                frame.clipped_quads[1].quad.color,
                rgba(theme.colors.primary)
            );
        }
    }
}
