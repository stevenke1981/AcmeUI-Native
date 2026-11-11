use super::*;
use acme_ui::foundations::{DataListDensity, DataListItem, DataListOrientation, data_list};

impl Gallery {
    pub(super) fn data_list_examples(&self) -> WidgetNode<GalleryMessage> {
        let mut examples = column().key("data_list_examples").gap(16.0);
        for (orientation, title) in [
            (DataListOrientation::Vertical, "Vertical items"),
            (DataListOrientation::Horizontal, "Horizontal items"),
        ] {
            let mut samples = row().gap(16.0);
            for (density, name) in [
                (DataListDensity::Compact, "Compact"),
                (DataListDensity::Default, "Default"),
                (DataListDensity::Relaxed, "Relaxed"),
            ] {
                let key = format!("data_list_{orientation:?}_{density:?}");
                samples = samples.child(
                    column().width(220.0).child(
                        card()
                            .key(key.clone())
                            .variant(CardVariant::Outlined)
                            .child(label(name))
                            .child(
                                data_list()
                                    .key(format!("{key}_content"))
                                    .orientation(orientation)
                                    .density(density)
                                    .item(DataListItem::new("Name", "陳小明"))
                                    .item(DataListItem::new("Role", "Developer")),
                            ),
                    ),
                );
            }
            examples = examples.child(label(title)).child(samples);
        }
        examples.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn density_gallery_keeps_real_extents_live_theme_and_clipped_text() {
        let mut gallery = Gallery::new();
        for pack in ["default", "apple"] {
            gallery.theme_pack = pack.into();
            for dark in [false, true] {
                gallery.dark = dark;
                let theme = gallery.current_theme();
                for scale in [1.0, 1.25, 2.0] {
                    let widget = gallery.data_list_examples();
                    let context = WidgetLayoutContext {
                        body_font_size: theme.typography.body,
                        body_line_height: theme.typography.body * theme.typography.line_height,
                        label_font_size: theme.typography.label,
                        control_height: 36.0,
                        scale_factor: scale,
                    };
                    let layout = retained_layout_root(&widget, &mut gallery.retained, &context);
                    let snapshot = gallery
                        .layout
                        .compute_with_text(&layout, (800.0, 600.0), &mut gallery.fonts, scale)
                        .unwrap();
                    for samples in [&layout.children[1], &layout.children[3]] {
                        let heights: Vec<_> = samples
                            .children
                            .iter()
                            .map(|sample| {
                                let list = &sample.children[0].children[1];
                                snapshot.get(list.id).unwrap().height
                            })
                            .collect();
                        assert!(heights[0] < heights[1] && heights[1] < heights[2]);
                    }
                    let mut frame = Frame::default();
                    let mut index = 0;
                    let clip = [0.0, 10.0, 800.0, 400.0];
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
                    assert_eq!(frame.clipped_quads.len(), 6);
                    assert!(frame.clipped_quads.iter().all(|quad| quad.clip == clip
                        && quad.quad.color == rgba(theme.colors.background)));
                    assert_eq!(frame.text.len(), 32);
                    assert!(frame.text.iter().all(|text| text.clip == Some(clip)
                        && text.color == rgba(theme.colors.foreground)));
                }
            }
        }
    }
}
