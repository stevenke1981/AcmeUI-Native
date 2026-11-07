# ROADMAP 003 — Retained-tree IDs in the real app path (BATCH-003)

Base: branch `sdk/roadmap-003-retained-ids` from master `896f4d6`. Rust 1.88.0
pinned, Windows MSVC host. Read first: `AGENTS.md`,
`docs/plans/P0-002_runtimetree_integration.md` (Phase 2 is this batch),
`docs/agent-team/ROADMAP.md` (row 003), and the committed contract
`crates/acme-widgets/src/retained_binding.rs`.

## Problem (OBSERVED)

- Gallery reconciles a `RetainedTree` every frame but only as a shadow
  (`apps/gallery/src/rt_shadow.rs`); nothing consumes its IDs.
- Layout IDs are a fresh DFS counter per frame
  (`WidgetNode::to_layout_alloc_with_context`, `acme-widgets/src/lib.rs`), so an
  insert/remove/reorder shifts the ID of every later widget.
- Accessibility assigns its own sequential IDs per frame
  (`acme-accessibility/src/lib.rs` `update`, TODO at line ~91) and assumes they
  equal layout IDs when it looks up bounds. Screen readers see unstable IDs.

## Goal

The Gallery frame path uses retained `NodeId`s for layout, hit-testing input and
accessibility, so a widget keeps its ID across frames and reorders. Existing
public APIs keep their behaviour (`to_layout`, `to_layout_with_context`,
`AccessibilityAdapter::update` unchanged). Additive APIs only.

## Shared contract (committed, Lead-owned — do not edit)

`acme_widgets::retained_binding`:
- `bind_retained(&'a [WidgetNode<M>], &RetainedTree) -> RetainedBinding<'a>`
- `RetainedBinding::id_for(&WidgetNode<M>) -> Option<NodeId>` (address-based,
  valid only for widgets of the bound borrowed description), `len`,
  `mismatches`.
- `SyntheticIds::{new, allocate}` — IDs from `SYNTHETIC_ID_BASE = 1 << 48`;
  `is_synthetic(NodeId)`.
- `view_bridge::view_child_widgets` (crate-private) defines the pairing walk.

## Work items

| ID | Role | Write set |
|---|---|---|
| A | team-rust-engineer | `crates/acme-widgets/src/lib.rs` (layout ID refactor + new method), new `crates/acme-widgets/tests/layout_binding.rs` |
| B | team-rust-engineer | `crates/acme-accessibility/src/lib.rs` |
| C (wave 2) | team-desktop-engineer | `apps/gallery/src/main.rs`, `apps/gallery/src/rt_shadow.rs`, new gallery tests |

### A — `WidgetNode::to_layout_with_binding`

Add
`pub fn to_layout_with_binding(&self, binding: &RetainedBinding<'_>, context: &WidgetLayoutContext) -> LayoutNode`.
It must produce exactly the same layout tree as
`to_layout_with_context(NodeId::new(1), context)` (same shapes, styles, measure
specs, child order) except for node IDs:
- a widget-derived node uses `binding.id_for(widget)`; when `None`, a
  `SyntheticIds` ID;
- Tooltip/Popover stay layout-transparent: the layout node is the child's (the
  current `*next -= 1` trick), so it carries the child's binding ID;
- synthetic children (Tree rows, Table/DataGrid header/row/cell nodes, any
  other `NodeId::new(*next)` allocation) use `SyntheticIds`.
Implement by refactoring the existing `to_layout_alloc_with_context` to take a
private ID-source abstraction (e.g. `enum LayoutIds<'b> { Sequential(&mut u64),
Bound { binding, synthetic } }` or a small trait) so there is ONE layout body —
do not copy the 300-line function. `to_layout_with_context` must keep
producing byte-identical results (sequential IDs, same Tooltip behaviour); the
existing tests prove it. Do not change `to_layout`/`to_layout_alloc` (the
no-context path) unless trivially sharing the same abstraction.

Tests (`tests/layout_binding.rs`, public API only):
1. Parity: for several trees (nested rows/columns, labels, buttons, tooltip,
   popover, separator, text input, Tree, Table, DataGrid, VirtualList, nav
   widgets — whatever builders exist), the bound layout and the sequential
   layout are structurally identical when IDs are ignored (compare style,
   measure, child counts recursively).
2. Stable IDs: keyed reorder/insert/remove across two reconciles keeps each
   keyed widget's layout node ID.
3. Uniqueness: all IDs in a bound layout tree are unique; non-synthetic IDs are
   exactly those in the binding.
4. Tooltip layout node carries the child's retained ID.

### B — `AccessibilityAdapter::update_with_binding`

Add `pub fn update_with_binding<M>(&mut self, root: &WidgetNode<M>, snapshot:
&LayoutSnapshot, binding: &RetainedBinding<'_>)` (and, if useful, a matching
free `build_tree_update_with_binding`). Same tree shape/roles/labels as
`update`, but each node's `AccessNodeId` and layout lookup ID is
`binding.id_for(widget)` or, when unbound, a `SyntheticIds` ID. Bounds are read
from `snapshot.get(id)` with the same ID, so they match layout produced by A.
Leave `update` and `build_tree_update` behaviour unchanged (refactor the walk
to take an ID source rather than duplicating it). Remove/replace the TODO with
an accurate note.

Tests (in the crate's test module): IDs equal binding IDs; same widget keeps
its accessibility ID across two frames after a sibling is inserted before it;
`route_action(Click(id))` resolves the center from a snapshot keyed by the
retained ID; the existing `update` tests still pass unchanged.

### C — Gallery uses the retained IDs (wave 2, after A and B land)

Replace the shadow call with: reconcile (keep error logging; on error fall back
to the old sequential path for that frame and log once), `bind_retained`,
`to_layout_with_binding`, `accessibility.update_with_binding`. Everything after
that (styles, hit regions, scroll, render) reads IDs from the bound layout tree.
Add tests proving for every Gallery page that: binding mismatches == 0; two
consecutive frames give identical layout IDs; the bound and sequential layout
snapshots give identical rect sequences in pre-order (parity).

## Hard rules

- Edit only your write set. Shared files (root manifests, `Cargo.lock`,
  `.github/**`, `docs/**`, `crates/acme-widgets/src/retained_binding.rs`,
  `view_bridge.rs`) are the Lead's; request changes as an exact patch.
- No new dependencies, no GPUI, no hardcoded theme colours, no
  platform-specific public types, no byte-offset cursors.
- No `git add/commit/stash/checkout/reset`, no `cargo clean`. Use
  `CARGO_TARGET_DIR=target/<your-id>` for all cargo commands.
- Pinned toolchain, `--locked`. Report PASS/FAIL/NOT_RUN honestly with exact
  commands and output tails. Never launch GUI apps (native runs are NOT_RUN).

## Checks per item

A: `cargo test --locked -p acme-widgets`, `cargo clippy --locked -p acme-widgets --all-targets --no-deps -- -D warnings` (report pre-existing diagnostics separately from new ones, by file:line), `rustfmt --edition 2024 --check <changed files>`.
B: same for `-p acme-accessibility`.
C: `cargo test --locked -p acme-gallery`, clippy on the gallery package, rustfmt.

## Report format

```
ID / role / files changed
Summary (≤10 lines)
Commands → status (+ output tails)
Patches requested for shared files
Risks / NOT_RUN / follow-ups
```
