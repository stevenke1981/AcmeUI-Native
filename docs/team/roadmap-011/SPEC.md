# ROADMAP 011 — Multi-page glyph atlas (SPEC)

Design: team-solution-architect (Sonnet, read-only). Adopted by Lead with the
changes marked **[Lead]**. Base: branch `sdk/roadmap-011-multipage-atlas` on
master `f6b2753`.

## 0. Findings

- `GlyphAtlas` (`crates/acme-text/src/lib.rs`) is a next-fit shelf allocator;
  any eviction is a full `clear()` (generation +1).
- `acme_core::AtlasUpload.page` exists; `GlyphDraw` has no page. Producers
  hardcode `page: 0`: `apps/gallery/src/render/text.rs`,
  `crates/acme-textinput/src/lib.rs`, `crates/acme-render-wgpu/src/batch.rs`
  (`scene_from_frame`).
- Renderers never see `generation`. GPU recovery = app calls `atlas.clear()`;
  renderers just apply uploads. Stale texels in an evicted page are harmless.
- CPU `GlyphAtlases`: one 2048² alpha + one 2048² rgba buffer;
  `validate_upload` rejects `page != 0`; `validate_scene` preflights uploads
  before mutation (keep).
- wgpu: `ATLAS_SIZE = 4096`, both formats eager (80 MiB). `text.wgsl` shared
  with the image path. Upload dedup key `(x,y,w,h)` ignores format and page —
  a real bug once pages exist.
- CI has no GPU; wgpu tests are pure helpers + one ignored smoke test.

## 1. Public contract

Core (`crates/acme-core/src/scene.rs`):

```rust
pub const ATLAS_PAGE_EDGE: u32 = 2048; // max page edge accepted by built-in renderers
pub const ATLAS_MAX_PAGES: u32 = 8;    // page index must be < this
pub struct GlyphDraw { /* existing */ pub atlas_page: u32 } // B1 (breaking for literals)
```

Text (`crates/acme-text`):

```rust
pub struct AtlasUpload   { pub page: u32, /* existing */ }       // B2
pub struct PreparedGlyph { pub atlas_page: u32, /* existing */ } // B2
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GlyphAtlasConfig { pub page_width: u32, pub page_height: u32, pub max_pages: u32 }
impl GlyphAtlasConfig { pub const MAX_PAGES_CAP: u32 = 8; pub const DEFAULT_PAGE_EDGE: u32 = 2048; pub const DEFAULT_MAX_PAGES: u32 = 4; }
impl Default for GlyphAtlasConfig { /* 2048, 2048, 4 */ }
impl GlyphAtlas {
    pub fn new(width: u32, height: u32) -> Self;          // unchanged: max_pages 1 (legacy semantics)
    pub fn with_config(config: GlyphAtlasConfig) -> Self; // clamps dims >= 1, max_pages 1..=MAX_PAGES_CAP
    pub fn page_count(&self) -> usize;                    // open pages
    pub fn max_pages(&self) -> u32;
    pub fn page_evictions(&self) -> u64;                  // cumulative; explicit clear not counted
    pub fn dimensions(&self) -> (u32, u32);               // size of ONE page
}
impl Default for GlyphAtlas { /* with_config(Default) */ }
```

- `FallbackDiagnostics` unchanged; `atlas_full_glyphs` = dropped because every
  page is full and eviction is deferred or impossible.
- Defaults: 4 pages × 2048² (worst case 80 MiB = today's eager wgpu
  allocation, same texel capacity); renderers allocate lazily per (page,
  format). Renderer cap 8 pages.
- MIGRATION.md: B1/B2 struct-literal breaks; `new` unchanged, multi-page opt-in
  via `Default`/`with_config`; renderer page edge now 2048 (wgpu 4096 removed);
  CPU error text changes.

## 2. Allocator (acme-text)

State `pages: Vec<Page { cursor_x, cursor_y, row_height, last_used }>`,
`AtlasEntry.page`, global `generation`/`frame`, new `evictions`.

- `try_place(page, w, h)` is pure (no mutation on failure). Placement rules on
  success are identical to today (single-page packing byte-identical).
- Insert: reject `w > page_w || h > page_h`; try active page, then other open
  pages ascending; else open a new page if `pages.len() < max_pages`; else
  full → eviction path.
- Success: `entry.page`, `page.last_used = frame`, upload carries `page`;
  `prepare` fills `PreparedGlyph.atlas_page`; `touch` updates page.last_used.
- Pressure (`begin_frame`): `pages.len() == max_pages` and every page has
  `used_rows*4 >= page_h*3` (identical for one page).
- Victim `lru_stale_page()`: pages holding ≥1 entry idle > 60 frames; smallest
  `page.last_used`, ties lowest index. Pages without stale entries are never
  evicted.
- `evict_page(i)`: drop its entries, reset cursors/last_used, generation +1
  once, evictions +1, `needs_space = false` if no stale candidate remains;
  other pages untouched; page stays open.
- `begin_frame`: frame +1; if `(needs_space || pressure)` and a victim exists,
  evict at most one page.
- Mid-frame full: if `in_use_this_frame()` → `needs_space = true`, return
  `None` (deferred, unchanged). If nothing in use this frame → evict
  `lru_stale_page()`, retry there, else `None`.
- `evict_stale(max_age)`: evict every page with an entry older than max_age;
  generation +1 once if any; returns stale-entry count. One page = old clear-all.
- `clear()`: reset all pages, active = 0, needs_space = false, generation +1,
  entries cleared; does not bump evictions.
- No renderer invalidation signal needed: within a frame no rect is reused.

## 3. CPU renderer

- `GlyphAtlases { alpha, rgba: Vec<Option<Vec<u8>>> }` of length
  `ATLAS_MAX_PAGES`, lazily allocated at `ATLAS_PAGE_EDGE`² × bpp.
- `validate_upload`: reject `page >= ATLAS_MAX_PAGES`; bounds against
  `ATLAS_PAGE_EDGE`; preflight unchanged. Display text of
  `InvalidGlyphUpload`: "atlas page out of range, region outside page, or
  payload size mismatch".
- `sample(format, page, x, y)`: out-of-range / unallocated page → 0.
- Tests expecting rejection of `page: 1` change to `page: ATLAS_MAX_PAGES`.
  Page-0 output unchanged; goldens unchanged.

## 4. wgpu — per-page textures + per-page bind groups (texture array rejected)

- New `crates/acme-render-wgpu/src/atlas_pages.rs`:
  - `struct AtlasPage { _texture: wgpu::Texture, bind_group: wgpu::BindGroup }`
  - `#[derive(Default)] struct AtlasPages { alpha, rgba: Vec<Option<AtlasPage>> }` (len `ATLAS_MAX_PAGES`)
  - `fn ensure(&mut self, device, layout, sampler, format: GlyphFormat, page: u32) -> &AtlasPage`
    (creates `ATLAS_PAGE_EDGE`² `R8Unorm` / `Rgba8UnormSrgb` + bind group with
    the image texture layout); `fn get(&self, format, page) -> Option<&AtlasPage>`
  - pure helpers:
    `pub(crate) fn check_upload(page: u32, origin: [u32;2], size: [u32;2], bpp: usize, pixels_len: usize) -> Result<(), UploadReject>`;
    `pub(crate) type UploadKey = (u32, GlyphFormat, u32, u32, u32, u32)`;
    `pub(crate) fn page_runs(pages: impl IntoIterator<Item = u32>) -> Vec<PageRun>`
    with `pub(crate) struct PageRun { pub page: u32, pub start: usize, pub len: usize }`.
- `lib.rs`: remove `alpha_atlas`/`rgba_atlas`/their bind groups, add
  `atlas_pages`; `build_render_resources` stops creating atlases;
  `ATLAS_SIZE` → `ATLAS_PAGE_EDGE` (uv divisor 2048); `upload_scene_atlas` uses
  `check_upload`, `UploadKey`, `ensure`; delete dead `upload_atlas` if unused;
  `text_primitive_glyphs` → `Vec<(u32, GlyphInstance)>`; `GlyphBatchDesc.page`;
  text batches split into `page_runs` (painter order, contiguous instances);
  ensure all used pages before the pass; `draw_calls` counts runs;
  `on_device_lost` resets `atlas_pages` (lazy recreate; content returns via
  app `atlas.clear()`).
- `batch.rs::scene_from_frame` maps pages. `text.wgsl` unchanged.
- Headless-testable: helpers, dedup key, runs, uv/page extraction, page
  propagation. NOT_RUN: real GPU texture creation/drawing, device loss,
  memory/timing.

## 5. Work packages

- **Wave 0 — Lead contract commit** (behavior unchanged, workspace compiles):
  constants + `GlyphDraw.atlas_page`; acme-text fields (always 0),
  `GlyphAtlasConfig`, `with_config`, `Default`, stub getters; `atlas_page: 0`
  in all literals; producers map pages; `acme-render-cpu` dev-dep of
  `acme-textinput`. **[Lead]** The optional mechanical split of acme-text is
  folded into P1 (P1 may move the atlas into `src/atlas.rs`).
- **Wave 1 (parallel):**
  - P1 allocator — `crates/acme-text/src/**` (unit tests in-crate). Existing
    `tests/atlas_lifecycle.rs` must pass unchanged.
  - P2 lifecycle integration — new `crates/acme-text/tests/atlas_pages.rs` only.
  - P3 CPU — `crates/acme-render-cpu/src/lib.rs`, `tests/regression.rs`, new
    `tests/atlas_pages.rs`.
  - P4 wgpu helpers — new `crates/acme-render-wgpu/src/atlas_pages.rs` only
    (Lead adds `mod atlas_pages;`).
- **Wave 2:** P5 wgpu integration — `crates/acme-render-wgpu/src/{lib.rs,batch.rs}`.
- **Wave 3:** P6 apps/e2e/benchmark — app `main.rs` files →
  `GlyphAtlas::default()`; sdk-consumer `with_config` snippet; new
  `crates/acme-textinput/tests/multipage_e2e.rs`; benchmark case
  `text/atlas_pressure_multipage` (+ quick_suite expected names).
  Docs (MIGRATION, BENCHMARKS, CHANGELOG, REPORT, roadmap, todos) — Lead.
- Review: independent Sonnet, read-only, reproduces eviction and preflight
  properties.

## 6. Risks and scope

- Alternating-page runs split draws (active-page-first keeps runs together).
- Victim page's hot glyphs re-upload (bounded by one page). All-fresh
  starvation still drops glyphs (documented, unchanged).
- Format waste: both formats share (page, x, y) namespace. Out of scope.
- Out of scope: mid-frame eviction of untouched pages, forced eviction after N
  overflow frames, per-format allocators, glyph reordering, texture arrays,
  renderer→app invalidation channel, GPU measurement/goldens, IME claims.
