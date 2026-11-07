# ROADMAP 010 — Report

Branch `sdk/roadmap-010-layout-id-mirror` on master `2ccdd15`. Windows 10, Rust 1.88.0 MSVC.

## Execution mode (actual)

| Role | Model | Work |
|---|---|---|
| Lead | Opus 5.5 | Root-cause analysis of the 009 cell-bounds gap, brief, gates, docs |
| P1 team-rust-engineer | Haiku 5.5 (high) | Walk mirrors layout IDs; fix round after review |
| Reviewer team-independent-reviewer | Sonnet 5.5, read-only | APPROVE_WITH_FIXES (0 CRITICAL/HIGH, 1 MEDIUM, 4 LOW) |

## Results

- `crates/acme-accessibility/src/{lib.rs,structure.rs}`; tests split into
  `structure/{tests,tests_support,tests_layout_ids}.rs` (all < 800 lines); `walk` and
  `grid_row_ids` split into functions < 50 lines.
- RED before implementation: 4 new tests failed (tree item read formula instead of layout
  row; widget after a tree/table read a wrong rect; retained cell had no rect; ragged rows
  grouped as [2,2,2] instead of [2,1,3]).
- Review fix round: tail-after-Dialog/VirtualList/Popover, Tooltip-in-cell and no-column
  table tests on sequential, bound and unbound paths; mutation RED (deleting the
  `Dialog | VirtualList` or `Popover index > 0` lowering arm fails them).
- Reviewer probe (scratch crate outside the repo): 17 widget scenarios × 4 paths, 68/68
  tail-widget rects matched, no duplicate or dangling AccessKit IDs.
- Changed existing tests: `malformed_cell_count_falls_back_to_flat_cells` →
  `rows_changed_after_build_fall_back_to_flat_cells` (ragged rows now group);
  bound tree-item bounds test now expects layout rects instead of the 009 formula.
- `acme-widgets` unchanged; no public API added.

## Verification (lead, fresh)

- `cargo test --locked --workspace`: 1492 passed, 0 failed.
- `cargo fmt --check`, `cargo clippy -p acme-accessibility --all-targets -D warnings`,
  `check_changed_rustfmt.py`, `check_changed_clippy.py`: pass.

## Open / NOT_RUN

- Bound-path Click on a virtual node still resolves (0,0) (no action advertised).
- Grids mutated after `build()` stay flat with no cell rects.
- DataGrid merged-cell spans; multi-page glyph atlas; tree-item content widgets/actions.
- Screen reader, Gallery visual, IME, DPI, GPU: NOT_RUN (manual).
