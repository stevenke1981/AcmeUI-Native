use super::*;
use acme_ui::layout::{PaperElevation, paper};

impl Gallery {
    pub(super) fn paper_examples(&self) -> WidgetNode<GalleryMessage> {
        let mut examples = row().key("paper_examples").gap(12.0);
        for (key, title, square, outlined) in [
            ("paper_rounded", "Rounded", false, false),
            ("paper_square", "Square", true, false),
            ("paper_outlined", "Square outlined", true, true),
        ] {
            examples = examples.child(
                column()
                    .width(200.0)
                    .child(
                        paper()
                            .key(key)
                            .elevation(PaperElevation::E8)
                            .square(square)
                            .outlined(outlined)
                            .child(label(title)),
                    )
                    .build(),
            );
        }
        examples.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paper_examples_paint_the_requested_corners_with_live_theme_and_clip() {
        let mut gallery = Gallery::new();
        for pack in ["default", "apple"] {
            gallery.theme_pack = pack.into();
            for dark in [false, true] {
                gallery.dark = dark;
                let theme = gallery.current_theme();
                for scale in [1.0, 1.25, 2.0] {
                    let widget = gallery.paper_examples();
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
                    let surfaces: Vec<_> = frame
                        .clipped_quads
                        .iter()
                        .filter(|q| q.quad.border_width == 1.0)
                        .collect();
                    assert_eq!(surfaces.len(), 3);
                    assert!(frame.clipped_quads.iter().all(|q| q.clip == clip));
                    for (i, quad) in surfaces.iter().enumerate() {
                        assert_eq!(quad.clip, clip);
                        assert_eq!(quad.quad.radius, if i == 0 { 12.0 } else { 0.0 });
                        assert_eq!(
                            quad.quad.color,
                            rgba(if i == 2 {
                                theme.colors.background
                            } else {
                                theme.colors.surface
                            })
                        );
                        assert_eq!(quad.quad.rect[2], 200.0);
                    }
                }
            }
        }
    }
}
