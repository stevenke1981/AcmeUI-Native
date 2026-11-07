# ROADMAP 010 — Accessibility walk mirrors layout IDs (BATCH-010)

Base: branch `sdk/roadmap-010-layout-id-mirror` on master `2ccdd15` (ROADMAP 009
merged). Rust 1.88.0, Windows MSVC. Read `AGENTS.md`, then
`docs/team/roadmap-009/{BRIEF,REPORT}.md` for context.

## Problem

Layout lowering (`crates/acme-widgets/src/lib.rs`, `to_layout_alloc` and
`to_layout_alloc_with_context` with `SequentialIds` / `BoundIds`) gives Tree
rows, Table/DataGrid header leaves, the header row, cell leaves and row
containers their own IDs (sequential counter, or `SyntheticIds` in the bound
path). Cell content widgets are NOT lowered into layout. The accessibility walk
(`crates/acme-accessibility/src/{lib.rs,structure.rs}`) numbers nodes
differently, so:

- sequential path: a cell's ID points at an unrelated layout leaf (wrong rect);
  every widget after a Tree/Table/DataGrid has an ID that is off from layout;
- bound path: cells have retained IDs that layout never uses (no rect), and the
  walk's `SyntheticIds` counter drifts from layout's after any Tree/Table/
  DataGrid, so unbound widgets after them can read another node's rect.

## Goal

The walk reproduces layout's ID consumption exactly, so every accessibility
node that has a layout counterpart reads that counterpart's rect.

## Work item P1 (team-rust-engineer) — write set `crates/acme-accessibility/src/**`

1. Sequential path: at a Tree, after the tree's own ID, take one counter ID per
   `visible_nodes()` entry, in order, and use it as that TreeItem's ID. At a
   Table/DataGrid, take IDs in exactly layout's order (header leaf per column,
   header row container, then for each `rows[r]`: one per cell, then the row
   container) and use them as ColumnHeader / header Row / cell / Row IDs. Cell
   content descendants (no layout node) and any other walk-only nodes take
   `VirtualIds`. Bounds of TreeItem/ColumnHeader/Row/cell = the snapshot rect of
   that ID; keep the 009 geometric formula only as a fallback when the
   snapshot has no entry.
2. Bound path: at the same points, advance the walk's `SyntheticIds` exactly as
   `BoundIds::synthetic_id` would, and look up the bounds of TreeItem /
   ColumnHeader / Row / cell by those synthetic IDs. Node identity of virtual
   nodes stays in the `2^56` VirtualIds range (do not expose synthetic layout
   IDs as virtual node IDs in the bound path, so `route_action` and focus keep
   their 009 behavior). Retained cell widgets keep their retained IDs but read
   the rect of their cell leaf. Unbound descendants inside cell content take
   `VirtualIds` (layout has no node for them).
3. Use per-row cell counts from `Table.rows` / `DataGrid.rows` (public fields)
   to group cells, so ragged rows are grouped correctly. Fall back to the 009
   flat layout only when the flattened `children()` count differs from the sum
   of row cell counts, or there are no columns. Remove the `data_rows` caveat
   comment once ragged rows work; keep `columns.len()` as the column count.
4. Do not change `acme-widgets`. If layout's order cannot be reproduced from
   public data, stop and report exactly why.
5. Tests (RED first): for both `to_layout` + `build_tree_update` and
   `to_layout_with_binding` + `build_tree_update_with_binding`, with a real
   computed snapshot (as in 009's real-layout test):
   - every cell / row / column header / tree item rect equals its layout node's;
   - a Button placed AFTER a Tree and AFTER a Table at the same level has
     accessibility bounds equal to its layout rect (sequential path), and an
     unbound widget after them reads its own rect (bound path), if an unbound
     widget kind exists — otherwise state that;
   - ragged table rows (e.g. 1 + 3 cells, 2 columns) group by actual rows;
   - the Gallery test `accessibility_bounds_match_the_bound_layout_rects`
     still passes (`cargo test --locked -p acme-gallery`).
   Update the doc paragraph near `build_tree_update`.

## Hard rules

Same as ROADMAP 009: own write set only; no git add/commit/stash/checkout; no
new dependencies; no new public API; test-first; run
`cargo test --locked -p acme-accessibility`, `cargo clippy --locked -p
acme-accessibility --all-targets -- -D warnings`, `cargo fmt -p
acme-accessibility`, `cargo test --locked -p acme-gallery`, and
`cargo check --locked --workspace --all-targets`. Never `cargo clean`.
Report < 50 lines: files, RED, GREEN, changed existing tests and why, NOT_RUN.

Lead owns docs, CHANGELOG, todos, ROADMAP, this folder.
