# ROADMAP 009 — Report

Branch `sdk/roadmap-009-structural-a11y` on master `883ff47`. Windows 10, Rust 1.88.0 MSVC.

## Execution mode (actual)

| Role | Model | Work |
|---|---|---|
| Lead | Opus 5.5 | Scope, brief, integration, gates, A/B benchmark, docs |
| P1 team-rust-engineer | Haiku 5.5 (high) | Virtual TreeItem/Row/ColumnHeader nodes; fix round after review |
| P2 team-graphics-engineer | Haiku 5.5 (high) | Exact-area CPU quad coverage |
| Reviewer team-independent-reviewer | Sonnet 5.5, read-only | APPROVE_WITH_FIXES (0 CRITICAL/HIGH, 1 MEDIUM, 7 LOW) |

Separate subagent instances; the author did not review its own work.

## Results

- `acme-accessibility` (`src/structure.rs`, `src/structure/tests.rs`): 101 tests (74 baseline).
  RED was captured before implementation (13 failing new tests); the fix round used
  mutation RED (disabling each fix fails its test). Bounds of rows and headers are checked
  against a real computed layout on both sequential and retained paths.
- Deviation accepted: retained-path virtual IDs come from `2^56`, not `SyntheticIds`,
  because the walk's synthetic counter aliased layout's synthetic tree-row leaves
  (Gallery test `accessibility_bounds_match_the_bound_layout_rects` caught it).
- One 008 test renamed and changed (`table_cells_have_cell_role_and_are_the_row_children`):
  cells now sit under Row nodes.
- `acme-render-cpu` (`src/coverage.rs`, `tests/coverage.rs`): DESIGN test un-ignored;
  RED for quarter-pixel corner (0.5 → 0.25) and stroke area. `round_rect_and_border`
  corner assertion changed from 0 to a 1..=8 red range (true area 0.029; the old 0 was
  an SDF artifact). Golden `rounded_bordered.ppm`: 188 of 4096 px differ, max channel
  diff 12; other goldens byte-identical.

## Verification (lead, fresh)

- `cargo test --locked --workspace`: 1483 passed, 0 failed.
- `check_changed_rustfmt.py`, `check_changed_clippy.py` (BASE_SHA = merge-base): pass;
  `cargo clippy -p acme-accessibility -p acme-render-cpu --all-targets -D warnings`: pass.
- `cargo check --locked -p acme-ui --all-targets --all-features`: pass; Cargo.lock unchanged.
- A/B `acme-benchmark --quick` (release; base built in a worktree at `883ff47`), median:
  quads ×1.0 110.2 → 112.6 ms; ×2.0 436.7 → 394.0 ms. Within run-to-run noise
  (~±10–15 %); no speed claim.

## Open / NOT_RUN

- Cell bounds: sequential cell IDs can read a different layout leaf; retained cells have no
  layout entry (pre-existing ID-alignment gap).
- Tree-item content widgets not walked; no Focus/Click on virtual nodes; no merged-cell spans.
- Clip edges still pixel-centre snapped; arc area is sampled, not analytic.
- Multi-page glyph atlas not started.
- Screen reader, Gallery visual check, IME, DPI, GPU: NOT_RUN (manual).
