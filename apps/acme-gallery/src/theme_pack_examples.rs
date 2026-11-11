use super::*;

impl Gallery {
    pub(super) fn theme_pack_swatches(&self) -> WidgetNode<GalleryMessage> {
        let mode = if self.dark {
            ThemeMode::Dark
        } else {
            ThemeMode::Light
        };
        let mut previews = column().key("theme_pack_swatches").gap(8.0).child(
            acme_widgets::label_builder(
                "Accent / surface / text / background. Use the toolbar to switch light/dark.",
            )
            .wrap(acme_widgets::LabelWrap::Character)
            .build(),
        );
        for name in available_themes() {
            let colors = theme_by_name(name, mode)
                .expect("registered theme pack")
                .colors;
            let mut palette = row().gap(8.0);
            for (token, color) in [
                ("primary", colors.primary),
                ("surface", colors.surface),
                ("foreground", colors.foreground),
                ("background", colors.background),
            ] {
                palette = palette.child(
                    card()
                        .key(format!("theme_pack_{name}_{token}"))
                        .variant(CardVariant::Outlined)
                        .w(Length::px(40.0))
                        .h(Length::px(24.0))
                        .background_color(color),
                );
            }
            previews = previews.child(
                card()
                    .key(format!("theme_pack_{name}"))
                    .variant(CardVariant::Outlined)
                    .padding(8.0)
                    .child(
                        row()
                            .gap(16.0)
                            .child(column().width(120.0).child(label(*name)))
                            .child(palette),
                    ),
            );
        }
        previews.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preview_contains_every_registered_pack_and_no_interactive_controls() {
        let gallery = Gallery::new();
        let widget = gallery.theme_pack_swatches();
        let mut names = Vec::new();
        fn labels<'a>(node: &'a WidgetNode<GalleryMessage>, out: &mut Vec<&'a str>) {
            if let WidgetNode::Label(label) = node {
                out.push(&label.text);
            }
            for child in node.children() {
                labels(child, out);
            }
        }
        labels(&widget, &mut names);
        for pack in available_themes() {
            assert!(names.contains(pack), "missing {pack}");
        }
        assert_eq!(names.len(), available_themes().len() + 1);
        fn noninteractive(node: &acme_core::ViewNode) {
            assert!(!node.focusable);
            for child in &node.children {
                noninteractive(child);
            }
        }
        for view in widget_to_views(&[widget]) {
            noninteractive(&view);
        }
    }

    #[test]
    fn palette_paint_uses_each_pack_mode_and_keeps_bounds_clips_and_retained_ids() {
        let mut gallery = Gallery::new();
        let mut saved_ids = None;
        for host in ["default", "apple"] {
            gallery.theme_pack = host.into();
            for dark in [false, true] {
                gallery.dark = dark;
                let theme = gallery.current_theme();
                let mode = if dark {
                    ThemeMode::Dark
                } else {
                    ThemeMode::Light
                };
                for scale in [1.0, 1.25, 2.0] {
                    for width in [360.0, 800.0] {
                        let widget = column()
                            .width(width)
                            .child(gallery.theme_pack_swatches())
                            .build();
                        let context = WidgetLayoutContext {
                            body_font_size: theme.typography.body,
                            body_line_height: theme.typography.body * theme.typography.line_height,
                            label_font_size: theme.typography.label,
                            control_height: 36.0,
                            scale_factor: scale,
                        };
                        let layout = retained_layout_root(&widget, &mut gallery.retained, &context);
                        let mut current_ids = Vec::new();
                        fn ids(node: &LayoutNode, out: &mut Vec<NodeId>) {
                            out.push(node.id);
                            for child in &node.children {
                                ids(child, out);
                            }
                        }
                        ids(&layout, &mut current_ids);
                        if let Some(saved) = &saved_ids {
                            assert_eq!(&current_ids, saved);
                        } else {
                            saved_ids = Some(current_ids);
                        }
                        let snapshot = gallery
                            .layout
                            .compute_with_text(&layout, (width, 1000.0), &mut gallery.fonts, scale)
                            .unwrap();
                        for card in layout.children[0].children.iter().skip(1) {
                            let outer = snapshot.get(card.id).unwrap();
                            assert!(outer.width <= width);
                            for tile in &card.children[0].children[1].children {
                                let tile = snapshot.get(tile.id).unwrap();
                                assert_eq!((tile.width, tile.height), (40.0, 24.0));
                                assert!(
                                    tile.x >= outer.x
                                        && tile.x + tile.width <= outer.x + outer.width
                                );
                                assert!(
                                    tile.y >= outer.y
                                        && tile.y + tile.height <= outer.y + outer.height
                                );
                            }
                        }
                        let mut frame = Frame::default();
                        let mut index = 0;
                        let clip = [0.0, 10.0, width, 400.0];
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
                        assert_eq!(frame.clipped_quads.len(), available_themes().len() * 5);
                        for (pack, quads) in available_themes()
                            .iter()
                            .zip(frame.clipped_quads.chunks_exact(5))
                        {
                            let sample = theme_by_name(pack, mode).unwrap().colors;
                            assert_eq!(quads[0].quad.color, rgba(theme.colors.background));
                            for (quad, color) in quads[1..].iter().zip([
                                sample.primary,
                                sample.surface,
                                sample.foreground,
                                sample.background,
                            ]) {
                                assert_eq!(quad.quad.color, rgba(color), "{pack} {mode:?}");
                            }
                            assert!(quads.iter().all(|q| q.clip == clip
                                && q.quad.border_color == rgba(theme.colors.border)));
                        }
                        assert_eq!(frame.text.len(), available_themes().len() + 1);
                        assert!(frame.text.iter().all(|text| text.clip == Some(clip)
                            && text.color == rgba(theme.colors.foreground)));
                    }
                }
            }
        }
    }
}
