use acme_core::Rect;
use acme_ui::foundations::SurfaceFinish;
use acme_widgets::painter_bridge::theme_color;
use acme_widgets::style::{ColorToken, Styled};
use acme_widgets::{WidgetNode, canvas, card, label_builder, row};

use super::{Gallery, GalleryMessage};

impl Gallery {
    pub(super) fn surface_finish_examples(&self) -> WidgetNode<GalleryMessage> {
        let high_contrast = self.theme_pack == "high-contrast";
        let metallic = if high_contrast {
            SurfaceFinish::Solid
        } else {
            SurfaceFinish::Metallic
        };
        let sample = |title, swatch| {
            card().padding(10.0).gap(8.0).child(swatch).child(
                label_builder(title)
                    .text_color(ColorToken::SurfaceForeground)
                    .build(),
            )
        };
        let glass =
            canvas()
                .key("finish_glass")
                .size(136.0, 60.0)
                .paint(move |painter, bounds, theme| {
                    if high_contrast {
                        SurfaceFinish::Solid.paint(painter, bounds, theme);
                    } else {
                        painter.fill_rect(bounds, theme_color(theme.colors.surface));
                        let x = bounds.origin.x.get();
                        let y = bounds.origin.y.get();
                        let width = bounds.size.width.get();
                        let height = bounds.size.height.get();
                        // The bars stay visible through the glass and make the
                        // edge of the reflection easier to see.
                        painter.fill_rect(
                            Rect::new(x + width * 0.32, y, width * 0.24, height),
                            theme_color(theme.colors.primary),
                        );
                        painter.fill_rect(
                            Rect::new(x + width * 0.60, y, width * 0.08, height),
                            theme_color(theme.colors.primary),
                        );
                        SurfaceFinish::Glass.paint(painter, bounds, theme);
                    }
                });

        row()
            .key("surface_finishes")
            .gap(12.0)
            .child(sample(
                "Metallic gradient",
                metallic
                    .swatch()
                    .key("finish_metallic")
                    .size(136.0, 60.0)
                    .build(),
            ))
            .child(sample("Glass reflection", glass.build()))
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use acme_core::DrawCommand;
    use acme_platform::{Application, FrameContext, WindowId};
    use acme_theme::contrast_ratio;

    #[test]
    fn foundations_gallery_paints_finishes_and_high_contrast_fallback() {
        let mut gallery = Gallery::new();
        gallery.selected_category = 1;
        for dark in [false, true] {
            gallery.dark = dark;
            for pack in ["default", "high-contrast"] {
                gallery.theme_pack = pack.into();
                let theme = gallery.current_theme();
                assert!(
                    contrast_ratio(theme.colors.surface_foreground, theme.colors.surface) >= 4.5
                );
                let scene = gallery.frame(FrameContext {
                    window: WindowId(1),
                    logical_width: 1100.0,
                    logical_height: 900.0,
                    scale_factor: 1.25,
                });
                let material_images: Vec<_> = scene
                    .commands()
                    .iter()
                    .filter_map(|command| match command {
                        DrawCommand::Image(primitive)
                            if primitive.rect.size.width.get() == 136.0
                                && primitive.rect.size.height.get() == 60.0 =>
                        {
                            Some(&primitive.image)
                        }
                        _ => None,
                    })
                    .collect();
                if pack == "high-contrast" {
                    assert!(material_images.is_empty());
                    let material_fills = scene
                        .commands()
                        .iter()
                        .filter(|command| {
                            matches!(command, DrawCommand::Quad(quad)
                                if quad.rect.size.width.get() == 136.0
                                    && quad.rect.size.height.get() == 60.0
                                    && quad.color == theme_color(theme.colors.surface))
                        })
                        .count();
                    assert_eq!(material_fills, 2);
                } else {
                    assert_eq!(material_images.len(), 2);
                    assert_eq!(material_images[0].pixels()[3], 255);
                    assert!(material_images[1].pixels()[3] < 255);
                }
                scene.validate().unwrap();
            }
        }
    }
}
