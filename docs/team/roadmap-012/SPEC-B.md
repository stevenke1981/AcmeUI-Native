# ROADMAP 012 B — bounded atlas demand eviction

Owner: P2 (Codex); Lead: p3. Base: `271e5e552c6f02d0856ca8fdd7b0a502c6eb8a09`.
A is green before B starts; this task explicitly authorizes design and implementation.
Execution: main implementer, followed by one scoped independent Rust review.

## Decisions

1. **Per-format allocators: REJECT for this batch.** Both formats currently
   share the shelf namespace, while renderers already isolate `(format, page)`
   textures. Separate allocators would change the meaning of the shared page
   budget and `page_count`, or require a new budget policy. There is no measured
   mixed-format capacity loss in this task to justify that policy expansion.
   Keep the existing format/page/upload contract and memory caps.
2. **Mid-frame eviction of untouched pages: REJECT for this batch.** Page-local
   texel safety alone is insufficient: ROADMAP 011 lifecycle tests explicitly
   require all `PreparedText` values in one frame to share `atlas_generation`.
   Eviction bumps that global generation, even when another page survives.
   Supporting this properly needs a revised invalidation contract and consumer
   audit. Preserve the guard once any entry has been handed out this frame.
3. **All-fresh starvation: DO.** A page can be recent without being needed in
   the new frame. The 60-frame threshold must not prevent demand-driven reuse
   indefinitely. Evict the least recently used page, regardless of age, only
   before any entry has been handed out this frame. A deferred overflow from
   the previous frame also permits this fallback at `begin_frame`.

## Invariants and algorithm

- No public API additions, removals, signature or layout changes.
- Successful packing, format namespace, page cap and lazy renderer allocation
  remain unchanged. Oversized glyphs fail before eviction or a pending request.
- Entries placed, looked up or touched in the current frame cannot be evicted
  automatically. `clear` and explicit `evict_stale` retain their immediate API.
- A full insert after current-frame use sets `needs_space` and returns `None`.
- Otherwise a full insert chooses the occupied page with minimum
  `(last_used, index)`, with occupancy independent of the stale-age switch.
  Demand (`max_age = 0`) has no age restriction; its callers first protect
  current-frame entries. Retry placement on that emptied page. Empty
  pages are never eviction victims, including while settling a pending request.
- At `begin_frame`, increment the frame first. A pending overflow chooses the
  same age-independent LRU victim; background pressure without an overflow
  keeps the existing stale-only policy. Evict at most one page per call.
- Each automatic victim bumps generation and eviction count exactly once;
  other pages survive. One eviction clears the pending request, even if another
  page still contains stale entries. Later automatic eviction requires a new
  overflow or stale-only background pressure; a satisfied demand cannot keep
  evicting fresh pages. Explicit stale eviction settles the request only when
  it empties at least one page; a no-victim call keeps the unmet request and
  counters unchanged. `evict_stale` returns the stale-entry count, whereas
  `page_evictions` counts emptied pages, including multiple pages in one call.
- No automatic eviction after a prepared glyph in the same frame, so generation
  remains equal across every `PreparedText` produced within that frame.
- Background pressure alone must not churn fresh pages. A true working set
  larger than the atlas can still drop glyphs within that frame; that capacity
  limit is not solved by invalidating glyphs already returned to callers.

## Exact implementation lease

- `crates/acme-text/src/atlas.rs`
- `crates/acme-text/src/atlas/tests.rs`
- `crates/acme-text/tests/atlas_lifecycle.rs`
- This SPEC, `REPORT-P2.md` and `evidence/*` in this roadmap directory.
- A's separate lease remains `crates/acme-render-wgpu/src/**`, excluding
  `atlas_pages.rs`, `batch.rs`, `golden.rs`, and shaders.
- No changes to p1's `acme-textinput` or `acme-ui`, manifests or lockfile.

## Tests first (RED before implementation)

- Enable the ignored public lifecycle regression: fill a 64-square atlas,
  begin a new frame, then prepare `O`; it must render instead of starving.
- Synthetic font-independent test: fill two pages, touch page zero in a later
  frame, then demand a new glyph in the next frame; recent page one is the LRU
  victim, other entries survive, and upload page/generation/count are correct.
- Synthetic test: overflow when every page was used this frame; keep entries
  and uploads unchanged, then evict only one recent page at the next boundary.
- Preserve the existing no-overwrite, generation-per-frame, oversized-glyph,
  page geometry, explicit clear, stale-eviction and page-budget tests.
- Update the old pending-overflow test that deliberately waited 61 frames:
  the accepted behavior now resolves that demand at the very next boundary.
- Add a fresh-pressure-without-overflow regression: repeated boundaries do
  not evict or re-upload the fresh resident set.
- Review regression: three full pages with an old entry on a recently touched
  page; after one deferred eviction and successful retry, another fresh page
  must survive the next boundary without pressure (RED exit 101, then GREEN).
- Fix wave: pin empty-page exclusion and occupancy independent of demand age,
  a no-victim explicit eviction retaining the request, and entry/page counts.

## Renderer impacts and validation

CPU and wgpu require no B production changes: uploads still identify their
format/page and only reuse pages before any current-frame glyph is issued.
The retained per-frame generation rule and painter order stay valid.
Run `cargo test --locked -p acme-text` and clippy with `--all-targets -- -D warnings`,
plus crate formatting. Run the same gates on A's wgpu crate and check its UI
dependent; then `python scripts/verify_sdk.py --profile cpu` for actual CPU
renderer tests, including existing page isolation/preflight coverage.
Preserve RED/GREEN logs, exit codes and real counts under `evidence/`.
Real GPU drawing, device loss, GPU memory/timing and manual Gallery acceptance
are NOT_RUN. No performance or pixel-parity claim follows from headless checks.
