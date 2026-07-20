//! Text shaping and renderer-neutral glyph atlas preparation for AcmeUI.
#![forbid(unsafe_op_in_unsafe_fn)]

use std::collections::{HashMap, HashSet};

use cosmic_text::{
    Attrs, Buffer, CacheKey, Family, FontSystem as CosmicFontSystem, Metrics, Shaping, SwashCache,
    SwashContent, Wrap,
};

/// A renderer-neutral text style expressed in logical pixels.
#[derive(Clone, Debug, PartialEq)]
pub struct TextStyle {
    pub font_size: f32,
    pub line_height: f32,
    pub family: String,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_size: 16.0,
            line_height: 20.0,
            family: "sans-serif".into(),
        }
    }
}

/// Constraints for shaping. `max_width` is in logical pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextConstraints {
    pub max_width: Option<f32>,
    pub wrap: TextWrap,
}

impl Default for TextConstraints {
    fn default() -> Self {
        Self {
            max_width: None,
            wrap: TextWrap::WordOrGlyph,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TextWrap {
    None,
    Word,
    Glyph,
    WordOrGlyph,
}

/// One positioned glyph. Coordinates are logical pixels.
#[derive(Clone, Debug, PartialEq)]
pub struct ShapedGlyph {
    pub line: usize,
    pub byte_range: std::ops::Range<usize>,
    pub x: f32,
    pub y: f32,
    pub advance: f32,
}

#[derive(Clone, Debug)]
pub struct TextLayout {
    pub width: f32,
    pub height: f32,
    pub line_count: usize,
    pub glyphs: Vec<ShapedGlyph>,
    raster_glyphs: Vec<RasterGlyph>,
}

#[derive(Clone, Copy, Debug)]
struct RasterGlyph {
    cache_key: CacheKey,
    x: i32,
    y: i32,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FallbackDiagnostics {
    pub system_faces: usize,
    pub distinct_fonts_used: usize,
    pub missing_glyphs: usize,
    pub atlas_full_glyphs: usize,
    pub used_multiple_fonts: bool,
}

/// Owns the system font database and rasterizer. Third-party types remain private.
pub struct FontSystem {
    inner: CosmicFontSystem,
    rasterizer: SwashCache,
    diagnostics: FallbackDiagnostics,
}

impl Default for FontSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl FontSystem {
    pub fn new() -> Self {
        let inner = CosmicFontSystem::new();
        let system_faces = inner.db().faces().count();
        Self {
            inner,
            rasterizer: SwashCache::new(),
            diagnostics: FallbackDiagnostics {
                system_faces,
                ..FallbackDiagnostics::default()
            },
        }
    }

    pub fn diagnostics(&self) -> &FallbackDiagnostics {
        &self.diagnostics
    }

    pub fn shape(
        &mut self,
        text: &str,
        style: &TextStyle,
        constraints: TextConstraints,
        scale: f32,
    ) -> TextLayout {
        let font_size = finite_positive(style.font_size, 16.0);
        let line_height = finite_positive(style.line_height, font_size * 1.25);
        let scale = finite_positive(scale, 1.0);
        let width = constraints
            .max_width
            .filter(|value| value.is_finite() && *value >= 0.0);
        let mut buffer = Buffer::new(&mut self.inner, Metrics::new(font_size, line_height));
        buffer.set_size(&mut self.inner, width, None);
        buffer.set_wrap(&mut self.inner, map_wrap(constraints.wrap));

        let attrs = if style.family.trim().is_empty() || style.family == "sans-serif" {
            Attrs::new().family(Family::SansSerif)
        } else {
            Attrs::new().family(Family::Name(style.family.trim()))
        };
        buffer.set_text(&mut self.inner, text, &attrs, Shaping::Advanced);

        let mut glyphs = Vec::new();
        let mut raster_glyphs = Vec::new();
        let mut fonts = HashSet::new();
        let mut measured_width = 0.0_f32;
        let mut measured_height = 0.0_f32;
        let mut line_count = 0;
        for run in buffer.layout_runs() {
            line_count += 1;
            measured_width = measured_width.max(run.line_w);
            measured_height = measured_height.max(run.line_top + run.line_height);
            for glyph in run.glyphs {
                fonts.insert(glyph.font_id);
                let physical = glyph.physical((0.0, 0.0), scale);
                glyphs.push(ShapedGlyph {
                    line: run.line_i,
                    byte_range: glyph.start..glyph.end,
                    x: glyph.x,
                    y: run.line_y + glyph.y,
                    advance: glyph.w,
                });
                raster_glyphs.push(RasterGlyph {
                    cache_key: physical.cache_key,
                    x: physical.x,
                    y: physical.y,
                });
            }
        }
        self.diagnostics.distinct_fonts_used = fonts.len();
        self.diagnostics.used_multiple_fonts = fonts.len() > 1;
        TextLayout {
            width: measured_width,
            height: measured_height,
            line_count,
            glyphs,
            raster_glyphs,
        }
    }

    pub fn measure(
        &mut self,
        text: &str,
        style: &TextStyle,
        constraints: TextConstraints,
    ) -> TextMeasurement {
        let layout = self.shape(text, style, constraints, 1.0);
        TextMeasurement {
            width: layout.width,
            height: layout.height,
            line_count: layout.line_count,
        }
    }

    /// Rasterize and place all glyphs not already present in `atlas`.
    pub fn prepare(&mut self, layout: &TextLayout, atlas: &mut GlyphAtlas) -> PreparedText {
        self.diagnostics.missing_glyphs = 0;
        self.diagnostics.atlas_full_glyphs = 0;
        let mut glyphs = Vec::with_capacity(layout.raster_glyphs.len());
        let mut uploads = Vec::new();
        for glyph in &layout.raster_glyphs {
            let entry = if let Some(entry) = atlas.entries.get(&glyph.cache_key).copied() {
                Some(entry)
            } else {
                let image = self
                    .rasterizer
                    .get_image_uncached(&mut self.inner, glyph.cache_key);
                match image {
                    Some(image) if image.placement.width > 0 && image.placement.height > 0 => {
                        let format = match image.content {
                            SwashContent::Color => AtlasFormat::Rgba8,
                            SwashContent::Mask | SwashContent::SubpixelMask => AtlasFormat::Alpha8,
                        };
                        atlas.insert(
                            glyph.cache_key,
                            RasterizedGlyph {
                                width: image.placement.width,
                                height: image.placement.height,
                                left: image.placement.left,
                                top: image.placement.top,
                                format,
                                pixels: image.data,
                            },
                            &mut uploads,
                        )
                    }
                    _ => {
                        self.diagnostics.missing_glyphs += 1;
                        None
                    }
                }
            };
            if let Some(entry) = entry {
                glyphs.push(PreparedGlyph {
                    atlas_id: entry.id,
                    x: glyph.x + entry.left,
                    y: glyph.y - entry.top,
                    width: entry.width,
                    height: entry.height,
                    atlas_x: entry.x,
                    atlas_y: entry.y,
                    format: entry.format,
                });
            } else if self
                .rasterizer
                .get_image(&mut self.inner, glyph.cache_key)
                .is_some()
            {
                self.diagnostics.atlas_full_glyphs += 1;
            }
        }
        PreparedText {
            atlas_generation: atlas.generation,
            glyphs,
            uploads,
        }
    }
}

pub trait TextMeasurer {
    fn measure_text(
        &mut self,
        text: &str,
        style: &TextStyle,
        constraints: TextConstraints,
    ) -> TextMeasurement;
}

impl TextMeasurer for FontSystem {
    fn measure_text(
        &mut self,
        text: &str,
        style: &TextStyle,
        constraints: TextConstraints,
    ) -> TextMeasurement {
        self.measure(text, style, constraints)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TextMeasurement {
    pub width: f32,
    pub height: f32,
    pub line_count: usize,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum AtlasFormat {
    Alpha8,
    Rgba8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AtlasUpload {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub format: AtlasFormat,
    pub pixels: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PreparedGlyph {
    pub atlas_id: u64,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub atlas_x: u32,
    pub atlas_y: u32,
    pub format: AtlasFormat,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PreparedText {
    pub atlas_generation: u64,
    pub glyphs: Vec<PreparedGlyph>,
    pub uploads: Vec<AtlasUpload>,
}

#[derive(Clone, Copy, Debug)]
struct AtlasEntry {
    id: u64,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    left: i32,
    top: i32,
    format: AtlasFormat,
}

struct RasterizedGlyph {
    width: u32,
    height: u32,
    left: i32,
    top: i32,
    format: AtlasFormat,
    pixels: Vec<u8>,
}

/// CPU-side shelf allocator bookkeeping for a renderer-owned texture.
pub struct GlyphAtlas {
    width: u32,
    height: u32,
    cursor_x: u32,
    cursor_y: u32,
    row_height: u32,
    generation: u64,
    next_id: u64,
    entries: HashMap<CacheKey, AtlasEntry>,
}

impl GlyphAtlas {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width: width.max(1),
            height: height.max(1),
            cursor_x: 0,
            cursor_y: 0,
            row_height: 0,
            generation: 0,
            next_id: 1,
            entries: HashMap::new(),
        }
    }

    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.cursor_x = 0;
        self.cursor_y = 0;
        self.row_height = 0;
        self.generation = self.generation.wrapping_add(1);
    }

    fn insert(
        &mut self,
        key: CacheKey,
        glyph: RasterizedGlyph,
        uploads: &mut Vec<AtlasUpload>,
    ) -> Option<AtlasEntry> {
        let RasterizedGlyph {
            width,
            height,
            left,
            top,
            format,
            pixels,
        } = glyph;
        if width > self.width || height > self.height {
            return None;
        }
        if self.cursor_x + width > self.width {
            self.cursor_x = 0;
            self.cursor_y = self.cursor_y.saturating_add(self.row_height + 1);
            self.row_height = 0;
        }
        if self.cursor_y + height > self.height {
            return None;
        }
        let entry = AtlasEntry {
            id: self.next_id,
            x: self.cursor_x,
            y: self.cursor_y,
            width,
            height,
            left,
            top,
            format,
        };
        self.next_id = self.next_id.wrapping_add(1);
        self.cursor_x += width + 1;
        self.row_height = self.row_height.max(height);
        self.entries.insert(key, entry);
        uploads.push(AtlasUpload {
            x: entry.x,
            y: entry.y,
            width,
            height,
            format,
            pixels,
        });
        Some(entry)
    }
}

fn finite_positive(value: f32, fallback: f32) -> f32 {
    if value.is_finite() && value > 0.0 {
        value
    } else {
        fallback
    }
}

fn map_wrap(wrap: TextWrap) -> Wrap {
    match wrap {
        TextWrap::None => Wrap::None,
        TextWrap::Word => Wrap::Word,
        TextWrap::Glyph => Wrap::Glyph,
        TextWrap::WordOrGlyph => Wrap::WordOrGlyph,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_text_is_safe() {
        let mut fonts = FontSystem::new();
        let layout = fonts.shape("", &TextStyle::default(), TextConstraints::default(), 1.0);
        assert!(layout.glyphs.is_empty());
        assert!(layout.width.is_finite());
        assert!(layout.height.is_finite());
    }

    #[test]
    fn constrained_text_wraps() {
        let mut fonts = FontSystem::new();
        let unconstrained = fonts.shape(
            "one two three four five",
            &TextStyle::default(),
            TextConstraints {
                max_width: None,
                wrap: TextWrap::WordOrGlyph,
            },
            1.0,
        );
        let wrapped = fonts.shape(
            "one two three four five",
            &TextStyle::default(),
            TextConstraints {
                max_width: Some(50.0),
                wrap: TextWrap::WordOrGlyph,
            },
            1.0,
        );
        assert!(wrapped.line_count > unconstrained.line_count);
        assert!(wrapped.width <= 50.01);
    }

    #[test]
    fn traditional_chinese_and_emoji_shape_without_panicking() {
        let mut fonts = FontSystem::new();
        let layout = fonts.shape(
            "繁體中文，標點。ABC🙂",
            &TextStyle::default(),
            TextConstraints::default(),
            1.0,
        );
        assert!(!layout.glyphs.is_empty());
        assert!(
            layout
                .glyphs
                .iter()
                .all(|glyph| glyph.x.is_finite() && glyph.advance.is_finite())
        );
        assert!(fonts.diagnostics().system_faces > 0);
    }

    #[test]
    fn dpi_changes_raster_keys_not_logical_measurement() {
        let mut fonts = FontSystem::new();
        let one = fonts.shape(
            "DPI 縮放",
            &TextStyle::default(),
            TextConstraints::default(),
            1.0,
        );
        let two = fonts.shape(
            "DPI 縮放",
            &TextStyle::default(),
            TextConstraints::default(),
            2.0,
        );
        assert_eq!(one.width, two.width);
        assert_eq!(one.height, two.height);
        assert_eq!(one.glyphs, two.glyphs);
        assert!(
            one.raster_glyphs
                .iter()
                .zip(&two.raster_glyphs)
                .any(|(a, b)| a.cache_key != b.cache_key)
        );
    }

    #[test]
    fn preparation_reuses_atlas_entries() {
        let mut fonts = FontSystem::new();
        let layout = fonts.shape(
            "Atlas",
            &TextStyle::default(),
            TextConstraints::default(),
            1.0,
        );
        let mut atlas = GlyphAtlas::new(512, 512);
        let first = fonts.prepare(&layout, &mut atlas);
        let second = fonts.prepare(&layout, &mut atlas);
        assert!(!first.glyphs.is_empty());
        assert!(!first.uploads.is_empty());
        assert!(second.uploads.is_empty());
        assert_eq!(first.glyphs, second.glyphs);
    }

    #[test]
    fn atlas_clear_forces_reupload_after_recovery() {
        // Encodes the GPU-recovery invalidation contract: after the CPU atlas is
        // cleared (as the app must do post device-loss), the next identical
        // prepare must re-emit uploads and advance the atlas generation so the
        // (now empty) GPU atlas gets repopulated instead of referencing blanks.
        let mut fonts = FontSystem::new();
        let layout = fonts.shape(
            "Recover",
            &TextStyle::default(),
            TextConstraints::default(),
            1.0,
        );
        let mut atlas = GlyphAtlas::new(512, 512);

        let first = fonts.prepare(&layout, &mut atlas);
        assert!(
            !first.uploads.is_empty(),
            "first prepare must upload glyphs"
        );
        let gen_before = atlas.generation();

        // Cache hit — no uploads on immediate identical prepare.
        let second = fonts.prepare(&layout, &mut atlas);
        assert!(
            second.uploads.is_empty(),
            "cache hit must not re-upload glyphs"
        );
        assert_eq!(second.atlas_generation, gen_before);

        // Simulate device-loss recovery: the app clears its CPU atlas.
        atlas.clear();
        assert!(atlas.is_empty());
        assert!(
            atlas.generation() > gen_before,
            "clear must advance the atlas generation"
        );

        // After invalidation, glyphs must be re-uploaded with a higher generation.
        let third = fonts.prepare(&layout, &mut atlas);
        assert!(
            !third.uploads.is_empty(),
            "post-clear prepare must re-upload glyphs"
        );
        assert!(
            third.atlas_generation > first.atlas_generation,
            "post-clear prepare must report a higher atlas generation"
        );
        assert_eq!(third.glyphs.len(), first.glyphs.len());
    }
}
