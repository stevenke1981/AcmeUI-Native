//! ROADMAP 011 / P2: multi-page glyph atlas lifecycle, through the public API only.
//!
//! Host-font tolerant like `atlas_lifecycle.rs`: a `(character, size)` pair
//! counts only if it shapes to exactly one glyph that rasterizes on this
//! machine. A test that needs more usable glyphs than the host offers prints
//! `SKIP` and returns. Latin shaping is never assumed in a way that hides a
//! failure: the pool is measured, not hard-coded.
//!
//! Residency model, checked after every `prepare`:
//! - an upload never overlaps an entry referenced earlier in the same frame
//!   (pages are separate textures, so only rects on the same page collide);
//! - two distinct entries referenced in one frame never share texels;
//! - every `PreparedText` of one frame reports the same `atlas_generation`;
//! - every upload is paired with a glyph of the same call, on the same page
//!   and rect.
//!
//! Entries are not tracked across frames: an eviction at a frame boundary
//! drops one page, and the public API does not say which one, so cross-frame
//! overlap is checked only where no eviction can occur (the fill test).

use std::collections::{BTreeMap, HashMap, HashSet};

use acme_text::{
    AtlasFormat, AtlasUpload, FontSystem, GlyphAtlas, GlyphAtlasConfig, PreparedGlyph,
    PreparedText, TextConstraints, TextLayout, TextStyle,
};

/// Page edge for the fill and clear tests. Small enough that the usable Latin
/// glyphs at `FONT_SIZE` cannot all fit in `MAX_PAGES` pages.
const FILL_PAGE_EDGE: u32 = 64;
/// Page edge for the rotation tests.
const ROTATION_PAGE_EDGE: u32 = 128;
const MAX_PAGES: u32 = 4;
const FONT_SIZE: f32 = 24.0;
/// Two sizes double the Latin pool, so a glyph can idle past the 60-frame
/// stale age before the rotation returns to it.
const POOL_SIZES: [f32; 2] = [20.0, 24.0];
/// Glyphs shown per frame; each is its own `PreparedText`.
const WINDOW: usize = 4;
const ROTATION_FRAMES: u64 = 600;
/// Fewer usable pool entries than this cannot age a glyph past the stale age.
const MIN_POOL: usize = 70;
/// Fewer usable glyphs at `FONT_SIZE` cannot span several small pages.
const MIN_FILL_GLYPHS: usize = 20;

fn style(size: f32) -> TextStyle {
    TextStyle {
        font_size: size,
        line_height: size + 6.0,
        family: "sans-serif".into(),
        ..TextStyle::default()
    }
}

fn shape(fonts: &mut FontSystem, text: &str, size: f32) -> TextLayout {
    fonts.shape(text, &style(size), TextConstraints::default(), 1.0)
}

fn latin_chars() -> Vec<char> {
    ('0'..='9').chain('A'..='Z').chain('a'..='z').collect()
}

fn fill_config() -> GlyphAtlasConfig {
    GlyphAtlasConfig {
        page_width: FILL_PAGE_EDGE,
        page_height: FILL_PAGE_EDGE,
        max_pages: MAX_PAGES,
    }
}

fn rotation_config(max_pages: u32) -> GlyphAtlasConfig {
    GlyphAtlasConfig {
        page_width: ROTATION_PAGE_EDGE,
        page_height: ROTATION_PAGE_EDGE,
        max_pages,
    }
}

/// A character at one font size, shaped and rasterized on this host.
struct PoolGlyph {
    c: char,
    size: f32,
    layout: TextLayout,
}

/// Keeps every `(char, size)` that shapes to one glyph, rasterizes without a
/// miss, and differs from every kept glyph. Characters with no font coverage
/// collapse onto one `.notdef` entry, so only the first of them is kept.
fn usable_pool(fonts: &mut FontSystem) -> Vec<PoolGlyph> {
    let mut scratch = GlyphAtlas::new(2048, 2048);
    let mut seen = HashSet::new();
    let mut pool = Vec::new();
    for &size in &POOL_SIZES {
        for c in latin_chars() {
            let layout = shape(fonts, &c.to_string(), size);
            if layout.glyphs.len() != 1 {
                continue;
            }
            let prepared = fonts.prepare(&layout, &mut scratch);
            let diag = fonts.diagnostics();
            let usable = prepared.glyphs.len() == 1
                && diag.missing_glyphs == 0
                && diag.atlas_full_glyphs == 0;
            if usable && seen.insert(prepared.glyphs[0].atlas_id) {
                pool.push(PoolGlyph { c, size, layout });
            }
        }
    }
    pool
}

/// An atlas rectangle on one page, in texels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Rect {
    page: u32,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
}

impl Rect {
    fn of_upload(upload: &AtlasUpload) -> Self {
        Self {
            page: upload.page,
            x: upload.x,
            y: upload.y,
            w: upload.width,
            h: upload.height,
        }
    }

    fn of_glyph(glyph: &PreparedGlyph) -> Self {
        Self {
            page: glyph.atlas_page,
            x: glyph.atlas_x,
            y: glyph.atlas_y,
            w: glyph.width,
            h: glyph.height,
        }
    }

    fn overlaps(&self, other: &Self) -> bool {
        self.page == other.page
            && self.x < other.x + other.w
            && other.x < self.x + self.w
            && self.y < other.y + other.h
            && other.y < self.y + self.h
    }

    fn inside(&self, (width, height): (u32, u32)) -> bool {
        self.x + self.w <= width && self.y + self.h <= height
    }
}

fn assert_upload_bytes(upload: &AtlasUpload) {
    let bytes_per_texel = match upload.format {
        AtlasFormat::Alpha8 => 1,
        AtlasFormat::Rgba8 => 4,
    };
    assert_eq!(
        upload.pixels.len(),
        upload.width as usize * upload.height as usize * bytes_per_texel,
        "upload {:?} has a pixel buffer of the wrong size",
        Rect::of_upload(upload)
    );
}

/// Entries referenced so far in one frame, keyed by `atlas_id`.
#[derive(Default)]
struct FrameResidency {
    generation: Option<u64>,
    referenced: HashMap<u64, Rect>,
}

impl FrameResidency {
    /// Forget the previous frame. Call after `begin_frame`.
    fn start_frame(&mut self) {
        *self = Self::default();
    }

    fn observe(&mut self, prepared: &PreparedText, atlas: &GlyphAtlas) {
        let generation = *self.generation.get_or_insert(prepared.atlas_generation);
        assert_eq!(
            prepared.atlas_generation, generation,
            "PreparedTexts of one frame disagree on atlas_generation"
        );
        assert_eq!(
            prepared.atlas_generation,
            atlas.generation(),
            "PreparedText must report the current generation"
        );
        let uploads: Vec<Rect> = prepared.uploads.iter().map(Rect::of_upload).collect();
        self.check_uploads(prepared, &uploads, atlas);
        self.check_glyphs(prepared, atlas);
        for rect in &uploads {
            assert!(
                prepared
                    .glyphs
                    .iter()
                    .any(|glyph| Rect::of_glyph(glyph) == *rect),
                "upload {rect:?} has no glyph in the same PreparedText"
            );
        }
    }

    fn check_uploads(&self, prepared: &PreparedText, uploads: &[Rect], atlas: &GlyphAtlas) {
        let dims = atlas.dimensions();
        let open_pages = atlas.page_count() as u32;
        for (upload, rect) in prepared.uploads.iter().zip(uploads) {
            assert!(
                rect.page < open_pages,
                "upload {rect:?} targets a page that is not open"
            );
            assert!(
                rect.inside(dims),
                "upload {rect:?} lies outside a {dims:?} page"
            );
            assert_upload_bytes(upload);
            assert!(
                self.referenced.values().all(|known| !known.overlaps(rect)),
                "upload {rect:?} overwrites an entry referenced earlier in this frame"
            );
        }
        for (i, rect) in uploads.iter().enumerate() {
            assert!(
                uploads[i + 1..].iter().all(|other| !other.overlaps(rect)),
                "upload {rect:?} shares texels with a later upload of the same call"
            );
        }
    }

    fn check_glyphs(&mut self, prepared: &PreparedText, atlas: &GlyphAtlas) {
        let dims = atlas.dimensions();
        let open_pages = atlas.page_count() as u32;
        for glyph in &prepared.glyphs {
            let rect = Rect::of_glyph(glyph);
            assert!(
                rect.page < open_pages && rect.inside(dims),
                "glyph {rect:?} lies outside the atlas"
            );
            match self.referenced.get(&glyph.atlas_id) {
                Some(known) => assert_eq!(
                    *known, rect,
                    "entry {} moved between prepares of one frame",
                    glyph.atlas_id
                ),
                None => {
                    assert!(
                        self.referenced.values().all(|other| !other.overlaps(&rect)),
                        "glyph {rect:?} shares texels with another entry referenced this frame"
                    );
                    self.referenced.insert(glyph.atlas_id, rect);
                }
            }
        }
    }
}

#[derive(Debug, Default)]
struct RotationStats {
    generation: u64,
    evictions: u64,
    pages_open: usize,
    uploads: usize,
    dropped: usize,
}

/// Each frame prepares `WINDOW` glyphs taken from the pool in rotation, one
/// `PreparedText` per glyph. A glyph is revisited only after the pool has
/// cycled past it, so it idles past the stale age and becomes evictable.
fn run_rotation(
    fonts: &mut FontSystem,
    pool: &[PoolGlyph],
    config: GlyphAtlasConfig,
    frames: u64,
) -> RotationStats {
    let mut atlas = GlyphAtlas::with_config(config);
    let mut residency = FrameResidency::default();
    let mut stats = RotationStats::default();
    for frame in 0..frames {
        atlas.begin_frame();
        residency.start_frame();
        for slot in 0..WINDOW {
            let glyph = &pool[(frame as usize + slot) % pool.len()];
            let prepared = fonts.prepare(&glyph.layout, &mut atlas);
            residency.observe(&prepared, &atlas);
            stats.uploads += prepared.uploads.len();
            stats.dropped += fonts.diagnostics().atlas_full_glyphs;
        }
    }
    stats.generation = atlas.generation();
    stats.evictions = atlas.page_evictions();
    stats.pages_open = atlas.page_count();
    assert_eq!(
        stats.generation, stats.evictions,
        "without clear(), each generation bump is exactly one page eviction"
    );
    stats
}

/// The longest run of usable glyphs at `FONT_SIZE` that fits every page of
/// `fill_config()` with no drop. The run stops at the first glyph that does
/// not fit, so the returned text saturates all `MAX_PAGES` pages.
fn fill_prefix(fonts: &mut FontSystem, pool: &[PoolGlyph]) -> Option<String> {
    let chars: Vec<char> = pool
        .iter()
        .filter(|glyph| glyph.size == FONT_SIZE)
        .map(|glyph| glyph.c)
        .collect();
    if chars.len() < MIN_FILL_GLYPHS {
        eprintln!(
            "SKIP: only {} glyphs shape at {FONT_SIZE}px on this system",
            chars.len()
        );
        return None;
    }
    let mut atlas = GlyphAtlas::with_config(fill_config());
    let mut prefix = String::new();
    for c in chars.iter().copied() {
        let layout = shape(fonts, &c.to_string(), FONT_SIZE);
        let prepared = fonts.prepare(&layout, &mut atlas);
        if prepared.glyphs.len() != 1 || fonts.diagnostics().atlas_full_glyphs != 0 {
            break;
        }
        prefix.push(c);
    }
    assert!(
        prefix.chars().count() < chars.len(),
        "all {} glyphs fit the fill atlas; FILL_PAGE_EDGE is too large to exercise spill",
        chars.len()
    );
    assert_eq!(
        atlas.page_count(),
        MAX_PAGES as usize,
        "a glyph was refused only after every page opened and filled"
    );
    Some(prefix)
}

#[test]
fn first_fill_spills_onto_page_one_without_overlap() {
    let mut fonts = FontSystem::new();
    let pool = usable_pool(&mut fonts);
    let Some(prefix) = fill_prefix(&mut fonts, &pool) else {
        return;
    };
    // Prepare each character as its own run, exactly as `fill_prefix`
    // measured it. Shaping the prefix as one run can change subpixel offsets
    // and so bitmap sizes, and the prefix fills every page exactly.
    let mut atlas = GlyphAtlas::with_config(fill_config());
    atlas.begin_frame();
    let mut prepared = acme_text::PreparedText {
        atlas_generation: atlas.generation(),
        glyphs: Vec::new(),
        uploads: Vec::new(),
    };
    for c in prefix.chars() {
        let layout = shape(&mut fonts, &c.to_string(), FONT_SIZE);
        let run = fonts.prepare(&layout, &mut atlas);
        assert_eq!(run.atlas_generation, prepared.atlas_generation);
        prepared.glyphs.extend(run.glyphs);
        prepared.uploads.extend(run.uploads);
    }
    assert_eq!(
        prepared.glyphs.len(),
        prefix.chars().count(),
        "every glyph of the saturating prefix must be placed"
    );

    // Residency map per page: no two live rects on one page overlap. Uploads
    // to page 1 land in their own map, so they cannot touch page 0 texels.
    let mut residency: BTreeMap<u32, Vec<Rect>> = BTreeMap::new();
    for upload in &prepared.uploads {
        let rect = Rect::of_upload(upload);
        assert!(
            rect.inside(atlas.dimensions()),
            "upload {rect:?} lies outside its page"
        );
        assert_upload_bytes(upload);
        let live = residency.entry(upload.page).or_default();
        assert!(
            live.iter().all(|other| !other.overlaps(&rect)),
            "upload {rect:?} overlaps a live rect on page {}",
            upload.page
        );
        live.push(rect);
    }
    assert_eq!(
        residency.keys().copied().collect::<Vec<_>>(),
        (0..MAX_PAGES).collect::<Vec<_>>(),
        "every page must hold at least one upload"
    );

    // Page 0 fills first: no glyph reports page 1 before the first spill, and
    // the first spill opens page 1.
    let spill = prepared
        .glyphs
        .iter()
        .position(|glyph| glyph.atlas_page != 0)
        .expect("page 0 must fill and spill");
    assert!(
        prepared.glyphs[..spill]
            .iter()
            .all(|glyph| glyph.atlas_page == 0),
        "glyphs before the first spill must stay on page 0"
    );
    assert_eq!(
        prepared.glyphs[spill].atlas_page, 1,
        "the first spill opens page 1"
    );

    // A fresh atlas misses on every glyph, so each glyph pairs with its upload
    // in order, and the upload carries the glyph's page.
    assert_eq!(prepared.uploads.len(), prepared.glyphs.len());
    for (upload, glyph) in prepared.uploads.iter().zip(&prepared.glyphs) {
        assert_eq!(Rect::of_upload(upload), Rect::of_glyph(glyph));
    }
}

#[test]
fn every_prepared_text_in_a_frame_shares_one_generation() {
    let mut fonts = FontSystem::new();
    let pool = usable_pool(&mut fonts);
    if pool.len() < MIN_POOL {
        eprintln!("SKIP: only {} usable glyph keys on this system", pool.len());
        return;
    }
    let single_page = run_rotation(&mut fonts, &pool, rotation_config(1), ROTATION_FRAMES / 2);
    if single_page.evictions == 0 {
        eprintln!("SKIP: the single-page rotation never evicted on this system");
        return;
    }
    // The residency check inside `run_rotation` compares the generation of
    // every PreparedText of each frame, including frames that evicted.
    let four_pages = run_rotation(
        &mut fonts,
        &pool,
        rotation_config(MAX_PAGES),
        ROTATION_FRAMES / 2,
    );
    println!("generation per frame: single={single_page:?} four={four_pages:?}");
}

#[test]
fn four_pages_evict_less_than_one_page_under_rotation() {
    let mut fonts = FontSystem::new();
    let pool = usable_pool(&mut fonts);
    if pool.len() < MIN_POOL {
        eprintln!("SKIP: only {} usable glyph keys on this system", pool.len());
        return;
    }
    let one = run_rotation(&mut fonts, &pool, rotation_config(1), ROTATION_FRAMES);
    let four = run_rotation(
        &mut fonts,
        &pool,
        rotation_config(MAX_PAGES),
        ROTATION_FRAMES,
    );
    println!(
        "atlas_pages rotation: {ROTATION_FRAMES} frames, window {WINDOW}, pool {}, page {ROTATION_PAGE_EDGE}px\n  \
         max_pages=1 {one:?}\n  max_pages={MAX_PAGES} {four:?}",
        pool.len()
    );
    if one.evictions == 0 {
        eprintln!("SKIP: the single-page rotation never evicted; the workload does not press here");
        return;
    }
    assert!(
        four.generation < one.generation,
        "4 pages must bump the generation less often than 1 page ({} vs {})",
        four.generation,
        one.generation
    );
    assert!(
        four.evictions < one.evictions,
        "4 pages must evict fewer pages than 1 page ({} vs {})",
        four.evictions,
        one.evictions
    );
}

#[test]
fn clear_reuploads_every_glyph_with_its_page() {
    let mut fonts = FontSystem::new();
    let pool = usable_pool(&mut fonts);
    let Some(prefix) = fill_prefix(&mut fonts, &pool) else {
        return;
    };
    let layout = shape(&mut fonts, &prefix, FONT_SIZE);
    let mut atlas = GlyphAtlas::with_config(fill_config());
    atlas.begin_frame();
    let first = fonts.prepare(&layout, &mut atlas);
    assert!(
        first.glyphs.iter().any(|glyph| glyph.atlas_page > 0),
        "precondition: the glyphs span more than one page"
    );

    let cached = fonts.prepare(&layout, &mut atlas);
    assert!(
        cached.uploads.is_empty(),
        "precondition: a cache hit uploads nothing"
    );

    let generation_before = atlas.generation();
    atlas.clear();
    assert!(atlas.is_empty());
    assert_eq!(atlas.generation(), generation_before + 1);
    assert_eq!(
        atlas.page_count(),
        MAX_PAGES as usize,
        "clear keeps pages open"
    );
    assert_eq!(atlas.page_evictions(), 0, "clear is not a page eviction");

    atlas.begin_frame();
    let after = fonts.prepare(&layout, &mut atlas);
    assert_eq!(after.atlas_generation, generation_before + 1);
    assert_eq!(after.glyphs.len(), first.glyphs.len());
    assert_eq!(
        after.uploads.len(),
        after.glyphs.len(),
        "after clear, every glyph must re-emit an upload"
    );
    for (upload, glyph) in after.uploads.iter().zip(&after.glyphs) {
        assert_eq!(
            Rect::of_upload(upload),
            Rect::of_glyph(glyph),
            "re-upload must carry the glyph's page and rect"
        );
    }
}
