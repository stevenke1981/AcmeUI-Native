# ROADMAP 012 — fix wave

Lead: p3. Same hard rules as BRIEF.md (no git add/commit/stash/checkout,
no Cargo edits, never cargo clean, test-first: show RED then GREEN).
F3 (fmt of `apps/gallery/src/events/ime.rs`) is already fixed by the lead.

## p2 Codex — findings from `REVIEW-AB.md` (your A/B)

- **A1 MEDIUM / A2 LOW**: replace the root `include!` of `types.rs` and
  `renderer/state.rs` with real modules (`mod types; pub use types::*;` or
  explicit re-exports) so `cargo fmt --check` covers them; rustfmt both files.
  Accept rustdoc redirect stubs: the API acceptance is that the set of public
  item pages at the crate root (`struct.*.html`, `enum.*.html`, `fn.*.html`,
  modules) is identical; redirect stubs under private-module paths are allowed.
  Re-save `a-doc-after.txt`/`a-api-after.txt` and state the rule you applied.
- **B1 MEDIUM**: `lru_page` must never pick a page with no entries; filter on
  occupancy, keep age as the stale/demand switch; add a unit test that pins it
  (construct the state through the module's test helpers if the public API
  cannot reach an empty page).
- **B2 LOW / B3 LOW**: align SPEC-B and doc comments with the code
  (`evict_stale` keeps the request when it evicts nothing; its return value
  counts entries, `page_evictions` counts pages).
Gates: test + clippy -D warnings + `fmt -- --check` for acme-render-wgpu and
acme-text; `python scripts/verify_sdk.py --profile cpu`. Append a "Fix wave"
section to `REPORT-P2.md`, then reply `FIX DONE`.

## p1 Sigma — findings from `REVIEW-D2.md` (your D2)

- **F1 MEDIUM**: backwards selection highlight is shifted twice
  (`preedit.rs` ~89/126/132): `tail_shift` already includes the prefix width.
  Reproduction is in REVIEW-D2.md. Add backwards and cross-caret selection
  tests at scales 1.0/1.25/1.5/2.0, RED then GREEN.
- **F2 LOW**: in an empty field with a placeholder, the composition is drawn in
  the placeholder (muted) colour. Use the normal/readonly text token while
  composing; keep the muted token only when the placeholder is drawn. Test it.
Gates: `cargo test --locked -p acme-textinput`, `cargo test --locked -p
acme-gallery --bins`, clippy -D warnings and `fmt -- --check` for both.
Append a "Fix wave" section to `REPORT-P1.md`, then reply `FIX DONE`.
