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
            let entry = if let Some(entry) = atlas.entries.get(&glyph.cache_key).copied() {
                atlas.touch(&glyph.cache_key);
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
    /// Distance from the top of the layout to the first baseline.
    /// Useful for precise vertical centering — set origin_y so that
    /// `origin_y - baseline` aligns the visual text top with the desired
    /// container top, or use `top + (height - layout_height) / 2 + baseline`
    /// for centering within a rect.
    pub baseline: f32,
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
    /// Frame index of last access — used for LRU-style eviction.
    last_used: u64,
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
///
/// Provides LRU-style eviction: call [`begin_frame`](Self::begin_frame) once
/// per frame. Automatic eviction of glyphs idle for more than 60 frames runs
/// in `begin_frame` (after an overflow, or above 75% fill), or during
/// preparation only while no glyph has been used in the current frame, so
/// glyphs handed out earlier in a frame are never overwritten. A glyph that
/// does not fit mid-frame is dropped for that frame (counted in
/// `atlas_full_glyphs`). [`evict_stale`](Self::evict_stale) evicts manually.
///
/// **Note:** Because the shelf allocator cannot reclaim fragmented space, any
/// eviction invalidates *all* atlas entries — surviving glyphs are re-uploaded
/// on subsequent frames. Call [`begin_frame`](Self::begin_frame) once per
/// frame and [`touch`](Self::touch) on cache hits to keep the LRU timestamps
/// accurate.
pub struct GlyphAtlas {
    width: u32,
    height: u32,
    cursor_x: u32,
    cursor_y: u32,
    row_height: u32,
    generation: u64,
    next_id: u64,
    /// Monotonic frame counter for LRU tracking.
    frame: u64,
    /// A glyph did not fit during a frame whose glyphs were already in use;
    /// eviction is deferred to the next [`begin_frame`](Self::begin_frame).
    needs_space: bool,
    entries: HashMap<CacheKey, AtlasEntry>,
}

/// Entries idle for more than this many frames may be evicted (≈1 s at 60 fps).
const STALE_AGE_FRAMES: u64 = 60;

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
            frame: 0,
            needs_space: false,
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
        self.needs_space = false;
        self.generation = self.generation.wrapping_add(1);
    }

    /// Advance the frame counter. Call once per frame before rendering.
    ///
    /// This is where eviction normally happens: when a glyph did not fit in
    /// the previous frame, or more than three quarters of the atlas is used,
    /// and some entries have been idle for more than 60 frames, the atlas is
    /// cleared (generation bumped). Evicting here, rather than in the middle of
    /// a frame, keeps every [`PreparedText`] already produced in the frame valid.
    pub fn begin_frame(&mut self) {
        self.frame = self.frame.wrapping_add(1);
        let used_rows = u64::from(self.cursor_y) + u64::from(self.row_height);
        let pressure = used_rows * 4 >= u64::from(self.height) * 3;
        if (self.needs_space || pressure) && self.stale_count(STALE_AGE_FRAMES) > 0 {
            self.clear();
        }
    }

    /// `true` when an entry was placed or used during the current frame.
    fn in_use_this_frame(&self) -> bool {
        self.entries
            .values()
            .any(|entry| entry.last_used == self.frame)
    }

    /// Mark an entry as recently used (updates its LRU timestamp).
    pub fn touch(&mut self, key: &CacheKey) {
        if let Some(entry) = self.entries.get_mut(key) {
            entry.last_used = self.frame;
        }
    }

    /// Number of entries not accessed within `max_age` frames.
    pub fn stale_count(&self, max_age: u64) -> usize {
        self.entries
            .values()
            .filter(|e| self.frame.saturating_sub(e.last_used) > max_age)
            .count()
    }

    /// Evict entries not accessed within `max_age` frames.
    ///
    /// Because the shelf allocator cannot reclaim fragmented space, the entire
    /// atlas is cleared when any entries are evicted — surviving glyphs will be
    /// re-uploaded on subsequent frames. Returns the number of evicted entries.
    /// The generation is bumped so the renderer can detect the invalidation.
    pub fn evict_stale(&mut self, max_age: u64) -> usize {
        let stale = self.stale_count(max_age);
        if stale > 0 {
            self.clear();
        }
        stale
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
            // Clearing now would invalidate glyphs already handed out this
            // frame (their atlas rectangles would be overwritten), so defer.
            if self.in_use_this_frame() {
                self.needs_space = true;
                return None;
            }
            // Nothing from this frame is in the atlas yet: evicting stale
            // entries is safe. `evict_stale` clears everything and resets the
            // allocator, so the insert is retried from (0,0).
            if self.evict_stale(STALE_AGE_FRAMES) > 0 {
                // Allocator was reset; re-check row-wrapping.
                if self.cursor_x + width > self.width {
                    self.cursor_x = 0;
                    self.cursor_y = 0;
                }
                if self.cursor_y + height > self.height {
                    return None; // Glyph still doesn't fit (shouldn't happen after clear)
                }
            } else {
                return None; // No stale entries to evict
            }
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
            last_used: self.frame,
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
