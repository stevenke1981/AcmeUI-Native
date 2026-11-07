# REVIEW — p2's A (wgpu split) and B (bounded atlas eviction)

Reviewer: p1 (Sigma), independent. Read-only: no source file was left modified (all
probe files removed; `git status` shows only p2's and the lead's own edits). Author
was Codex (p2). Tree reviewed: working tree at base `271e5e5` + p2's uncommitted A/B.

## Verdicts

- **A — APPROVE_WITH_FIXES.** Behaviour-neutral, API-identical, and the evidence
  reproduces — but the `include!` choice silently removes two files from the
  formatter (A1, MEDIUM) and leaves one file in a state `rustfmt` would change (A2, LOW).
- **B — APPROVE_WITH_FIXES.** The demand-LRU invariants hold and the formerly
  ignored starvation test now really passes. One spec/invariant claim is not true
  of the code (B1, MEDIUM) and one eviction-count doc/spec sentence is inaccurate
  (B2, LOW); neither causes a wrong frame today.

No CRITICAL or HIGH finding. Nothing here justifies REJECT.

## Reproduced gates (my own runs, exit codes)

- `cargo test --locked -p acme-render-wgpu` → **0**, 59 passed / 2 ignored (unit),
  0 passed / 1 ignored (doc) — matches `evidence/a-*.log`.
- `cargo clippy --locked -p acme-render-wgpu --all-targets -- -D warnings` → **0**;
  `cargo fmt -p acme-render-wgpu -- --check` → **0**.
- `cargo test --locked -p acme-text` → **0**, 35 + 6 + 4 = **45 passed**, 0 ignored;
  the starvation test is no longer `#[ignore]` (HEAD had 2 ignores, now 0).
- `cargo clippy --locked -p acme-text --all-targets -- -D warnings` → **0**;
  `cargo fmt -p acme-text -- --check` → **0**.
- `cargo doc --locked -p acme-render-wgpu --no-deps` → **0**; regenerated HTML list
  equals `evidence/a-doc-after.txt` (20 paths, sorted-identical) and that equals
  `a-doc-before.txt`.
- `evidence/p2-source-manifest.json` re-hashed against the working tree: all **23**
  files match (sha256 over both CRLF and LF normalisation), line counts match.
- p2's RED evidence is credible, not just claimed: `b-red-starvation.log` shows the
  un-ignored test failing with `atlas_full=1`, and `b-review-red.log` shows the
  self-found second-eviction bug failing before the `settle_needs_space` removal.

## A — independent verification beyond the supplied evidence

I re-derived these myself instead of trusting `a-*.txt`:

- **Function bodies:** parsed HEAD `lib.rs` and all 20 split files, comparing 64
  HEAD functions by (impl context, name) after comment/whitespace normalisation:
  0 missing, and the only 2 "diffs" are `upload: &acme_core::AtlasUpload,)` versus
  `upload: &acme_core::AtlasUpload)` — i.e. a trailing comma the formatter dropped
  when the signature moved to its own file. No body changed.
- **Test module:** all 24 test fns present, same order, 0 body diffs.
- **Type declarations:** all 9 public types (`Quad`, `Frame`, `ClippedQuad`,
  `TextRun`, `RenderStats`, `SurfaceStatus`, `SurfaceAction`, `RenderError`,
  `Renderer`) have byte-identical `pub` declarations including derives.
- **Public API as a consumer sees it:** a throw-away integration test compiled
  every public path (`acme_render_wgpu::{Quad, Frame, …, SurfaceStatus}`,
  `batch::compile_scene`, `scene_from_frame`, `golden::pixels_match`) and
  `RenderError: std::error::Error` — all resolve. No visibility was narrowed;
  `pub(crate)` items (`UploadKey`, `UploadReject`, `check_upload`, `page_runs`,
  `surface::*`, `draws::*`) were private in HEAD too.
- File sizes: largest is `batch.rs` 768 lines (untouched file), all new files
  ≤ 385 (`tests.rs`), `lib.rs` 20. Under the 800-line rule.

### A1 — MEDIUM — `include!` puts `types.rs` and `renderer/state.rs` outside rustfmt

`src/lib.rs:19-20` includes both files, so they are body text of `lib.rs`, not
modules. Consequence, reproduced: I inserted a deliberately misformatted
signature into `types.rs`; `cargo fmt -p acme-render-wgpu -- --check` still exited
**0**, while `rustfmt --check src/types.rs` exited **1** on the same content. Two
source files are therefore permanently unguarded by the crate's fmt gate (CI
`cargo fmt --check` cannot fail on them). Reverting my edit restored fmt=0.
Clippy is *not* affected: a planted `let unused_probe: u32 = 5;` in `types.rs`
made `clippy … -D warnings` exit 101, so lint coverage is intact.

Concrete failing scenario: a contributor reformats `types.rs` by hand (or an
editor reflows it); CI stays green, and the next `rustfmt` run over that file —
any tool invoked outside cargo — produces a conflicting diff with no gate to stop it.
`mod types; pub use types::*;` would close the hole but, as I verified with a
minimal two-module crate, adds `types/struct.Quad.html`-style **redirect** pages
(388-byte meta-refresh stubs) that the current doc-evidence comparison would then
see as extra HTML. So the fix is a deliberate trade (accept redirect stubs, or add
a `rustfmt --check` step over the two files), not a free change. Please record the
decision in `REPORT-P2.md`; the p2 report's stated reason ("extra rustdoc
redirects") is accurate, but the fmt blind spot is not mentioned.

### A2 — LOW — `renderer/state.rs` is not rustfmt-clean in isolation

`rustfmt --edition 2024 --check src/renderer/state.rs` reports the `use std::sync::Arc;`
ordering; `cargo fmt --check` reports nothing because of A1. Either leave it and
document A1, or fold the two files back under a real module.

### Non-findings (checked, correct)

`#![forbid(unsafe_op_in_unsafe_fn)]` still applies to the included files (they are
crate-root text). The two `include!` files carry no `mod`-level `use` isolation,
but every name they need is imported in-file, so no accidental cross-file coupling
was found. `mod tests` correctly stays `#[cfg(test)]`.

## B — invariants re-derived with my own probes

- **No glyph emitted this frame is invalidated.** Probe: 40 frames, 6 texts, a
  3-page 64² atlas, per-frame texel occupancy map over every prepared glyph. No
  two live glyphs ever shared a texel; every mid-frame `prepare` that produced
  glyphs shared one `atlas_generation`; ≤1 eviction per `begin_frame`. Also
  verified directly: prepare text A, overflow with text B in the same frame, then
  re-prepare A → same `atlas_id`s and **0 uploads** (nothing was evicted or
  re-uploaded).
- **One eviction per request.** `place_after_eviction` (`atlas.rs:392-398`) evicts
  at most one page and `evict_page` now clears `needs_space` unconditionally
  (`atlas.rs:449-455`), so the next boundary with no new demand does not churn; I
  confirmed with a scratch test that after one satisfied demand, three further
  boundaries add zero evictions (probe removed afterwards).
- **Background eviction stays stale-only.** `begin_frame` picks `max_age = 0` only
  when `needs_space`, else `STALE_AGE_FRAMES` (`atlas.rs:236-247`). Probe: with a
  full page and no demand, 59 boundaries never evicted; the 61st did. Fresh-page
  pressure without demand never evicts (`background_pressure_without_demand_keeps_fresh_pages`).
- **Renderers still receive correct uploads.** Probe over 20 frames on a 2-page
  atlas: every upload has non-zero size, `pixels.len() == w*h`, page < budget,
  region inside the page. wgpu dedups by `(page, format, x, y, w, h)`
  (`renderer/atlas.rs:45`) and creates a texture per `(format, page)`
  (`ensure_glyph_pages`), so a reused page re-uploads into the same texture and an
  evicted page's stale texels are overwritten by the new upload before the draw —
  no renderer change was needed, matching the SPEC.
- **Starvation test is genuinely enabled** (`atlas_lifecycle.rs:642`, `#[ignore]`
  removed) and passes; the old 60-frame-deferred-overflow expectation was updated
  to one boundary (`atlas/tests.rs:219-246`), consistent with decision 3.

### B1 — MEDIUM — "empty pages are never eviction victims" is not implemented as stated

`lru_page` (`atlas.rs:429-435`) selects victims via `page_has_stale`, i.e. "the page
holds at least one entry older than `max_age`". That is a *has-a-stale-entry* test,
not an *is-occupied* test. With demand (`max_age = 0`) it happens to equal "not in
use this frame", so behaviour is right; but the doc comments at `atlas.rs:427-430`
and SPEC-B's "Empty pages are never eviction victims, including while settling a
pending request" describe a guarantee the predicate does not express, and no test
pins an empty page as a would-be victim (an empty page is unreachable through the
public API today, which is why nothing fails). Concrete risk: the moment a page can
be open-but-empty (e.g. the per-format allocators deferred in decision 1, or an
explicit page-open API), `lru_page(0)` would return it and the demand retry would
"evict" a page with nothing on it — a silent no-op eviction counted in
`page_evictions` and bumping `generation`. Suggested: filter on "page has any entry"
and keep the age test as the demand/stale switch, then state the invariant in one place.

### B2 — LOW — "one eviction clears the pending request" over-claims for `evict_stale`

`evict_stale` sets `needs_space = false` only inside `if let Some(&first) = victims.first()`
(`atlas.rs:296-304`). A deferred request survives an `evict_stale(age)` call that
finds no victim, and SPEC-B line 46 ("Explicit stale eviction also settles the
request") does not say "only when it evicted something". Behaviour is defensible
(the demand is still unmet), but the invariant sentence and the code disagree;
either tighten the sentence or clear the flag unconditionally.

### B3 — LOW — `evict_stale` counts pages, `stale_count` counts entries

Unchanged from HEAD, so not a regression: `evict_stale` returns `stale_count`
(entries) while `page_evictions` grows by `victims.len()` (pages). SPEC-B keeps the
public API, so this is only worth a doc line to stop a future caller assuming the
return value equals the eviction delta.

## Deviations / limits of this review

- I did not re-run `python scripts/verify_sdk.py --profile cpu` (p2's evidence has
  it at exit 0) and did not render real GPU frames: device loss, GPU memory,
  timing, pixel parity and manual Gallery acceptance remain **NOT_RUN**.
- The two files I touched while probing (`atlas/tests.rs` scratch test, `types.rs`
  reformat) were restored; final `git status` matches the pre-review state.