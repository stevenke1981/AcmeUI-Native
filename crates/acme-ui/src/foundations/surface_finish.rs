//! Optional metallic and glass finishes for Canvas and custom painting.

use acme_core::{ImageFit, ImageKey, Logical, Painter, Rect, Rgba8Image};
use acme_theme::{Theme, ThemeColor};
use acme_widgets::painter_bridge::theme_color;
use acme_widgets::{CanvasBuilder, canvas};
use std::hash::{DefaultHasher, Hash, Hasher};

const GRADIENT_SAMPLES: u32 = 512;

/// Theme-derived decorative finishes shared by the CPU and GPU renderers.
///
/// Keep readable content on an opaque theme surface beside the finish, or
/// use an opaque text backing if content overlays it. Select [Self::Solid]
/// when the application enables high contrast or reduced transparency.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SurfaceFinish {
    /// Opaque metallic face with alternating bands of shade and specular light.
    Metallic,
    /// Translucent themed tint with a highlight that fades down the face.
    Glass,
    /// Opaque theme surface for high contrast and reduced transparency.
    Solid,
}

impl SurfaceFinish {
    /// Build a passive Canvas swatch. Add a visible semantic label if the
    /// finish conveys information. Set dimensions with the Canvas builder.
    ///
    ///     use acme_ui::foundations::SurfaceFinish;
    ///
    ///     let swatch = SurfaceFinish::Metallic
    ///         .swatch::<()>()
    ///         .key("metallic-sample")
    ///         .size(144.0, 64.0)
    ///         .build();
    pub fn swatch<M>(self) -> CanvasBuilder<M> {
        canvas().paint(move |painter, rect, theme| self.paint(painter, rect, theme))
    }

    /// Paint the finish within a rectangle in the active painter coordinate
    /// space. Glass lets previously painted content show through its highlight.
    pub fn paint(self, painter: &mut Painter<'_>, rect: Rect<Logical>, theme: &Theme) {
        let (dx, dy) = painter.current_offset();
        let x = rect.origin.x.get() + dx;
        let y = rect.origin.y.get() + dy;
        let width = rect.size.width.get();
        let height = rect.size.height.get();
        if ![x, y, x + width, y + height]
            .into_iter()
            .all(f32::is_finite)
            || !painter.is_visible(rect)
        {
            return;
        }

        let colors = &theme.colors;
        let surface = colors.surface.with_alpha(1.0);
        let stops: &[(f32, ThemeColor)] = match self {
            Self::Metallic => {
                let steel = surface.lerp(colors.primary.with_alpha(1.0), 0.15);
                &[
                    (0.0, steel.lighten(0.33)),
                    (0.24, steel.darken(0.22)),
                    (0.43, steel.lighten(0.68)),
                    (0.56, steel.darken(0.30)),
                    (0.84, steel.lighten(0.14)),
                    (1.0, steel.darken(0.14)),
                ]
            }
            Self::Glass => {
                let tint = surface.lerp(colors.primary.with_alpha(1.0), 0.28);
                &[
                    (0.0, tint.lighten(0.68).with_alpha(0.80)),
                    (0.16, tint.lighten(0.52).with_alpha(0.58)),
                    (0.43, tint.lighten(0.33).with_alpha(0.38)),
                    (0.46, tint.with_alpha(0.24)),
                    (0.88, tint.with_alpha(0.34)),
                    (1.0, tint.lighten(0.38).with_alpha(0.56)),
                ]
            }
            Self::Solid => {
                painter.fill_rect(rect, theme_color(surface));
                return;
            }
        };

        // A single 1-D texture avoids seams between adjacent quads at fractional DPI.
        let mut pixels = Vec::with_capacity(GRADIENT_SAMPLES as usize * 4);
        let mut stop = 0;
        for sample in 0..GRADIENT_SAMPLES {
            let t = sample as f32 / (GRADIENT_SAMPLES - 1) as f32;
            while t > stops[stop + 1].0 {
                stop += 1;
            }
            let (start, color) = stops[stop];
            let (end, next_color) = stops[stop + 1];
            let u = (t - start) / (end - start);
            let smooth = u * u * (3.0 - 2.0 * u);
            let color = color.lerp(next_color, smooth);
            pixels.extend(
                [color.red, color.green, color.blue, color.alpha]
                    .map(|channel| (channel.clamp(0.0, 1.0) * 255.0).round() as u8),
            );
        }

        // A content-based revision lets different themes coexist in one scene
        // and reuses unchanged uploads.
        let mut hash = DefaultHasher::new();
        pixels.hash(&mut hash);
        let image = Rgba8Image::new(
            ImageKey::new(u64::from_be_bytes(*b"acmeMtrl")),
            hash.finish(),
            1,
            GRADIENT_SAMPLES,
            pixels,
        )
        .expect("one RGBA8 texel per gradient sample");
        painter.image_fit(rect, image, ImageFit::Fill);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use acme_core::DrawCommand;
    use acme_theme::contrast_ratio;
    use acme_theme::packs::{ThemePack, high_contrast::HighContrastPack};

    #[test]
    fn metallic_has_opaque_highlights_and_shadows() {
        let mut painter = Painter::new();
        let rect = Rect::new(3.5, 5.0, 120.0, 28.0);
        SurfaceFinish::Metallic.paint(&mut painter, rect, &Theme::dark());
        let [DrawCommand::Image(image)] = painter.scene().commands() else {
            panic!("expected one gradient image");
        };
        assert_eq!(image.rect, rect);
        assert_eq!(image.fit, ImageFit::Fill);
        assert_eq!(image.image.width(), 1);
        assert_eq!(image.image.height(), GRADIENT_SAMPLES);
        let pixels = image.image.pixels();
        let red_at = |offset: usize| pixels[offset * 4];
        assert!(red_at(0) > red_at(123));
        assert!(red_at(220) > red_at(0));
        assert!(red_at(220) > red_at(286));
        assert!(red_at(511) < red_at(430));
        assert!(pixels.chunks_exact(4).all(|pixel| pixel[3] == 255));
    }

    #[test]
    fn glass_fades_to_a_translucent_tint_and_tracks_theme_changes() {
        let mut painter = Painter::new();
        let rect = Rect::new(0.0, 0.0, 160.0, 64.0);
        for theme in [Theme::light(), Theme::dark(), Theme::light()] {
            SurfaceFinish::Glass.paint(&mut painter, rect, &theme);
        }
        let [
            DrawCommand::Image(light),
            DrawCommand::Image(dark),
            DrawCommand::Image(repeat),
        ] = painter.scene().commands()
        else {
            panic!("expected three glass draws");
        };
        assert_ne!(light.image.revision(), dark.image.revision());
        assert_eq!(light.image, repeat.image);
        let pixels = dark.image.pixels();
        assert_eq!(pixels[3], 204);
        assert_eq!(pixels[511 * 4 + 3], 143);
        assert!(pixels[3] > pixels[236 * 4 + 3]);
        assert!(pixels[3] > pixels[511 * 4 + 3]);
        assert!(pixels[0] > pixels[236 * 4]);
        assert!(
            pixels
                .chunks_exact(4)
                .all(|pixel| (1..255).contains(&pixel[3]))
        );
    }

    #[test]
    fn solid_fallback_uses_opaque_high_contrast_theme_surface() {
        let rect = Rect::new(0.0, 0.0, 120.0, 48.0);
        for theme in [HighContrastPack.light(), HighContrastPack.dark()] {
            let mut painter = Painter::new();
            SurfaceFinish::Solid.paint(&mut painter, rect, &theme);
            let [DrawCommand::Quad(quad)] = painter.scene().commands() else {
                panic!("expected a solid theme fill");
            };
            assert_eq!(quad.color, theme_color(theme.colors.surface));
            assert_eq!(quad.rect, rect);
            assert!(contrast_ratio(theme.colors.foreground, theme.colors.surface) >= 7.0);
        }
    }

    #[test]
    fn swatch_uses_canvas_layout_and_preserves_painter_clip_and_order() {
        let swatch = SurfaceFinish::Glass
            .swatch::<()>()
            .size(160.0, 48.0)
            .into_canvas();
        assert_eq!(swatch.width, Some(160.0));
        assert_eq!(swatch.height, Some(48.0));
        assert_eq!(swatch.activate(), None);

        let mut painter = Painter::new();
        let rect = Rect::new(0.0, 0.0, 160.0, 48.0);
        let theme = Theme::dark();
        let clip = Rect::new(10.0, 20.0, 120.0, 36.0);
        painter.with_offset(10.0, 20.0, |painter| {
            painter.with_clip(Rect::new(0.0, 0.0, 120.0, 36.0), |painter| {
                painter.fill_rect(rect, theme_color(theme.colors.primary));
                swatch.paint_into(painter, rect, &theme);
                swatch.paint_into(painter, Rect::new(200.0, 0.0, 20.0, 20.0), &theme);
                swatch.paint_into(painter, Rect::new(0.0, 0.0, 0.0, 20.0), &theme);
                swatch.paint_into(painter, Rect::new(f32::MAX, 0.0, f32::MAX, 20.0), &theme);
            });
        });
        let scene = painter.finish_checked().unwrap();
        let [
            DrawCommand::PushClip(clipped),
            DrawCommand::Quad(_),
            DrawCommand::Image(image),
            DrawCommand::PopClip,
        ] = scene.commands()
        else {
            panic!("expected the glass above the original quad within the original clip");
        };
        assert_eq!(*clipped, clip);
        assert_eq!(image.rect, Rect::new(10.0, 20.0, 160.0, 48.0));
    }
}
