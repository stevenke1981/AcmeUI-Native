//! Text shaping and renderer-neutral glyph atlas preparation for AcmeUI.
#![forbid(unsafe_op_in_unsafe_fn)]

mod atlas;

use std::collections::HashSet;

use cosmic_text::{
    Attrs, Buffer, CacheKey, Family, FontSystem as CosmicFontSystem, Metrics, Shaping, SwashCache,
    SwashContent, Wrap,
};

use atlas::RasterizedGlyph;
pub use atlas::{AtlasFormat, AtlasUpload, GlyphAtlas, GlyphAtlasConfig};

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
    /// Distance from the top of the layout to the first baseline.
    pub baseline: f32,
    pub glyphs: Vec<ShapedGlyph>,
    pub(crate) raster_glyphs: Vec<RasterGlyph>,
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
        let mut baseline = 0.0_f32;
        for run in buffer.layout_runs() {
            line_count += 1;
            if line_count == 1 {
                baseline = run.line_y;
            }
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
            baseline,
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
            baseline: layout.baseline,
        }
    }

    /// Rasterize and place all glyphs not already present in `atlas`.
    pub fn prepare(&mut self, layout: &TextLayout, atlas: &mut GlyphAtlas) -> PreparedText {
        self.diagnostics.missing_glyphs = 0;
        self.diagnostics.atlas_full_glyphs = 0;
        let mut glyphs = Vec::with_capacity(layout.raster_glyphs.len());
        let mut uploads = Vec::new();
        for glyph in &layout.raster_glyphs {
            let entry = if let Some(entry) = atlas.lookup(&glyph.cache_key) {
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
                    atlas_page: entry.page,
                    format: entry.format,
                });
            } else if self
                .rasterizer
                .get_image(&mut self.inner, glyph.cache_key)
                .as_ref()
                .is_some_and(|image| image.placement.width > 0 && image.placement.height > 0)
            {
                // Only a glyph with a bitmap can be refused for lack of atlas
                // space; whitespace has none.
                self.diagnostics.atlas_full_glyphs += 1;
            }
        }
        PreparedText {
            atlas_generation: atlas.generation(),
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
    /// Distance from the top of the layout to the first baseline.
    /// Useful for precise vertical centering — set origin_y so that
    /// `origin_y - baseline` aligns the visual text top with the desired
    /// container top, or use `top + (height - layout_height) / 2 + baseline`
    /// for centering within a rect.
    pub baseline: f32,
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
    /// Atlas page holding the glyph.
    pub atlas_page: u32,
    pub format: AtlasFormat,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PreparedText {
    pub atlas_generation: u64,
    pub glyphs: Vec<PreparedGlyph>,
    pub uploads: Vec<AtlasUpload>,
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

    // Synthetic keys and pixels keep allocator regressions independent of host fonts.
    fn test_key(glyph: u16) -> CacheKey {
        CacheKey::new(
            Default::default(),
            glyph,
            16.0,
            (0.0, 0.0),
            cosmic_text::CacheKeyFlags::empty(),
        )
        .0
    }

    fn test_glyph() -> RasterizedGlyph {
        RasterizedGlyph {
            width: 2,
            height: 2,
            left: 0,
            top: 0,
            format: AtlasFormat::Alpha8,
            pixels: vec![255; 4],
        }
    }

    #[test]
    fn overflow_preserves_current_frame_glyphs_until_begin_frame() {
        let mut atlas = GlyphAtlas::new(5, 3);
        let mut uploads = Vec::new();
        atlas
            .insert(test_key(1), test_glyph(), &mut uploads)
            .unwrap();
        for _ in 0..61 {
            atlas.begin_frame();
        }
        let live = atlas
            .insert(test_key(2), test_glyph(), &mut uploads)
            .unwrap();
        let generation = atlas.generation();
        assert!(
            atlas
                .insert(test_key(3), test_glyph(), &mut uploads)
                .is_none(),
            "overflow must not overwrite glyphs already issued this frame"
        );
        assert_eq!(atlas.generation(), generation);
        assert_eq!(atlas.entries[&test_key(2)].id, live.id);
        assert_eq!(
            uploads.len(),
            2,
            "overflow must not emit an overlapping upload"
        );
        atlas.begin_frame();
        assert!(
            atlas.is_empty(),
            "pending eviction runs at the frame boundary"
        );
        assert_eq!(atlas.generation(), generation + 1);
        assert!(
            atlas
                .insert(test_key(3), test_glyph(), &mut uploads)
                .is_some()
        );
    }

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
    fn whitespace_is_not_counted_as_an_atlas_full_glyph() {
        let mut fonts = FontSystem::new();
        let layout = fonts.shape(
            "a b c",
            &TextStyle::default(),
            TextConstraints::default(),
            1.0,
        );
        let mut atlas = GlyphAtlas::new(512, 512);
        let prepared = fonts.prepare(&layout, &mut atlas);
        if prepared.glyphs.is_empty() {
            eprintln!("SKIP: no usable host font");
            return;
        }
        assert_eq!(
            fonts.diagnostics().atlas_full_glyphs,
            0,
            "a space has no bitmap, so it is never dropped for lack of atlas space"
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

    #[test]
    fn atlas_evict_stale_clears_all_entries() {
        let mut fonts = FontSystem::new();
        let layout = fonts.shape(
            "Evict",
            &TextStyle::default(),
            TextConstraints::default(),
            1.0,
        );
        let mut atlas = GlyphAtlas::new(512, 512);

        // First prepare: upload glyphs.
        let first = fonts.prepare(&layout, &mut atlas);
        assert!(!first.uploads.is_empty());
        assert_eq!(atlas.len(), first.glyphs.len());
        let gen_before = atlas.generation();

        // Advance frames so entries become stale (max_age = 1 frame).
        atlas.begin_frame(); // frame 1
        atlas.begin_frame(); // frame 2

        assert!(
            atlas.stale_count(1) > 0,
            "entries should be stale after 2 frames with max_age=1"
        );

        // Evict stale entries — should clear everything.
        let evicted = atlas.evict_stale(1);
        assert!(evicted > 0, "must have evicted at least one entry");
        assert!(
            atlas.is_empty(),
            "all entries must be cleared after eviction"
        );
        assert!(
            atlas.generation() > gen_before,
            "evict_stale must advance the atlas generation"
        );
    }

    #[test]
    fn atlas_auto_evict_on_full() {
        let mut fonts = FontSystem::new();
        // Tiny atlas so it fills up fast.
        let mut atlas = GlyphAtlas::new(32, 32);

        // Fill the atlas until it is full.
        let mut full = false;
        for letter in 'A'..='Z' {
            let letter_layout = fonts.shape(
                &letter.to_string(),
                &TextStyle::default(),
                TextConstraints::default(),
                1.0,
            );
            let prepared = fonts.prepare(&letter_layout, &mut atlas);
            if prepared.uploads.is_empty() && !prepared.glyphs.is_empty() {
                continue; // all cached
            }
            if fonts.diagnostics().atlas_full_glyphs > 0 {
                full = true;
                break;
            }
        }
        if !full {
            // 32×32 atlas didn't fill with A-Z (very compact glyphs).
            return;
        }

        // Advance frame counter past the 60-frame max_age used by auto-eviction.
        for _ in 0..61 {
            atlas.begin_frame();
        }

        // A glyph did not fit, so the frame boundary evicted the stale entries.
        assert!(
            atlas.generation() > 0,
            "begin_frame must evict once entries are stale after an overflow"
        );

        // The glyph that did not fit is now placed.
        let new_layout = fonts.shape("!", &TextStyle::default(), TextConstraints::default(), 1.0);
        let after = fonts.prepare(&new_layout, &mut atlas);
        assert!(
            !after.uploads.is_empty(),
            "new glyph should have been uploaded after auto-eviction"
        );
        assert!(
            !after.glyphs.is_empty(),
            "new glyph should be present after auto-eviction"
        );
        // Generation should have been bumped by the eviction.
        assert!(
            after.atlas_generation > 0,
            "atlas generation must be >0 after eviction"
        );
    }

    #[test]
    fn atlas_touch_updates_last_used() {
        let mut fonts = FontSystem::new();
        let layout = fonts.shape(
            "Touch",
            &TextStyle::default(),
            TextConstraints::default(),
            1.0,
        );
        let mut atlas = GlyphAtlas::new(512, 512);

        let first = fonts.prepare(&layout, &mut atlas);
        assert!(!first.uploads.is_empty());
        assert_eq!(first.glyphs.len(), atlas.len());

        // Advance frame counter.
        atlas.begin_frame();

        // Second prepare: should touch all cache keys, updating last_used.
        let second = fonts.prepare(&layout, &mut atlas);
        assert!(second.uploads.is_empty(), "cache hit must not re-upload");

        // After one frame advance, entries should NOT be stale (they were touched).
        atlas.begin_frame();
        assert_eq!(atlas.stale_count(1), 0, "touched entries must not be stale");
    }

    #[test]
    fn begin_frame_pressure_check_does_not_overflow_on_tall_atlas() {
        let mut atlas = GlyphAtlas::new(1, u32::MAX);
        atlas.begin_frame();
        assert_eq!(atlas.generation(), 0);
    }

    #[test]
    fn clear_resets_deferred_eviction_request() {
        let mut fonts = FontSystem::new();
        let mut atlas = GlyphAtlas::new(64, 64);
        let style = TextStyle {
            font_size: 24.0,
            line_height: 30.0,
            ..TextStyle::default()
        };
        let overflow = fonts.shape(
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
            &style,
            TextConstraints::default(),
            1.0,
        );
        fonts.prepare(&overflow, &mut atlas);
        assert!(
            fonts.diagnostics().atlas_full_glyphs > 0,
            "precondition: overflow"
        );
        atlas.clear();
        let generation = atlas.generation();

        let one = fonts.shape("A", &style, TextConstraints::default(), 1.0);
        fonts.prepare(&one, &mut atlas);
        for _ in 0..70 {
            atlas.begin_frame();
        }
        assert_eq!(
            atlas.generation(),
            generation,
            "a nearly empty atlas must not be evicted by a request made before clear()"
        );
        assert_eq!(atlas.len(), 1);
    }
}
