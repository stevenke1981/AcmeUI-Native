//! Allocator contract of the multi-page glyph atlas (ROADMAP 011 P1, SPEC §2).
//!
//! Synthetic cache keys and pixel-free glyphs keep these tests independent of
//! host fonts. Page geometry of 5x3 holds exactly two 2x2 glyphs, which makes
//! the expected coordinates easy to derive by hand.

use super::*;

/// Idle age after which an entry counts as stale (`STALE_AGE_FRAMES`).
const STALE: u64 = 60;

fn key(glyph: u16) -> CacheKey {
    CacheKey::new(
        Default::default(),
        glyph,
        16.0,
        (0.0, 0.0),
        cosmic_text::CacheKeyFlags::empty(),
    )
    .0
}

fn glyph(width: u32, height: u32) -> RasterizedGlyph {
    RasterizedGlyph {
        width,
        height,
        left: 0,
        top: 0,
        format: AtlasFormat::Alpha8,
        pixels: vec![255; (width * height) as usize],
    }
}

fn paged(width: u32, height: u32, max_pages: u32) -> GlyphAtlas {
    GlyphAtlas::with_config(GlyphAtlasConfig {
        page_width: width,
        page_height: height,
        max_pages,
    })
}

fn put(
    atlas: &mut GlyphAtlas,
    id: u16,
    width: u32,
    height: u32,
    uploads: &mut Vec<AtlasUpload>,
) -> Option<AtlasEntry> {
    atlas.insert(key(id), glyph(width, height), uploads)
}

fn has(atlas: &GlyphAtlas, id: u16) -> bool {
    atlas.entries.contains_key(&key(id))
}

fn page_of(atlas: &GlyphAtlas, id: u16) -> Option<u32> {
    atlas.entries.get(&key(id)).map(|entry| entry.page)
}

fn age(atlas: &mut GlyphAtlas, frames: u64) {
    for _ in 0..frames {
        atlas.begin_frame();
    }
}

#[test]
fn full_page_opens_page_one_and_uploads_to_page_one() {
    let mut atlas = paged(5, 3, 2);
    let mut uploads = Vec::new();
    put(&mut atlas, 1, 2, 2, &mut uploads).unwrap();
    put(&mut atlas, 2, 2, 2, &mut uploads).unwrap();

    let third = put(&mut atlas, 3, 2, 2, &mut uploads).unwrap();

    assert_eq!((third.page, third.x, third.y), (1, 0, 0));
    assert_eq!(
        uploads.iter().map(|upload| upload.page).collect::<Vec<_>>(),
        vec![0, 0, 1],
        "uploads must carry the page they were placed on"
    );
    assert_eq!(atlas.page_count(), 2);
    assert_eq!(atlas.generation(), 0, "opening a page is not an eviction");
}

#[test]
fn max_pages_caps_open_pages_and_drops_glyph_when_every_page_is_in_use() {
    let mut atlas = paged(5, 3, 2);
    let mut uploads = Vec::new();
    for id in 1..=4 {
        put(&mut atlas, id, 2, 2, &mut uploads).unwrap();
    }

    let dropped = put(&mut atlas, 5, 2, 2, &mut uploads);

    assert!(
        dropped.is_none(),
        "every page holds glyphs used this frame, so the glyph is deferred"
    );
    assert_eq!(atlas.page_count(), 2, "the page budget is 2");
    assert_eq!(atlas.generation(), 0);
    assert_eq!(uploads.len(), 4);
    assert!(!has(&atlas, 5));
}

#[test]
fn failed_placement_leaves_cursors_untouched() {
    // A 3x3 glyph cannot fit next to the 2x2 glyph: the wrapped row would start
    // at y=3, which leaves no room in a 3-pixel-high page.
    let mut with_failure = paged(5, 3, 1);
    let mut clean = paged(5, 3, 1);
    let mut uploads = Vec::new();
    put(&mut with_failure, 1, 2, 2, &mut uploads).unwrap();
    put(&mut clean, 1, 2, 2, &mut uploads).unwrap();
    assert!(put(&mut with_failure, 2, 3, 3, &mut uploads).is_none());

    let after_failure = put(&mut with_failure, 3, 1, 1, &mut uploads).unwrap();
    let undisturbed = put(&mut clean, 3, 1, 1, &mut uploads).unwrap();

    assert_eq!(
        (after_failure.page, after_failure.x, after_failure.y),
        (undisturbed.page, undisturbed.x, undisturbed.y),
        "a failed placement must not move the shelf cursors"
    );
    assert_eq!((after_failure.x, after_failure.y), (3, 0));
}

#[test]
fn placement_prefers_active_page_then_other_pages_ascending_then_opens_new_page() {
    let mut uploads = Vec::new();

    // Active page wins over an earlier page that also has room.
    let mut active_first = paged(10, 10, 3);
    put(&mut active_first, 1, 2, 2, &mut uploads).unwrap();
    put(&mut active_first, 2, 10, 8, &mut uploads).unwrap();
    let on_active = put(&mut active_first, 3, 1, 1, &mut uploads).unwrap();
    assert_eq!(
        (on_active.page, on_active.x, on_active.y),
        (1, 0, 9),
        "the active page is tried before page 0, which also has room"
    );

    // Full active page: the next open page with room wins before a new page opens.
    let mut ascending = paged(5, 3, 3);
    put(&mut ascending, 1, 3, 2, &mut uploads).unwrap();
    put(&mut ascending, 2, 5, 3, &mut uploads).unwrap();
    assert_eq!(ascending.page_count(), 2);
    let on_page_zero = put(&mut ascending, 3, 1, 1, &mut uploads).unwrap();
    assert_eq!(
        (on_page_zero.page, on_page_zero.x, on_page_zero.y),
        (0, 4, 0)
    );
    assert_eq!(
        ascending.page_count(),
        2,
        "no third page opens while page 0 still has room"
    );
}

#[test]
fn page_eviction_removes_only_victim_entries_and_retargets_that_page() {
    let mut atlas = paged(5, 3, 2);
    let mut uploads = Vec::new();
    for id in 1..=4 {
        put(&mut atlas, id, 2, 2, &mut uploads).unwrap();
    }
    age(&mut atlas, 60);
    atlas.touch(&key(3)); // page 1 is used at frame 60; entry 4 stays stale
    age(&mut atlas, 1);

    let fresh = put(&mut atlas, 5, 2, 2, &mut uploads).unwrap();

    assert_eq!(
        (fresh.page, fresh.x, fresh.y),
        (0, 0, 0),
        "the victim page is re-targeted"
    );
    assert!(
        !has(&atlas, 1) && !has(&atlas, 2),
        "victim entries are removed"
    );
    assert_eq!(
        page_of(&atlas, 3),
        Some(1),
        "other pages keep their entries"
    );
    assert_eq!(page_of(&atlas, 4), Some(1));
    assert_eq!(atlas.page_evictions(), 1);
    assert_eq!(atlas.generation(), 1);

    let next = put(&mut atlas, 6, 2, 2, &mut uploads).unwrap();
    assert_eq!(
        (next.page, next.x),
        (0, 3),
        "the re-targeted page is the active page"
    );
}

#[test]
fn victim_is_the_stale_page_with_the_oldest_last_use_not_the_lowest_index() {
    let mut atlas = paged(5, 3, 2);
    let mut uploads = Vec::new();
    for id in 1..=4 {
        put(&mut atlas, id, 2, 2, &mut uploads).unwrap();
    }
    age(&mut atlas, 10);
    atlas.touch(&key(1)); // page 0 last used at frame 10; page 1 at frame 0
    age(&mut atlas, 51);

    let fresh = put(&mut atlas, 5, 2, 2, &mut uploads).unwrap();

    assert_eq!(fresh.page, 1, "page 1 has the oldest last use");
    assert!(!has(&atlas, 3) && !has(&atlas, 4));
    assert!(has(&atlas, 1) && has(&atlas, 2), "page 0 is untouched");
    assert_eq!(atlas.page_evictions(), 1);
    assert_eq!(atlas.generation(), 1);
}

#[test]
fn begin_frame_evicts_at_most_one_page_and_bumps_generation_once_per_call() {
    let mut atlas = paged(5, 3, 2);
    let mut uploads = Vec::new();
    for id in 1..=4 {
        put(&mut atlas, id, 2, 2, &mut uploads).unwrap();
    }
    age(&mut atlas, 60);
    assert_eq!(atlas.generation(), 0, "no entry is stale at age 60");
    atlas.touch(&key(3));
    assert!(
        put(&mut atlas, 9, 5, 3, &mut uploads).is_none(),
        "precondition: overflow in use sets the deferred eviction request"
    );

    atlas.begin_frame(); // frame 61: both pages are stale, page 0 has the tie-break
    assert_eq!(atlas.generation(), 1);
    assert!(!has(&atlas, 1) && !has(&atlas, 2), "page 0 is the victim");
    assert!(
        has(&atlas, 3) && has(&atlas, 4),
        "page 1 survives this call"
    );

    atlas.begin_frame(); // frame 62: one eviction settled the pending demand
    assert_eq!(atlas.generation(), 1);
    assert!(has(&atlas, 3) && has(&atlas, 4));
}

#[test]
fn mid_frame_overflow_defers_eviction_and_keeps_glyphs_already_handed_out() {
    let mut atlas = paged(5, 3, 2);
    let mut uploads = Vec::new();
    for id in 1..=3 {
        put(&mut atlas, id, 2, 2, &mut uploads).unwrap();
    }
    age(&mut atlas, 61);

    put(&mut atlas, 4, 2, 2, &mut uploads).unwrap();
    assert!(
        put(&mut atlas, 5, 5, 3, &mut uploads).is_none(),
        "overflow mid-frame is deferred"
    );
    assert_eq!(atlas.generation(), 0, "no eviction while glyphs are in use");
    assert!(
        has(&atlas, 3) && has(&atlas, 4),
        "glyphs handed out survive"
    );
    assert_eq!(page_of(&atlas, 4), Some(1));

    atlas.begin_frame(); // frame 62: the stale page 0 is evicted
    assert_eq!(atlas.generation(), 1);
    assert!(!has(&atlas, 1) && !has(&atlas, 2));
    assert!(has(&atlas, 3) && has(&atlas, 4), "page 1 keeps its entries");

    let retry = put(&mut atlas, 5, 5, 3, &mut uploads).unwrap();
    assert_eq!((retry.page, retry.x, retry.y), (0, 0, 0));
}

#[test]
fn evict_stale_evicts_only_pages_holding_stale_entries() {
    let mut atlas = paged(5, 3, 2);
    let mut uploads = Vec::new();
    for id in 1..=3 {
        put(&mut atlas, id, 2, 2, &mut uploads).unwrap();
    }
    age(&mut atlas, 61);
    atlas.touch(&key(3));

    assert_eq!(atlas.evict_stale(STALE), 2, "entries 1 and 2 are stale");
    assert_eq!(
        atlas.generation(),
        1,
        "one generation bump for the whole call"
    );
    assert!(!has(&atlas, 1) && !has(&atlas, 2));
    assert!(has(&atlas, 3), "page 1 holds no stale entry");
    assert_eq!(atlas.page_count(), 2, "evicted pages stay open");
    assert_eq!(atlas.page_evictions(), 1, "one page was evicted");

    assert_eq!(atlas.evict_stale(STALE), 0);
    assert_eq!(atlas.generation(), 1, "nothing stale, so no bump");
}

#[test]
fn clear_is_immediate_keeps_pages_open_and_is_not_counted_as_eviction() {
    let mut atlas = paged(5, 3, 2);
    let mut uploads = Vec::new();
    for id in 1..=3 {
        put(&mut atlas, id, 2, 2, &mut uploads).unwrap();
    }

    atlas.clear();

    assert!(atlas.is_empty());
    assert_eq!(atlas.generation(), 1);
    assert_eq!(atlas.page_count(), 2, "pages stay open after clear");
    assert_eq!(
        atlas.page_evictions(),
        0,
        "explicit clear is not an eviction"
    );
    let again = put(&mut atlas, 1, 2, 2, &mut uploads).unwrap();
    assert_eq!((again.page, again.x, again.y), (0, 0, 0));
}

#[test]
fn new_single_page_packing_matches_legacy_sequence_and_rejects_when_full() {
    let mut atlas = GlyphAtlas::new(10, 10);
    let mut uploads = Vec::new();

    let placed: Vec<(u16, u32, u32)> = [(1, 4, 3), (2, 4, 2), (3, 1, 1), (4, 3, 5)]
        .into_iter()
        .map(|(id, width, height)| {
            let entry = put(&mut atlas, id, width, height, &mut uploads).unwrap();
            (id, entry.x, entry.y)
        })
        .collect();

    assert_eq!(placed, vec![(1, 0, 0), (2, 5, 0), (3, 0, 4), (4, 2, 4)]);
    assert!(
        put(&mut atlas, 5, 5, 2, &mut uploads).is_none(),
        "the single page is full"
    );
    assert_eq!(atlas.max_pages(), 1);
    assert_eq!(atlas.page_count(), 1);
    assert_eq!(atlas.page_evictions(), 0);
    assert_eq!(atlas.generation(), 0);
    assert_eq!(atlas.dimensions(), (10, 10));
    assert!(uploads.iter().all(|upload| upload.page == 0));
}

#[test]
fn pressure_does_not_evict_while_a_new_page_can_still_open() {
    let mut atlas = paged(5, 3, 2);
    let mut uploads = Vec::new();
    // A 2x3 glyph fills the page height, so the page is under pressure.
    put(&mut atlas, 1, 2, 3, &mut uploads).unwrap();
    age(&mut atlas, 61);
    assert_eq!(
        atlas.generation(),
        0,
        "one open page of two: pressure must not evict"
    );
    assert!(has(&atlas, 1));

    let second = put(&mut atlas, 2, 5, 3, &mut uploads).unwrap();

    assert_eq!(second.page, 1, "a new page opens instead of evicting");
    assert_eq!(atlas.generation(), 0);
    assert!(has(&atlas, 1), "the stale page is not touched");
}

#[test]
fn with_config_clamps_page_budget_and_dimensions() {
    let zero = GlyphAtlas::with_config(GlyphAtlasConfig {
        page_width: 0,
        page_height: 0,
        max_pages: 0,
    });
    assert_eq!(zero.dimensions(), (1, 1));
    assert_eq!(zero.max_pages(), 1);

    let big = GlyphAtlas::with_config(GlyphAtlasConfig {
        page_width: 16,
        page_height: 8,
        max_pages: 99,
    });
    assert_eq!(big.max_pages(), GlyphAtlasConfig::MAX_PAGES_CAP);
}

#[test]
fn page_edges_are_capped_at_the_renderer_page_edge() {
    let mut atlas = GlyphAtlas::new(4096, 4096);
    assert_eq!(atlas.dimensions(), (2048, 2048));
    let wide = GlyphAtlas::with_config(GlyphAtlasConfig {
        page_width: 4096,
        page_height: 1,
        max_pages: 2,
    });
    assert_eq!(wide.dimensions(), (2048, 1));

    let mut uploads = Vec::new();
    assert!(
        put(&mut atlas, 1, 2049, 1, &mut uploads).is_none(),
        "a glyph wider than the capped page is rejected"
    );
    let full = put(&mut atlas, 2, 2048, 2048, &mut uploads).unwrap();
    assert_eq!((full.page, full.x, full.y), (0, 0, 0));
    assert!(
        uploads
            .iter()
            .all(|upload| upload.width <= 2048 && upload.height <= 2048)
    );
}

#[test]
fn failed_too_tall_placement_does_not_advance_the_row_cursor() {
    // The old code wrapped the cursor before the height check failed, so a
    // later small glyph landed on the wrapped row. The pure placement keeps the
    // remaining space of the current row usable.
    let mut atlas = GlyphAtlas::new(10, 10);
    let mut uploads = Vec::new();
    put(&mut atlas, 1, 2, 2, &mut uploads).unwrap(); // (0,0); cursor x = 3, row 2
    assert!(
        put(&mut atlas, 2, 8, 9, &mut uploads).is_none(),
        "8 does not fit beside the first glyph, and the wrapped row would overflow"
    );

    let small = put(&mut atlas, 3, 2, 1, &mut uploads).unwrap();

    assert_eq!(
        (small.x, small.y),
        (3, 0),
        "the remaining space of the current row is still used"
    );
}

#[test]
fn begin_frame_does_not_evict_when_only_some_pages_are_under_pressure() {
    // Page 0 is 90% full and its entry is stale; page 1 is nearly empty.
    // Pressure needs every open page to be full, so nothing is evicted.
    let mut atlas = paged(10, 10, 2);
    let mut uploads = Vec::new();
    put(&mut atlas, 1, 10, 9, &mut uploads).unwrap(); // page 0: 9 of 10 rows used
    put(&mut atlas, 2, 1, 1, &mut uploads).unwrap(); // page 0 has no room; page 1 opens
    assert_eq!(page_of(&atlas, 2), Some(1));

    age(&mut atlas, 61);

    assert_eq!(
        atlas.generation(),
        0,
        "pressure needs every open page to be full"
    );
    assert!(has(&atlas, 1));
}

#[test]
fn settled_eviction_request_does_not_evict_again_later() {
    let mut atlas = paged(5, 3, 1);
    let mut uploads = Vec::new();
    put(&mut atlas, 1, 2, 2, &mut uploads).unwrap();
    put(&mut atlas, 2, 2, 2, &mut uploads).unwrap();
    assert!(
        put(&mut atlas, 3, 5, 3, &mut uploads).is_none(),
        "overflow while in use sets the deferred request"
    );
    age(&mut atlas, 61);
    assert_eq!(
        atlas.generation(),
        1,
        "the stale page is evicted for the pending request"
    );

    put(&mut atlas, 4, 2, 2, &mut uploads).unwrap(); // fresh at frame 61
    age(&mut atlas, 61); // frame 122: entry 4 is stale, but no request is pending

    assert_eq!(
        atlas.generation(),
        1,
        "a settled request must not evict again"
    );
    assert!(has(&atlas, 4));
}

#[test]
fn insert_refreshes_page_recency_so_an_older_page_is_the_victim() {
    // Page 0 was touched at frame 3. Page 1 was only inserted into, at frame 5,
    // and never touched again. Page 0 is the least recently used page.
    let mut atlas = paged(5, 3, 2);
    let mut uploads = Vec::new();
    put(&mut atlas, 1, 2, 2, &mut uploads).unwrap();
    put(&mut atlas, 2, 2, 2, &mut uploads).unwrap();
    put(&mut atlas, 3, 2, 2, &mut uploads).unwrap(); // opens page 1 at frame 0
    age(&mut atlas, 3);
    atlas.touch(&key(1));
    age(&mut atlas, 2);
    assert_eq!(put(&mut atlas, 4, 2, 2, &mut uploads).unwrap().page, 1);
    age(&mut atlas, 56); // frame 61: entries 2 (page 0) and 3 (page 1) are stale

    let fresh = put(&mut atlas, 9, 2, 2, &mut uploads).unwrap();

    assert_eq!(
        fresh.page, 0,
        "page 0 was used at frame 3, page 1 at frame 5"
    );
    assert!(has(&atlas, 3) && has(&atlas, 4), "page 1 survives");
    assert!(!has(&atlas, 1) && !has(&atlas, 2));
}

#[test]
fn demand_evicts_the_least_recent_page_even_when_every_entry_is_fresh() {
    let mut atlas = paged(5, 3, 2);
    let mut uploads = Vec::new();
    for id in 1..=4 {
        put(&mut atlas, id, 2, 2, &mut uploads).unwrap();
    }
    atlas.begin_frame();
    atlas.touch(&key(1));
    atlas.begin_frame();
    assert_eq!(atlas.stale_count(STALE), 0);
    assert_eq!(atlas.page_evictions(), 0);

    let inserted = put(&mut atlas, 5, 2, 2, &mut uploads)
        .expect("recent entries on an unused page must not starve a new frame");

    assert_eq!((inserted.page, inserted.x, inserted.y), (1, 0, 0));
    assert!(has(&atlas, 1) && has(&atlas, 2));
    assert!(!has(&atlas, 3) && !has(&atlas, 4));
    assert_eq!(atlas.page_count(), 2);
    assert_eq!(atlas.generation(), 1);
    assert_eq!(atlas.page_evictions(), 1);
    assert_eq!(uploads.len(), 5);
    assert_eq!(uploads.last().unwrap().page, 1);
}

#[test]
fn deferred_demand_evicts_one_fresh_page_at_the_next_boundary() {
    let mut atlas = paged(5, 3, 2);
    let mut uploads = Vec::new();
    for id in 1..=3 {
        put(&mut atlas, id, 2, 2, &mut uploads).unwrap();
    }
    assert!(put(&mut atlas, 9, 5, 3, &mut uploads).is_none());
    assert_eq!(atlas.len(), 3);
    assert_eq!(atlas.generation(), 0, "current-frame entries stay valid");
    assert_eq!(uploads.len(), 3, "a rejected insert emits no upload");

    atlas.begin_frame();

    assert_eq!(atlas.generation(), 1);
    assert_eq!(atlas.page_evictions(), 1);
    assert!(!has(&atlas, 1) && !has(&atlas, 2));
    assert!(has(&atlas, 3), "only the lowest-index LRU page is evicted");
    let retry = put(&mut atlas, 9, 5, 3, &mut uploads).unwrap();
    assert_eq!((retry.page, retry.x, retry.y), (0, 0, 0));
    atlas.begin_frame();
    assert_eq!(atlas.page_evictions(), 1, "the demand has been satisfied");
}

#[test]
fn background_pressure_without_demand_keeps_fresh_pages_resident() {
    let mut atlas = paged(3, 3, 1);
    let mut uploads = Vec::new();
    put(&mut atlas, 1, 3, 3, &mut uploads).unwrap();

    age(&mut atlas, STALE);

    assert!(has(&atlas, 1));
    assert_eq!(atlas.page_evictions(), 0);
    assert_eq!(atlas.generation(), 0);
    assert_eq!(uploads.len(), 1);
}

#[test]
fn satisfied_demand_does_not_evict_a_fresh_page_while_another_page_has_stale_entries() {
    let mut atlas = paged(5, 3, 3);
    let mut uploads = Vec::new();
    for id in 1..=6 {
        put(&mut atlas, id, 2, 2, &mut uploads).unwrap();
    }
    age(&mut atlas, 2);
    atlas.touch(&key(3));
    atlas.touch(&key(4));
    age(&mut atlas, 58);
    atlas.touch(&key(5)); // page 2 is recent but entry 6 becomes stale next frame
    assert!(put(&mut atlas, 9, 5, 3, &mut uploads).is_none());

    atlas.begin_frame();
    assert_eq!(atlas.page_evictions(), 1);
    assert_eq!(put(&mut atlas, 9, 5, 3, &mut uploads).unwrap().page, 0);
    atlas.begin_frame();

    assert_eq!(
        atlas.page_evictions(),
        1,
        "the demand was already satisfied"
    );
    assert_eq!(atlas.generation(), 1);
    assert!(has(&atlas, 3) && has(&atlas, 4), "fresh page 1 survives");
    assert!(has(&atlas, 5) && has(&atlas, 6), "no background pressure");
}

#[test]
fn lru_demand_filters_on_occupancy_without_a_stale_age_requirement() {
    let mut atlas = paged(5, 3, 2);
    let mut uploads = Vec::new();
    for id in 1..=3 {
        put(&mut atlas, id, 2, 2, &mut uploads).unwrap();
    }
    atlas.drop_page(0); // Open but empty, with the oldest page-index tie-break.
    assert_eq!(atlas.lru_page(STALE), None, "no stale entry exists");
    assert_eq!(
        atlas.lru_page(0),
        Some(1),
        "demand selects the occupied page independently of entry age"
    );
    assert_eq!(atlas.page_evictions(), 0, "selection does not evict");
    assert_eq!(atlas.generation(), 0);
    atlas.drop_page(1);
    assert_eq!(atlas.lru_page(0), None, "empty pages are never victims");
    assert_eq!(atlas.lru_page(STALE), None);
}

#[test]
fn empty_pages_are_not_lru_victims_when_occupied_entries_are_from_an_earlier_frame() {
    let mut atlas = paged(5, 3, 2);
    let mut uploads = Vec::new();
    for id in 1..=3 {
        put(&mut atlas, id, 2, 2, &mut uploads).unwrap();
    }
    atlas.drop_page(0);
    atlas.begin_frame();
    assert_eq!(atlas.lru_page(0), Some(1));
    atlas.drop_page(1);
    assert_eq!(atlas.lru_page(0), None);
}

#[test]
fn evict_stale_without_a_victim_keeps_the_deferred_request() {
    let mut atlas = paged(5, 3, 1);
    let mut uploads = Vec::new();
    for id in 1..=2 {
        put(&mut atlas, id, 2, 2, &mut uploads).unwrap();
    }
    assert!(put(&mut atlas, 3, 5, 3, &mut uploads).is_none());
    assert!(atlas.needs_space);

    assert_eq!(atlas.evict_stale(STALE), 0);

    assert!(atlas.needs_space, "the unmet demand remains pending");
    assert_eq!(atlas.page_evictions(), 0);
    assert_eq!(atlas.generation(), 0);
    atlas.begin_frame();
    assert_eq!(atlas.page_evictions(), 1);
    assert!(!atlas.needs_space);
}

#[test]
fn evict_stale_returns_entries_and_counts_evicted_pages_separately() {
    let mut atlas = paged(5, 3, 2);
    let mut uploads = Vec::new();
    for id in 1..=3 {
        put(&mut atlas, id, 2, 2, &mut uploads).unwrap();
    }
    age(&mut atlas, STALE + 1);
    assert_eq!(atlas.page_evictions(), 0, "no background pressure");

    assert_eq!(atlas.evict_stale(STALE), 3, "three stale entries");

    assert_eq!(atlas.page_evictions(), 2, "two emptied pages");
    assert_eq!(atlas.generation(), 1, "one explicit eviction call");
    assert!(atlas.is_empty());
}
