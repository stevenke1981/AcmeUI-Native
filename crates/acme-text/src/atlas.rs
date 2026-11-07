//! CPU-side glyph atlas bookkeeping: shelf allocation per page, page selection,
//! and LRU-stale page eviction. Public types are re-exported from the crate root.

use std::collections::HashMap;

use cosmic_text::CacheKey;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum AtlasFormat {
    Alpha8,
    Rgba8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AtlasUpload {
    /// Atlas page the region belongs to.
    pub page: u32,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub format: AtlasFormat,
    pub pixels: Vec<u8>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct AtlasEntry {
    pub(crate) id: u64,
    pub(crate) page: u32,
    pub(crate) x: u32,
    pub(crate) y: u32,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) left: i32,
    pub(crate) top: i32,
    pub(crate) format: AtlasFormat,
    /// Frame index of last access — used for LRU-style eviction.
    pub(crate) last_used: u64,
}

pub(crate) struct RasterizedGlyph {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) left: i32,
    pub(crate) top: i32,
    pub(crate) format: AtlasFormat,
    pub(crate) pixels: Vec<u8>,
}

/// Shelf cursors and recency of one page.
#[derive(Clone, Copy, Debug, Default)]
struct Page {
    cursor_x: u32,
    cursor_y: u32,
    row_height: u32,
    /// Frame of the most recent placement or use on this page.
    last_used: u64,
}

/// Position of a glyph on a page and the page state after placing it.
#[derive(Clone, Copy, Debug)]
struct Placement {
    x: u32,
    y: u32,
    next: Page,
}

/// CPU-side shelf allocator bookkeeping for renderer-owned atlas pages.
///
/// A glyph goes to the active page first, then to the other open pages in
/// ascending order, then to a newly opened page while `max_pages` allows it.
/// Only when every page is full does the atlas evict: one page at a time, the
/// least recently used page that holds an entry idle for more than 60 frames.
/// Eviction runs in [`begin_frame`](Self::begin_frame) after an overflow or
/// under pressure, or inside an insert only while no glyph has been used in the
/// current frame, so glyphs handed out earlier in a frame are never overwritten.
/// A glyph that cannot be placed mid-frame is dropped for that frame.
/// [`evict_stale`](Self::evict_stale) evicts manually.
///
/// Evicting a page invalidates only the entries on that page; surviving entries
/// on other pages stay valid. Every eviction bumps [`generation`](Self::generation)
/// once. [`clear`](Self::clear) resets all pages without counting an eviction.
pub struct GlyphAtlas {
    /// Size of one page.
    width: u32,
    height: u32,
    generation: u64,
    next_id: u64,
    /// Monotonic frame counter for LRU tracking.
    frame: u64,
    /// A glyph did not fit during a frame whose glyphs were already in use;
    /// eviction is deferred to the next [`begin_frame`](Self::begin_frame).
    needs_space: bool,
    /// Upper bound on open pages (1 = legacy single-page behavior).
    max_pages: u32,
    /// Page most recently placed into or re-targeted by an eviction.
    active: usize,
    /// Open pages; never empty and never shrinks.
    pages: Vec<Page>,
    /// Cumulative number of page evictions (explicit `clear` excluded).
    evictions: u64,
    pub(crate) entries: HashMap<CacheKey, AtlasEntry>,
}

/// Page geometry and page budget of a [`GlyphAtlas`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GlyphAtlasConfig {
    /// Width of one page in pixels.
    pub page_width: u32,
    /// Height of one page in pixels.
    pub page_height: u32,
    /// Maximum number of pages the atlas may open.
    pub max_pages: u32,
}

impl GlyphAtlasConfig {
    /// Largest accepted `max_pages`; equals `acme_core::ATLAS_MAX_PAGES`.
    pub const MAX_PAGES_CAP: u32 = 8;
    /// Default page edge; equals `acme_core::ATLAS_PAGE_EDGE`.
    pub const DEFAULT_PAGE_EDGE: u32 = 2048;
    /// Default page budget (4 × 2048² holds as many texels as one 4096² page).
    pub const DEFAULT_MAX_PAGES: u32 = 4;
}

impl Default for GlyphAtlasConfig {
    fn default() -> Self {
        Self {
            page_width: Self::DEFAULT_PAGE_EDGE,
            page_height: Self::DEFAULT_PAGE_EDGE,
            max_pages: Self::DEFAULT_MAX_PAGES,
        }
    }
}

impl Default for GlyphAtlas {
    fn default() -> Self {
        Self::with_config(GlyphAtlasConfig::default())
    }
}

/// Entries idle for more than this many frames may be evicted (≈1 s at 60 fps).
const STALE_AGE_FRAMES: u64 = 60;

impl GlyphAtlas {
    /// A single-page atlas of `width × height` pixels (legacy behavior:
    /// eviction clears the whole page). See [`with_config`](Self::with_config)
    /// for a multi-page atlas.
    pub fn new(width: u32, height: u32) -> Self {
        Self::with_config(GlyphAtlasConfig {
            page_width: width,
            page_height: height,
            max_pages: 1,
        })
    }

    /// An atlas with the given page geometry and page budget. Dimensions are
    /// clamped to at least 1 and `max_pages` to `1..=MAX_PAGES_CAP`.
    pub fn with_config(config: GlyphAtlasConfig) -> Self {
        Self {
            max_pages: config.max_pages.clamp(1, GlyphAtlasConfig::MAX_PAGES_CAP),
            width: config.page_width.max(1),
            height: config.page_height.max(1),
            generation: 0,
            next_id: 1,
            frame: 0,
            needs_space: false,
            active: 0,
            pages: vec![Page::default()],
            evictions: 0,
            entries: HashMap::new(),
        }
    }

    /// Size of one page in pixels.
    pub fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Number of open pages (at least 1).
    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    /// Maximum number of pages this atlas may open.
    pub fn max_pages(&self) -> u32 {
        self.max_pages
    }

    /// Cumulative number of page evictions (explicit `clear` not counted).
    pub fn page_evictions(&self) -> u64 {
        self.evictions
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

    /// Drops every entry and resets all pages. Pages stay open. Not counted as
    /// a page eviction; bumps the generation.
    pub fn clear(&mut self) {
        self.entries.clear();
        for page in &mut self.pages {
            *page = Page::default();
        }
        self.active = 0;
        self.needs_space = false;
        self.generation = self.generation.wrapping_add(1);
    }

    /// Advance the frame counter. Call once per frame before rendering.
    ///
    /// Evicts at most one page: when a glyph was refused during a frame whose
    /// glyphs were in use, or when every page is at least three quarters full
    /// and the page budget is exhausted, the least recently used page holding
    /// an entry idle for more than 60 frames is emptied (generation bumped
    /// once). Evicting here keeps every [`PreparedText`](crate::PreparedText)
    /// already produced in the frame valid.
    pub fn begin_frame(&mut self) {
        self.frame = self.frame.wrapping_add(1);
        if self.needs_space || self.under_pressure() {
            if let Some(victim) = self.lru_stale_page() {
                self.evict_page(victim);
            }
        }
    }

    /// `true` when the page budget is exhausted and every page is at least
    /// three quarters used.
    fn under_pressure(&self) -> bool {
        self.pages.len() == self.max_pages as usize
            && self.pages.iter().all(|page| {
                let used_rows = u64::from(page.cursor_y) + u64::from(page.row_height);
                used_rows * 4 >= u64::from(self.height) * 3
            })
    }

    /// `true` when an entry was placed or used during the current frame.
    fn in_use_this_frame(&self) -> bool {
        self.entries
            .values()
            .any(|entry| entry.last_used == self.frame)
    }

    /// Mark an entry as recently used (updates its LRU timestamp and its page's).
    pub fn touch(&mut self, key: &CacheKey) {
        let frame = self.frame;
        if let Some(entry) = self.entries.get_mut(key) {
            entry.last_used = frame;
            if let Some(page) = self.pages.get_mut(entry.page as usize) {
                page.last_used = frame;
            }
        }
    }

    /// Number of entries not accessed within `max_age` frames.
    pub fn stale_count(&self, max_age: u64) -> usize {
        self.entries
            .values()
            .filter(|entry| self.is_stale(entry, max_age))
            .count()
    }

    /// Evict every page holding an entry not accessed within `max_age` frames.
    ///
    /// Only those pages are emptied; other pages keep their entries. Returns the
    /// number of stale entries found. The generation is bumped once when any
    /// page is evicted, so the renderer can detect the invalidation.
    pub fn evict_stale(&mut self, max_age: u64) -> usize {
        let stale = self.stale_count(max_age);
        let victims: Vec<usize> = (0..self.pages.len())
            .filter(|&index| self.page_has_stale(index, max_age))
            .collect();
        if let Some(&first) = victims.first() {
            for &index in &victims {
                self.drop_page(index);
            }
            self.evictions += victims.len() as u64;
            self.active = first;
            self.generation = self.generation.wrapping_add(1);
            self.settle_needs_space();
        }
        stale
    }

    /// Looks up `key` and marks it used in the current frame.
    pub(crate) fn lookup(&mut self, key: &CacheKey) -> Option<AtlasEntry> {
        let entry = self.entries.get(key).copied()?;
        self.touch(key);
        Some(entry)
    }

    pub(crate) fn insert(
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
        let (page, placement) = self.place(width, height)?;
        self.pages[page] = placement.next;
        self.pages[page].last_used = self.frame;
        self.active = page;
        let entry = AtlasEntry {
            id: self.next_id,
            page: page as u32,
            x: placement.x,
            y: placement.y,
            width,
            height,
            left,
            top,
            format,
            last_used: self.frame,
        };
        self.next_id = self.next_id.wrapping_add(1);
        self.entries.insert(key, entry);
        uploads.push(AtlasUpload {
            page: entry.page,
            x: entry.x,
            y: entry.y,
            width,
            height,
            format,
            pixels,
        });
        Some(entry)
    }

    /// Chooses a page for a `width` × `height` glyph and its placement there.
    /// Order: active page, other open pages ascending, a new page while the
    /// budget allows, then eviction when no glyph is in use this frame.
    fn place(&mut self, width: u32, height: u32) -> Option<(usize, Placement)> {
        if let Some(found) = self.place_on_open_pages(width, height) {
            return Some(found);
        }
        if self.pages.len() < self.max_pages as usize {
            self.pages.push(Page::default());
            let index = self.pages.len() - 1;
            return self
                .try_place(index, width, height)
                .map(|placement| (index, placement));
        }
        self.place_after_eviction(width, height)
    }

    fn place_on_open_pages(&self, width: u32, height: u32) -> Option<(usize, Placement)> {
        std::iter::once(self.active)
            .chain((0..self.pages.len()).filter(|&index| index != self.active))
            .find_map(|index| {
                self.try_place(index, width, height)
                    .map(|placement| (index, placement))
            })
    }

    fn place_after_eviction(&mut self, width: u32, height: u32) -> Option<(usize, Placement)> {
        // Clearing now would invalidate glyphs already handed out this frame,
        // so defer to the next frame boundary instead.
        if self.in_use_this_frame() {
            self.needs_space = true;
            return None;
        }
        let victim = self.lru_stale_page()?;
        self.evict_page(victim);
        self.try_place(victim, width, height)
            .map(|placement| (victim, placement))
    }

    /// Pure shelf placement on page `index`. Returns the position and the
    /// page state after placing it; nothing is mutated, so a `None` result
    /// leaves the cursors exactly as they were.
    fn try_place(&self, index: usize, width: u32, height: u32) -> Option<Placement> {
        let page = *self.pages.get(index)?;
        let (mut cursor_x, mut cursor_y, mut row_height) =
            (page.cursor_x, page.cursor_y, page.row_height);
        if u64::from(cursor_x) + u64::from(width) > u64::from(self.width) {
            cursor_x = 0;
            cursor_y = cursor_y.saturating_add(row_height.saturating_add(1));
            row_height = 0;
        }
        if u64::from(cursor_y) + u64::from(height) > u64::from(self.height) {
            return None;
        }
        Some(Placement {
            x: cursor_x,
            y: cursor_y,
            next: Page {
                cursor_x: cursor_x.saturating_add(width).saturating_add(1),
                cursor_y,
                row_height: row_height.max(height),
                last_used: page.last_used,
            },
        })
    }

    /// Least recently used page holding an entry idle for more than
    /// [`STALE_AGE_FRAMES`]. Ties go to the lowest index.
    fn lru_stale_page(&self) -> Option<usize> {
        (0..self.pages.len())
            .filter(|&index| self.page_has_stale(index, STALE_AGE_FRAMES))
            .min_by_key(|&index| (self.pages[index].last_used, index))
    }

    fn page_has_stale(&self, index: usize, max_age: u64) -> bool {
        let page = index as u32;
        self.entries
            .values()
            .any(|entry| entry.page == page && self.is_stale(entry, max_age))
    }

    fn is_stale(&self, entry: &AtlasEntry, max_age: u64) -> bool {
        self.frame.saturating_sub(entry.last_used) > max_age
    }

    /// Empties one page and re-targets it, bumping the generation once.
    fn evict_page(&mut self, index: usize) {
        self.drop_page(index);
        self.active = index;
        self.evictions += 1;
        self.generation = self.generation.wrapping_add(1);
        self.settle_needs_space();
    }

    /// Removes the entries on `index` and resets its cursors. Does not touch
    /// the generation or the eviction count.
    fn drop_page(&mut self, index: usize) {
        let page = index as u32;
        self.entries.retain(|_, entry| entry.page != page);
        self.pages[index] = Page::default();
    }

    /// A deferred eviction request stays pending only while a stale page remains.
    fn settle_needs_space(&mut self) {
        if self.lru_stale_page().is_none() {
            self.needs_space = false;
        }
    }
}

#[cfg(test)]
mod tests;
