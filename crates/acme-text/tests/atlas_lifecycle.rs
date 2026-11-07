//! ROADMAP 006 / B2: glyph atlas lifecycle under pressure.
//!
//! Uses only the public `acme_text` API. System fonts differ per OS, so a
//! character that does not shape to exactly one glyph with a rasterized bitmap
//! on the current machine is skipped. Latin must always shape. Hit rates are
//! printed (run with `--nocapture`) and never asserted against a target.
//!
//! Eviction contract under test:
//! - Eviction clears the whole atlas (shelf allocator) and bumps the
//!   generation. It happens in `begin_frame`, or inside `prepare` only while no
//!   glyph has been handed out in the current frame, so every `PreparedText`
//!   of one frame shares one generation and never references overwritten
//!   texels.
//! - Scenario constraint (documented, not hidden): the per-frame working set
//!   stays below capacity over any 60-frame window, because only entries idle
//!   for more than 60 frames are evicted. A working set above that is the
//!   `#[ignore]`d starvation test.

use std::collections::HashSet;

use acme_text::{
    AtlasFormat, AtlasUpload, FontSystem, GlyphAtlas, PreparedGlyph, PreparedText, TextConstraints,
    TextLayout, TextStyle,
};

/// Idle age used by the crate's automatic eviction (`STALE_AGE_FRAMES`, not public).
const STALE_AGE: u64 = 60;
const ATLAS_SIDE: u32 = 256;
const FONT_SIZE: f32 = 24.0;
const ROTATION_FRAMES: usize = 600;
const EMOJI_EVERY: usize = 25;
const EXPLICIT_CLEAR_FRAME: usize = 300;
/// Fewer shapeable CJK characters than this cannot press a 256x256 atlas.
const MIN_CJK_FOR_PRESSURE: usize = 150;
const POST_EVICTION_FRAMES: u64 = 4;

/// Common Traditional Chinese characters; duplicates are removed at runtime.
const CJK_COMMON: &str = "的一是不了在人有我他這個們中來上大為和國地到以說時要就出會可也你對生能而子那得於著下自之年過發後作里用道行所然家種事成方多經麼去法學如都同現當沒動面起看定天分還進好小部其些主樣理心她本前開但因只從想實日軍者意無力它與長把機十民第公此已工使情明性知全三又關點正業外將兩高間由問很最重並物手應戰向頭文體政美相見被利什二等產或新己制身果加西斯月話合回特代內信表化老給世位次度門任常先海通教兒原東聲提立及比員解水名真論處走義各入幾認條平系";
const EMOJI: &str = "🙂🚀🎉🔥";

fn style() -> TextStyle {
    TextStyle {
        font_size: FONT_SIZE,
        line_height: 30.0,
        family: "sans-serif".into(),
    }
}

fn latin_chars() -> Vec<char> {
    ('0'..='9').chain('A'..='Z').chain('a'..='z').collect()
}

fn unique_chars(text: &str) -> Vec<char> {
    let mut seen = HashSet::new();
    text.chars()
        .filter(|c| !c.is_whitespace() && seen.insert(*c))
        .collect()
}

fn single_char_layout(fonts: &mut FontSystem, c: char) -> TextLayout {
    fonts.shape(&c.to_string(), &style(), TextConstraints::default(), 1.0)
}

/// Keeps candidates that shape to exactly one glyph and rasterize without a
/// miss on this system, and whose glyph is distinct from every glyph already
/// kept. A tofu fallback cannot be told apart through the public API, but all
/// characters that collapse onto one `.notdef` glyph share one atlas entry, so
/// only the first of them is kept and the pressure thresholds stay honest on
/// systems without CJK or emoji fonts.
fn shapeable_chars(fonts: &mut FontSystem, candidates: &[char]) -> Vec<char> {
    let mut scratch = GlyphAtlas::new(2048, 2048);
    let mut seen_entries = HashSet::new();
    let mut kept = Vec::new();
    for &c in candidates {
        let layout = single_char_layout(fonts, c);
        if layout.glyphs.len() != 1 {
            continue;
        }
        let prepared = fonts.prepare(&layout, &mut scratch);
        let diag = fonts.diagnostics();
        if prepared.glyphs.len() == 1
            && diag.missing_glyphs == 0
            && diag.atlas_full_glyphs == 0
            && seen_entries.insert(prepared.glyphs[0].atlas_id)
        {
            kept.push(c);
        }
    }
    kept
}

/// Atlas rectangle in texels, taken from uploads or prepared glyphs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Rect {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
}

impl Rect {
    fn of_upload(upload: &AtlasUpload) -> Self {
        Self {
            x: upload.x,
            y: upload.y,
            w: upload.width,
            h: upload.height,
        }
    }

    fn of_glyph(glyph: &PreparedGlyph) -> Self {
        Self {
            x: glyph.atlas_x,
            y: glyph.atlas_y,
            w: glyph.width,
            h: glyph.height,
        }
    }

    fn area(&self) -> u64 {
        u64::from(self.w) * u64::from(self.h)
    }

    fn inside(&self, (width, height): (u32, u32)) -> bool {
        self.x + self.w <= width && self.y + self.h <= height
    }

    fn overlaps(&self, other: &Self) -> bool {
        self.x < other.x + other.w
            && other.x < self.x + self.w
            && self.y < other.y + other.h
            && other.y < self.y + self.h
    }
}

fn bytes_per_pixel(format: AtlasFormat) -> usize {
    match format {
        AtlasFormat::Alpha8 => 1,
        AtlasFormat::Rgba8 => 4,
    }
}

/// Tracks the atlas memory that is valid since the last invalidation. Every
/// observed `PreparedText` must stay within capacity, must not overlap live
/// allocations, and must only reference memory uploaded since the last
/// invalidation.
#[derive(Default)]
struct Residency {
    generation: u64,
    live: Vec<Rect>,
}

impl Residency {
    /// Returns `(glyphs, uploads)` for the call; hits are `glyphs - uploads`.
    fn observe(&mut self, prepared: &PreparedText, dims: (u32, u32)) -> (usize, usize) {
        if prepared.atlas_generation != self.generation {
            self.live.clear();
            self.generation = prepared.atlas_generation;
        }
        for upload in &prepared.uploads {
            let rect = Rect::of_upload(upload);
            assert!(
                rect.inside(dims),
                "upload {rect:?} lies outside atlas {dims:?}"
            );
            assert_eq!(
                upload.pixels.len(),
                rect.area() as usize * bytes_per_pixel(upload.format),
                "upload {rect:?} pixel buffer has the wrong size"
            );
            assert!(
                self.live.iter().all(|other| !rect.overlaps(other)),
                "upload {rect:?} overlaps a live allocation"
            );
            self.live.push(rect);
        }
        let live_area: u64 = self.live.iter().map(Rect::area).sum();
        assert!(
            live_area <= u64::from(dims.0) * u64::from(dims.1),
            "live allocations ({live_area} texels) exceed atlas capacity"
        );
        assert!(
            prepared.uploads.len() <= prepared.glyphs.len(),
            "more uploads than prepared glyphs"
        );
        for glyph in &prepared.glyphs {
            let rect = Rect::of_glyph(glyph);
            assert!(
                rect.area() > 0 && self.live.contains(&rect),
                "glyph {glyph:?} references memory not uploaded since the last invalidation"
            );
        }
        (prepared.glyphs.len(), prepared.uploads.len())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Lane {
    Hot,
    Cold,
    Emoji,
}

#[derive(Default, Debug)]
struct LaneTally {
    calls: u64,
    glyphs: u64,
    uploads: u64,
}

impl LaneTally {
    fn hit_rate_percent(&self) -> f64 {
        if self.glyphs == 0 {
            return 0.0;
        }
        (self.glyphs - self.uploads) as f64 * 100.0 / self.glyphs as f64
    }

    fn report(&self, name: &str) {
        println!(
            "  {name:<6} calls={:<5} glyphs={:<6} uploads(misses)={:<6} hits={:<6} hit_rate={:.2}%",
            self.calls,
            self.glyphs,
            self.uploads,
            self.glyphs - self.uploads,
            self.hit_rate_percent()
        );
    }
}

struct Outcome {
    prepared: PreparedText,
    /// The atlas generation changed during this call (eviction).
    evicted: bool,
}

/// Drives `FontSystem::prepare` against one atlas and checks every call.
struct Harness {
    fonts: FontSystem,
    atlas: GlyphAtlas,
    residency: Residency,
    frame: u64,
    explicit_clears: u64,
    evictions: u64,
    /// Generation of the first `PreparedText` of the current frame.
    frame_generation: Option<u64>,
    hot: LaneTally,
    cold: LaneTally,
    emoji: LaneTally,
}

impl Harness {
    fn new() -> Self {
        Self {
            fonts: FontSystem::new(),
            atlas: GlyphAtlas::new(ATLAS_SIDE, ATLAS_SIDE),
            residency: Residency::default(),
            frame: 0,
            explicit_clears: 0,
            evictions: 0,
            frame_generation: None,
            hot: LaneTally::default(),
            cold: LaneTally::default(),
            emoji: LaneTally::default(),
        }
    }

    /// Returns `true` when the frame boundary evicted the atlas.
    fn begin_frame(&mut self) -> bool {
        let generation_before = self.atlas.generation();
        self.atlas.begin_frame();
        self.frame += 1;
        self.frame_generation = None;
        let evicted = self.atlas.generation() != generation_before;
        if evicted {
            self.evictions += 1;
            assert_eq!(
                self.atlas.generation(),
                generation_before + 1,
                "frame {}: one eviction must bump the generation once",
                self.frame
            );
            assert!(
                self.atlas.is_empty(),
                "frame {}: eviction must clear the atlas",
                self.frame
            );
        }
        evicted
    }

    fn clear(&mut self) {
        self.atlas.clear();
        self.explicit_clears += 1;
        self.frame_generation = None;
    }

    fn prepare(&mut self, layout: &TextLayout, lane: Lane) -> Outcome {
        let generation_before = self.atlas.generation();
        let stale_before = self.atlas.stale_count(STALE_AGE);
        let prepared = self.fonts.prepare(layout, &mut self.atlas);
        let evicted = self.atlas.generation() != generation_before;
        if evicted {
            self.evictions += 1;
            assert!(
                stale_before > 0,
                "frame {}: eviction happened while no entry was stale",
                self.frame
            );
            assert_eq!(
                self.atlas.stale_count(STALE_AGE),
                0,
                "frame {}: stale entries survived an eviction",
                self.frame
            );
        }
        assert_eq!(
            prepared.atlas_generation,
            self.atlas.generation(),
            "PreparedText must report the current generation"
        );
        let frame_generation = *self
            .frame_generation
            .get_or_insert(prepared.atlas_generation);
        assert_eq!(
            prepared.atlas_generation, frame_generation,
            "frame {}: the atlas was invalidated after glyphs were handed out this frame",
            self.frame
        );
        let diag = self.fonts.diagnostics();
        assert_eq!(
            (diag.missing_glyphs, diag.atlas_full_glyphs),
            (0, 0),
            "frame {}: a current-frame glyph was not resolvable",
            self.frame
        );
        assert_eq!(
            prepared.glyphs.len(),
            layout.glyphs.len(),
            "frame {}: every current-frame glyph must be resolvable",
            self.frame
        );
        let (glyphs, uploads) = self.residency.observe(&prepared, self.atlas.dimensions());
        let tally = match lane {
            Lane::Hot => &mut self.hot,
            Lane::Cold => &mut self.cold,
            Lane::Emoji => &mut self.emoji,
        };
        tally.calls += 1;
        tally.glyphs += glyphs as u64;
        tally.uploads += uploads as u64;
        Outcome { prepared, evicted }
    }
}

fn hot_layout(fonts: &mut FontSystem) -> TextLayout {
    let latin = latin_chars();
    assert_eq!(
        shapeable_chars(fonts, &latin).len(),
        latin.len(),
        "the Latin set must shape and rasterize on every supported system"
    );
    let text: String = latin.iter().collect();
    fonts.shape(&text, &style(), TextConstraints::default(), 1.0)
}

#[test]
fn latin_set_shapes_and_resolves_in_one_frame() {
    let mut h = Harness::new();
    let layout = hot_layout(&mut h.fonts);
    h.begin_frame();
    let outcome = h.prepare(&layout, Lane::Hot);
    assert_eq!(outcome.prepared.glyphs.len(), latin_chars().len());
    assert_eq!(outcome.prepared.uploads.len(), latin_chars().len());
    assert!(!outcome.evicted);
}

#[test]
fn explicit_clear_forces_full_reupload_and_bumps_generation() {
    let mut h = Harness::new();
    let layout = hot_layout(&mut h.fonts);

    h.begin_frame();
    let first = h.prepare(&layout, Lane::Hot);
    let first_uploads = first.prepared.uploads.len();
    assert!(first_uploads > 0, "first prepare must upload glyphs");

    h.begin_frame();
    let cached = h.prepare(&layout, Lane::Hot);
    assert!(
        cached.prepared.uploads.is_empty(),
        "cache hit must not upload"
    );
    let generation_before_clear = h.atlas.generation();

    h.clear();
    assert!(h.atlas.is_empty());
    assert_eq!(h.atlas.generation(), generation_before_clear + 1);

    h.begin_frame();
    let after = h.prepare(&layout, Lane::Hot);
    assert_eq!(
        after.prepared.uploads.len(),
        first_uploads,
        "clear must force a full re-upload"
    );
    assert_eq!(after.prepared.atlas_generation, generation_before_clear + 1);
    assert_eq!(after.prepared.glyphs.len(), first.prepared.glyphs.len());
}

#[test]
fn hot_set_is_not_counted_stale_while_idle_glyphs_age_out() {
    let mut h = Harness::new();
    let hot = hot_layout(&mut h.fonts);
    let cjk = shapeable_chars(&mut h.fonts, &unique_chars(CJK_COMMON));
    if cjk.len() < MIN_CJK_FOR_PRESSURE {
        eprintln!(
            "SKIP: only {} CJK characters shape on this system",
            cjk.len()
        );
        return;
    }

    // Each cold glyph is distinct and used once, so it ages out after STALE_AGE frames.
    let mut cold_insert_frames: Vec<u64> = Vec::new();
    let mut first_eviction: Option<u64> = None;
    let mut hot_generation = h.atlas.generation();
    for (index, &c) in cjk.iter().enumerate() {
        if first_eviction.is_some_and(|at| h.frame >= at + POST_EVICTION_FRAMES) {
            break;
        }
        if h.begin_frame() {
            // Eviction drops everything; the cold glyphs start over.
            cold_insert_frames.clear();
            first_eviction.get_or_insert(h.frame);
        }
        let hot_invalidated = h.atlas.generation() != hot_generation;
        let hot_outcome = h.prepare(&hot, Lane::Hot);
        hot_generation = h.atlas.generation();
        assert!(!hot_outcome.evicted, "hot lane must never trigger eviction");
        if index == 0 || hot_invalidated {
            assert_eq!(
                hot_outcome.prepared.uploads.len(),
                hot.glyphs.len(),
                "frame {}: hot set must be fully re-uploaded after invalidation",
                h.frame
            );
        } else {
            assert!(
                hot_outcome.prepared.uploads.is_empty(),
                "frame {}: hot set was re-rasterized while the atlas was not invalidated",
                h.frame
            );
        }

        let layout = single_char_layout(&mut h.fonts, c);
        let cold_outcome = h.prepare(&layout, Lane::Cold);
        assert!(
            !cold_outcome.evicted,
            "frame {}: the hot set was handed out first, so prepare must not evict",
            h.frame
        );
        cold_insert_frames.push(h.frame);
        let expected_stale = cold_insert_frames
            .iter()
            .filter(|&&inserted| h.frame - inserted > STALE_AGE)
            .count();
        assert_eq!(
            h.atlas.stale_count(STALE_AGE),
            expected_stale,
            "frame {}: stale count must equal idle cold glyphs; the hot set must never be stale",
            h.frame
        );
    }
    assert!(
        first_eviction.is_some(),
        "the distinct glyph sequence never pressed the atlas into an eviction"
    );
    println!(
        "hot-set accounting: first eviction at frame {:?}, evictions={}",
        first_eviction, h.evictions
    );
}

#[test]
fn rotation_over_600_frames_keeps_atlas_invariants() {
    let mut h = Harness::new();
    let hot = hot_layout(&mut h.fonts);
    let cjk = shapeable_chars(&mut h.fonts, &unique_chars(CJK_COMMON));
    let emoji_chars = shapeable_chars(&mut h.fonts, &unique_chars(EMOJI));
    if cjk.is_empty() {
        eprintln!("SKIP CJK lane: no CJK character shapes on this system");
    }

    let mut hot_full_uploads: Option<usize> = None;
    let mut hot_generation = h.atlas.generation();
    for frame in 0..ROTATION_FRAMES {
        let _ = h.begin_frame();
        if frame == EXPLICIT_CLEAR_FRAME {
            h.clear();
        }

        let hot_invalidated = h.atlas.generation() != hot_generation;
        let hot_outcome = h.prepare(&hot, Lane::Hot);
        hot_generation = h.atlas.generation();
        assert!(
            !hot_outcome.evicted,
            "frame {frame}: hot lane must never trigger eviction"
        );
        let full = *hot_full_uploads.get_or_insert(hot_outcome.prepared.uploads.len());
        assert!(full > 0);
        if frame == 0 || hot_invalidated {
            assert_eq!(
                hot_outcome.prepared.uploads.len(),
                full,
                "frame {frame}: hot set must be fully re-uploaded after invalidation"
            );
        } else {
            assert!(
                hot_outcome.prepared.uploads.is_empty(),
                "frame {frame}: hot set was re-rasterized without invalidation"
            );
        }

        if !cjk.is_empty() {
            let cold_layout = single_char_layout(&mut h.fonts, cjk[frame % cjk.len()]);
            h.prepare(&cold_layout, Lane::Cold);
        }

        if frame % EMOJI_EVERY == 0 && !emoji_chars.is_empty() {
            let emoji = emoji_chars[(frame / EMOJI_EVERY) % emoji_chars.len()];
            let emoji_layout = single_char_layout(&mut h.fonts, emoji);
            h.prepare(&emoji_layout, Lane::Emoji);
        }
    }

    assert_eq!(h.explicit_clears, 1);
    assert_eq!(
        h.atlas.generation(),
        h.explicit_clears + h.evictions,
        "generation must advance exactly once per clear or eviction"
    );
    if cjk.len() >= MIN_CJK_FOR_PRESSURE {
        assert!(
            h.evictions >= 1,
            "the rotation never pressed the atlas; the eviction path was not exercised"
        );
    } else {
        eprintln!(
            "SKIP pressure assertion: only {} CJK characters shape",
            cjk.len()
        );
    }

    let total_calls = h.hot.calls + h.cold.calls + h.emoji.calls;
    let total_glyphs = h.hot.glyphs + h.cold.glyphs + h.emoji.glyphs;
    let total_uploads = h.hot.uploads + h.cold.uploads + h.emoji.uploads;
    let (width, height) = h.atlas.dimensions();
    println!(
        "atlas_lifecycle rotation: {ROTATION_FRAMES} frames, atlas {width}x{height}, font {FONT_SIZE}px, \
         latin={} cjk_shapeable={} emoji_shapeable={}",
        latin_chars().len(),
        cjk.len(),
        emoji_chars.len()
    );
    h.hot.report("hot");
    h.cold.report("cjk");
    h.emoji.report("emoji");
    println!(
        "  overall calls={total_calls} glyphs={total_glyphs} uploads={total_uploads} \
         hit_rate={:.2}% evictions={} explicit_clears={} final_generation={} entries={}",
        (total_glyphs - total_uploads) as f64 * 100.0 / total_glyphs as f64,
        h.evictions,
        h.explicit_clears,
        h.atlas.generation(),
        h.atlas.len()
    );
}

fn rects_overlap(a: &PreparedGlyph, b: &PreparedGlyph) -> bool {
    Rect::of_glyph(a).overlaps(&Rect::of_glyph(b))
}

#[test]
fn overflow_mid_frame_defers_eviction_to_next_frame() {
    // Regression: eviction used to clear the atlas in the middle of a
    // `prepare`, so glyphs placed earlier in the same call shared texels with
    // later ones. Now the overflowing glyphs are dropped for this frame and the
    // stale entries are evicted at the next frame boundary.
    let mut fonts = FontSystem::new();
    let mut atlas = GlyphAtlas::new(64, 64);
    let warm = single_char_layout(&mut fonts, 'A');
    fonts.prepare(&warm, &mut atlas);
    for _ in 0..=STALE_AGE {
        atlas.begin_frame();
    }
    assert_eq!(
        atlas.stale_count(STALE_AGE),
        1,
        "precondition: 'A' is stale"
    );
    assert_eq!(atlas.generation(), 0, "precondition: no pressure yet");

    let layout = fonts.shape(
        "abcdefghijklmnopqrstuvwxyz0123456789",
        &style(),
        TextConstraints::default(),
        1.0,
    );
    let prepared = fonts.prepare(&layout, &mut atlas);
    assert_eq!(
        prepared.atlas_generation, 0,
        "prepare must not evict after it handed out glyphs this frame"
    );
    assert!(
        fonts.diagnostics().atlas_full_glyphs > 0,
        "precondition: 36 glyphs overflow a 64x64 atlas"
    );
    for (i, a) in prepared.glyphs.iter().enumerate() {
        for b in &prepared.glyphs[i + 1..] {
            if a.atlas_id != b.atlas_id {
                assert!(
                    !rects_overlap(a, b),
                    "glyphs {a:?} and {b:?} share atlas texels in one PreparedText"
                );
            }
        }
    }

    atlas.begin_frame();
    assert_eq!(
        atlas.generation(),
        1,
        "the frame boundary evicts the stale 'A'"
    );
    assert!(atlas.is_empty());
    let retry_layout = single_char_layout(&mut fonts, 'z');
    let retry = fonts.prepare(&retry_layout, &mut atlas);
    assert_eq!(retry.glyphs.len(), 1);
    assert_eq!(retry.uploads.len(), 1, "evicted glyphs are re-uploaded");
}

#[test]
#[ignore = "KNOWN LIMITATION: a glyph is dropped (atlas_full) while the atlas holds only entries used within the last 60 frames; needs a multi-page or real LRU allocator"]
fn bug_glyph_dropped_while_atlas_holds_only_recent_entries() {
    let mut fonts = FontSystem::new();
    let mut atlas = GlyphAtlas::new(64, 64);
    let fill = fonts.shape("ABCDEFGHIJKLMN", &style(), TextConstraints::default(), 1.0);
    let filled = fonts.prepare(&fill, &mut atlas);
    assert_eq!(
        filled.glyphs.len(),
        14,
        "precondition: 14 glyphs fill the 64x64 atlas"
    );

    atlas.begin_frame();
    let next = single_char_layout(&mut fonts, 'O');
    let prepared = fonts.prepare(&next, &mut atlas);
    assert_eq!(
        prepared.glyphs.len(),
        1,
        "the needed glyph 'O' fits an empty 64x64 atlas but was dropped (atlas_full={})",
        fonts.diagnostics().atlas_full_glyphs
    );
}
