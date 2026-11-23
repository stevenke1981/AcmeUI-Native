//! Text rendering helpers — shape, prepare, and emit text runs via [`Painter`].

use acme_core::{AtlasUpload, Color, GlyphDraw, GlyphFormat, Painter, Point, Rect, TextPrimitive};
use acme_text::{FontSystem, GlyphAtlas, TextConstraints, TextStyle};
use acme_theme::ThemeColor;

/// Shape text and emit it as a [`TextPrimitive`] through [`Painter`].
#[allow(clippy::too_many_arguments)]
pub fn add_text(
    fonts: &mut FontSystem,
    atlas: &mut GlyphAtlas,
    painter: &mut Painter<'_>,
    text: &str,
    geometry: ([f32; 2], f32),
    color: ThemeColor,
    scale: f32,
    clip: Option<[f32; 4]>,
    line_height_ratio: f32,
) {
    let (origin, size) = geometry;
    let style = TextStyle {
        font_size: size,
        line_height: size * line_height_ratio,
        ..TextStyle::default()
    };
    let layout = fonts.shape(text, &style, TextConstraints::default(), scale);
    add_shaped_text(fonts, atlas, painter, &layout, origin, color, clip);
}

pub(super) fn add_shaped_text(
    fonts: &mut FontSystem,
    atlas: &mut GlyphAtlas,
    painter: &mut Painter<'_>,
    layout: &acme_text::TextLayout,
    origin: [f32; 2],
    color: ThemeColor,
    clip: Option<[f32; 4]>,
) {
    let prepared = fonts.prepare(layout, atlas);

    // Convert prepared text to scene types
    let mut glyphs = Vec::new();
    let mut uploads = Vec::new();
    for region in &prepared.uploads {
        let pfmt = match region.format {
            acme_text::AtlasFormat::Alpha8 => GlyphFormat::Alpha8,
            acme_text::AtlasFormat::Rgba8 => GlyphFormat::Rgba8,
        };
        uploads.push(AtlasUpload {
            page: region.page,
            origin: [region.x, region.y],
            size: [region.width, region.height],
            format: pfmt,
            pixels: region.pixels.clone(),
        });
    }
    for glyph in &prepared.glyphs {
        let gfmt = match glyph.format {
            acme_text::AtlasFormat::Alpha8 => GlyphFormat::Alpha8,
            acme_text::AtlasFormat::Rgba8 => GlyphFormat::Rgba8,
        };
        glyphs.push(GlyphDraw {
            x: glyph.x as f32,
            y: glyph.y as f32,
            width: glyph.width as f32,
            height: glyph.height as f32,
            atlas_x: glyph.atlas_x,
            atlas_y: glyph.atlas_y,
            atlas_page: glyph.atlas_page,
            format: gfmt,
        });
    }

    let primitive = TextPrimitive {
        origin: Point::new(origin[0], origin[1]),
        color: Color::rgba(color.red, color.green, color.blue, color.alpha),
        glyphs,
        uploads,
    };

    if let Some(clip_rect) = clip {
        painter.with_clip(
            Rect::new(clip_rect[0], clip_rect[1], clip_rect[2], clip_rect[3]),
            |painter| {
                painter.text_primitive(primitive);
            },
        );
    } else {
        painter.text_primitive(primitive);
    }
}
