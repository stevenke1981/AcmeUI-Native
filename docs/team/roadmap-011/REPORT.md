# ROADMAP 011 — Report: multi-page glyph atlas

Branch `sdk/roadmap-011-multipage-atlas` on master `f6b2753`. Windows 10, Rust 1.88.0 MSVC.

## Execution mode (actual)

| Role | Model | Work |
|---|---|---|
| Lead | Opus 5.5 | Scope, SPEC adoption, wave-0 contract commit, `GlyphFormat: Hash`, whitespace accounting fix, gates, docs |
| Design team-solution-architect | Sonnet 5.5, read-only | SPEC (`SPEC.md`) |
| P1 team-rust-engineer | Haiku 5.5 (high) | acme-text allocator (`src/atlas.rs`); review fix round |
| P2 team-test-engineer | Haiku 5.5 (high) | `acme-text/tests/atlas_pages.rs` |
| P3 team-rust-engineer | Haiku 5.5 (high) | CPU per-page atlases |
| P4 team-graphics-engineer | Haiku 5.5 (high) | wgpu `atlas_pages.rs` helpers |
| P5 team-graphics-engineer | Haiku 5.5 (high) | wgpu integration (`lib.rs`, `batch.rs`) |
| P6 team-rust-engineer | Haiku 5.5 (high) | apps → `GlyphAtlas::default()`, e2e test, benchmark case |
| Reviewer team-independent-reviewer | Sonnet 5.5, read-only | APPROVE_WITH_FIXES (0 CRITICAL/HIGH, 1 MEDIUM, 3 LOW + info) |

Waves: 0 contract (`04881aa`) → 1 P1/P3/P4 parallel → 2 P2/P5 → 3 P6 + review → fixes → docs.

## Evidence

- RED before GREEN: P1 11 failing allocator tests; P3 7 failing CPU page tests; P4 19
  failing helper tests; P5 16 compile errors against the new API; P6 quick-suite missing
  case; Lead: whitespace counted as atlas-full (`left: 2, right: 0`).
- Reviewer: randomized simulation (24 configs, 21,600 frames, 562 evictions) with zero
  violations of the generation/overlap/bounds invariants; differential run against
  `f6b2753` shows identical successful placement for `GlyphAtlas::new` (divergence only on
  the intended failed-placement fix); 15 mutants of `atlas.rs`, 12 killed. The 3 survivors
  were pinned by new tests in the fix round (each shown failing under its mutant).
- P2 600-frame rotation: 1 page → 9 evictions, 52 dropped; 4 pages → 0 evictions, 0 dropped.
- Benchmark `text/atlas_pressure_multipage` (`--quick`, this host): median 14.20 ms;
  1 page: 2 evictions, 986 drawn; 4 pages: 3 pages, 0 evictions, 1584 drawn (all
  non-space glyphs). Font-dependent; no speed claim across hosts.
- CPU e2e (`acme-textinput/tests/multipage_e2e.rs`): a 64² × 4-page atlas draws from page 1
  and matches a single 2048² page pixel-for-pixel.
- Goldens unchanged.

## Verification (lead, fresh, HEAD before docs commit)

- `cargo test --locked --workspace`: 1553 passed, 0 failed.
- `check_changed_rustfmt.py`, `check_changed_clippy.py`: pass. `cargo check` workspace,
  `acme-ui --all-features`, `acme-platform --no-default-features`: pass. Cargo.lock changed
  only by the `acme-render-cpu` dev-dependency of `acme-textinput`.
- `python scripts/verify_sdk.py --profile cpu`: all PASS.
- `examples/sdk-consumer`: `cargo test` 13 passed (P6).

## Accepted deviations

- `GlyphFormat` gained `Hash` (needed for the page-aware upload key).
- `AtlasPage.texture` is `pub(crate)` so `lib.rs` can write uploads.
- Glyphs with `atlas_page >= 8` are skipped by wgpu (CPU samples them as transparent).
- Page edges are clamped to 2048 (review MEDIUM): `GlyphAtlas::new(4096, 4096)` yields a
  2048 page instead of uploads that both renderers reject.

## Open / NOT_RUN

- Real GPU texture creation, multi-page drawing, device-loss recovery with live pages,
  GPU memory and timing: NOT_RUN (no GPU; the fallback adapter segfaults on this host).
- All-fresh starvation still drops glyphs; per-format allocators; mid-frame eviction of
  untouched pages; wgpu `lib.rs` size (~1800 lines).
- IME, DPI, screen reader, Gallery visual: NOT_RUN (manual).
