use super::*;
#[test]
fn sheet_handles_paint_live_themed_rounded_bars_without_hits_or_index_drift() {
    let mut g = Gallery::new();
    for pack in ["default", "apple"] {
        g.theme_pack = pack.into();
        for dark in [false, true] {
            g.dark = dark;
            let theme = g.current_theme();
            for scale in [1.0, 1.25, 2.0] {
                let widget = column()
                    .width(400.0)
                    .child(g.sheet_handle_examples())
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
                    .compute_with_text(&layout, (400.0, 1000.0), &mut g.fonts, scale)
                    .unwrap();
                let mut hits = Vec::new();
                collect_hit_regions(&widget, &layout, &snapshot, true, &mut hits);
                assert_eq!(hits.len(), 1);
                assert_eq!(hits[0].message, GalleryMessage::DpiInfo);
                let mut f = Frame::default();
                let mut index = 0;
                let clip = [5.0, 40.0, 380.0, 600.0];
                render_content(
                    &mut f,
                    &widget,
                    &layout,
                    &snapshot,
                    &theme,
                    scale,
                    7.0,
                    clip,
                    &mut index,
                    Some(0),
                    None,
                    &mut g.fonts,
                    &mut g.atlas,
                );
                assert_eq!(index, 1);
                assert!(f.clipped_quads.iter().all(|q| q.clip == clip));
                let bars: Vec<_> = f
                    .clipped_quads
                    .iter()
                    .filter(|q| q.quad.rect[3] == 4.0)
                    .collect();
                assert_eq!(bars.len(), 2);
                for (bar, width) in bars.iter().zip([36.0, 48.0]) {
                    assert_eq!(bar.quad.rect[2], width);
                    assert_eq!(bar.quad.radius, 2.0);
                    assert_eq!(bar.quad.color, rgba(theme.colors.muted_foreground));
                }
                // The preview's two flexible spacers center the default bar.
                let card = &layout.children[0].children[0];
                let card_rect = snapshot.get(card.id).unwrap();
                let bar = snapshot.get(card.children[0].children[1].id).unwrap();
                assert_eq!(bar.x + bar.width / 2.0, card_rect.x + card_rect.width / 2.0);
            }
        }
    }
}
