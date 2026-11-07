# ROADMAP 009 — Structural accessibility and exact-area CPU coverage (BATCH-009)

Base: branch `sdk/roadmap-009-structural-a11y` on master `883ff47`. Rust 1.88.0
pinned, Windows MSVC host. Read `AGENTS.md` first. Baseline:
`cargo test --locked -p acme-accessibility -p acme-render-cpu` all green
(one ignored DESIGN test in `crates/acme-render-cpu/tests/dpi.rs`).

ROADMAP 008 left two automatable gaps that do not need a widget-structure
change if they are solved inside the consuming crate:

1. AccessKit has no `TreeItem`, `Row` or `ColumnHeader` nodes, because
   `Tree::children()` is empty and Table/DataGrid children are a flat
   row-major cell list.
2. CPU raster coverage of quad edges/corners uses a signed-distance
   approximation, not pixel-area coverage.

Not in scope (owner decisions / other batches): changing `acme-widgets`
structure or public types, multi-page glyph atlas (cross-crate Scene
contract), version bump, `acme-core` rename, `move_cursor_to_offset`, manual
screen-reader/IME/DPI/GPU checks.

## Hard rules (all agents)

- Edit only files in your write set. Do not run `git add/commit/stash/checkout`
  or switch branches. Do not add dependencies.
- AGENTS.md: no GPUI; no hardcoded theme colors in widgets; no byte offsets
  for text cursors; no platform-specific types in public API.
- No new public API unless unavoidable; if you add one, justify it.
- Test-first: write the failing test, show it fails (RED), then fix (GREEN).
- Run only your crate: `cargo test --locked -p <crate>`,
  `cargo clippy --locked -p <crate> --all-targets -- -D warnings`,
  `cargo fmt -p <crate>`. Never `cargo clean`.
- Report (< 60 lines): files changed, RED excerpt, GREEN command + result,
  NOT_RUN items, open questions.

## Work items

| ID | Role | Write set |
|---|---|---|
| P1 | team-rust-engineer | `crates/acme-accessibility/src/**` (new submodule files allowed) |
| P2 | team-graphics-engineer | `crates/acme-render-cpu/src/**`, `crates/acme-render-cpu/tests/**` |

Lead owns: `docs/CHANGELOG.md`, `docs/status/*`, `docs/agent-team/ROADMAP.md`,
`Cargo.toml`, `Cargo.lock`, workflows, this folder.

### P1 — Virtual structural nodes in the accessibility walk

Context: `crates/acme-accessibility/src/lib.rs` `walk` / `IdSource` / `Slot`;
`crates/acme-widgets/src/data/{tree,table,datagrid}.rs`;
`crates/acme-widgets/src/lib.rs` (`children()`, layout lowering of Tree and
Table — tree rows are 24px synthetic leaves in a column, padded by
`depth * indent`). AccessKit 0.21 source is in the cargo registry; verify
method names (`set_level`, `set_expanded`, `set_selected`, `set_row_index`,
`set_column_index`, `set_row_count`, `set_column_count`, `Role::TreeItem`,
`Role::Row`, `Role::ColumnHeader`, `Role::RowHeader`) before using them.

1. Tree: emit one `Role::TreeItem` node per `Tree::visible_nodes()` entry,
   as children of the `Tree` node, in visible order. Name = text of the
   item's `content` when it is a Label (else first Label text found in it,
   else empty). Set level = depth + 1; set expanded true/false only when the
   item has children; set selected for the `Tree.selected` key (state
   `false` explicitly on other items — tree items are selectable).
   Bounds: derive from the Tree's layout rect: x = tree.x + depth*indent,
   y = tree.y + index*24, width = tree.width - depth*indent, height = 24,
   clipped to the tree rect; no bounds when the tree has no rect.
2. Table / DataGrid: group the flat cells into `Role::Row` nodes
   (one per row, `columns.len()` cells each, row-major). Prepend a header
   `Row` holding one `Role::ColumnHeader` per column, named by the header's
   Label text. Set row/column counts on the Table/Grid node and row/column
   indices on rows and cells (header row = index 0 only if you also count it
   in row_count; pick one convention and document it). Keep the 008 rule:
   a control inside a cell keeps its own role. Row bounds = union of its
   cells' bounds when known. If `columns.len() == 0` or the cell count is
   not a multiple of the column count, fall back to the 008 flat layout.
   Selected row/cell (Table `selected_row`/`selected_cell`, DataGrid
   equivalent if any) → `set_selected(true)` on that row/cell.
3. IDs: virtual nodes have no widget. In the sequential path take the next
   counter value; in the bound path use `SyntheticIds`. IDs must be
   deterministic for an unchanged description (same frame input → same IDs)
   and must never collide with retained IDs. Do not change the IDs of
   existing widget nodes in the sequential path unless unavoidable; if
   unavoidable, state which tests changed and why.
4. Tests for both `build_tree_update` and `build_tree_update_with_binding`:
   tree items (roles, names, level, expanded, selected, order, bounds),
   collapsed children absent, table header + rows + indices, controls in
   cells keep their role, malformed cell count falls back, ID stability
   across two builds. Update the role table in the doc comment near
   `build_tree_update`.

### P2 — Exact-area coverage for CPU quads

Context: `crates/acme-render-cpu/src/lib.rs` `inside_rounded`, `for_pixels`,
quad fill/stroke paths; ignored test
`sharp_quad_corner_pixel_uses_exact_area_coverage` in `tests/dpi.rs`;
`tests/golden.rs`.

1. Replace the coverage approximation with pixel-area coverage:
   analytic for axis-aligned edges and sharp corners (product of the x and
   y overlaps), and a bounded supersample (e.g. 4×4 or 8×8 fixed grid) only
   for pixels within ~1px of a rounded-corner arc. Interior pixels must stay
   fast (coverage 1.0 without sampling).
2. Stroke (border) coverage must use the same area model
   (outer area − inner area), never negative.
3. Un-ignore the DESIGN test and make it pass. Add tests: half-pixel edge →
   0.5 area, quarter-pixel corner → 0.25, full circle (radius = half side)
   total coverage within 1% of πr², no coverage outside the rect, monotonic
   in radius, NaN/inf/zero-size still rejected as before.
4. If golden images change, regenerate only those, explain the pixel
   delta (count + max channel diff) in the report, and keep
   `cargo run --locked -p acme-render-cpu --example cpu_snapshot -- <tmp>.ppm`
   working. Measure before/after with
   `cargo run --locked --release -p acme-benchmark -- --quick` and report the
   CPU numbers (no invented targets).
