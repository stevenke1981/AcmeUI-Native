# P0-002 — RuntimeTree Integration Plan

| Field | Value |
|---|---|
| ID | P0-002 |
| Title | RuntimeTree Integration — stable NodeId via RetainedTree |
| Status | Planned (design complete) |
| Owner | runtime / gallery |
| Risk | T2 Controlled |
| Depends on | none |
| Blocks | incremental layout/paint, stable accessibility IDs, focus stability |

## 1. Current Pipeline (No RuntimeTree)

### Flow today

```
Gallery::frame(FrameContext)
  ├─ description() → WidgetNode<GalleryMessage>   // full rebuild every frame
  ├─ description.to_layout_with_context(NodeId::new(1), &ctx)
  │     └─ DFS: id = *next; *next += 1
  │        Tooltip/Popover: discard wrapper id, recurse into child
  │        Tree/Table/DataGrid: allocate SYNTHETIC leaf IDs not in WidgetNode tree
  ├─ apply_gallery_styles(&mut root, w, h)
  ├─ layout.compute_with_text(&root, …)  // NEW TaffyTree every call
  ├─ accessibility.update(&description, &snapshot)  // full rebuild from ID=1
  ├─ extract_gallery_ids(&root)  // PATH-based: children[0]/[1]…
  ├─ collect_hit_regions / collect_data_widget_hits
  ├─ compute_scroll_state(snapshot, ids.scroll_view, …)
  └─ render_* → Frame  // full paint rebuild
```

### Identity reality today

- `NodeId` = **frame-local DFS counter** starting at 1
- Insert/remove/reorder shifts all subsequent IDs
- Hover/press/focus stored as indices that **drift** when tree shape changes
- `WidgetKey` exists on interactive widgets but **not used** for layout IDs
- `Label` / `Separator`: `key()` returns `None`

## 2. Target Pipeline (With RuntimeTree)

```
Gallery::frame(FrameContext)
  ├─ description() → WidgetNode<M>              // still declarative
  ├─ views = widget_to_view_forest(&description) // NEW: WidgetNode → Vec<ViewNode>
  ├─ report = retained.reconcile_roots(&views)   // acme-core::RetainedTree
  │     • mount/reuse/remove by WidgetKey under parent
  │     • stable NodeId across frames
  │     • DirtyFlags on mount & prop/kind change
  ├─ runtime.sync_from_widget(&description, &report)
  ├─ layout_root = to_layout_with_ids(&description, &IdMapper, &ctx)
  │     • NodeId from RetainedTree (no fresh DFS counter)
  ├─ if any LAYOUT|CHILDREN dirty:
  │     snapshot = layout.compute_with_text(...)
  │   else:
  │     reuse previous LayoutSnapshot
  ├─ if any SEMANTICS dirty: accessibility.update(...)
  ├─ if any PAINT dirty: rebuild Frame commands for dirty subtrees
  └─ clear dirty flags for processed nodes
```

### Identity rules (end state)

1. **Stable id source**: only `RetainedTree` allocates `NodeId`
2. **Keyed identity**: sibling uniqueness by `WidgetKey`
3. **Shared id space**: layout, hit-test, focus, accessibility, paint all use the same `NodeId`
4. **Synthetic nodes** (tree rows, table cells) are first-class ViewNodes with keys
5. **Kind change** on same key: treat as props update + dirty STYLE|LAYOUT|PAINT|SEMANTICS

## 3. Key Design Decisions

### D1 — Two layers, not one mega-struct

| Layer | Crate | Role |
|---|---|---|
| `RetainedTree` / `Node` / `ViewNode` | `acme-core` | Identity + parent/children + dirty + kind |
| `RuntimeTree<M>` / `RuntimeNode<M>` | `acme-widgets` | Holds widget props, maps to layout |

### D2 — Auto-keys for currently unkeyed variants

- `Label`, `Separator`: auto-key by `format!("__auto:{path}:{index}:{kind}")`
- Phase 2+: add optional `Label::key` / `Separator::key`

### D3 — Synthetic layout children

Tree/Table/DataGrid expand into ViewNode children with stable keys:

```
Tree key="demo_tree"
  ├─ row:{row_key} kind="tree_row"
  └─ row:{row_key} kind="tree_row"
```

### D4 — DirtyFlags ownership

| Flag | Set when | Consumed by |
|---|---|---|
| CHILDREN | mount/remove/reorder | layout structure, hit index |
| LAYOUT | size/text/style geometry (propagates up) | LayoutEngine |
| PAINT | color/text/visual (no ancestor prop) | frame builder |
| SEMANTICS | name/role/disabled/focusable | accessibility |
| STYLE | token/theme resolution | style resolve |

## 4. Migration Phases

### Phase 1 — Parallel shadow (no functional change)

- `crates/acme-widgets/src/view_bridge.rs`: `widget_to_view()` converts WidgetNode → ViewNode forest
- Gallery carries `RetainedTree` field; after `description()`, calls shadow reconcile
- Compare DFS IDs vs RT IDs (logging only)
- Auto-key for unkeyed Labels/Separators

### Phase 2 — Stable ID source

- `to_layout_with_runtime()` uses RetainedTree-assigned NodeIds
- No more fresh DFS counter
- Stable ID tests: reorder siblings → same NodeId; insert before → later ids unchanged
- Update Gallery path-based lookups to key-based

### Phase 3 — Dirty-driven layout

- Cache `last_snapshot`; only recompute when LAYOUT|CHILDREN dirty
- First frame always dirty; theme toggle/resize always force layout
- Gate: dual-run parity check (full vs gated produce identical rects)

### Phase 4 — Dirty-driven paint & semantics

- 4a: Skip `accessibility.update` when no SEMANTICS dirty; skip hit-region rebuild when no LAYOUT dirty
- 4b: True partial paint (harder — needs clip/layer management)

## 5. Verification Matrix

| Phase | Automated | Manual |
|---|---|---|
| 1 | bridge tests; Gallery build | open all pages, no reconcile error |
| 2 | stable id reorder/insert tests | page switch, focus chrome stable |
| 3 | parity test full vs gated layout | resize + theme, rect equality |
| 4 | golden frame / screenshot diff | hover, focus rings, pixel match |

## 6. Risks

| Risk | Mitigation |
|---|---|
| Identity drift during dual systems | Phase 1 shadow only; feature flag |
| Gallery path-indexed lookups break | Key-based lookup; keep path as debug assert |
| Unkeyed Label/Separator collisions | Auto-key + audit Gallery builders |
| Tooltip/Popover flatten mismatch | allowlist; LayoutBinding |
| Tree/Table synthetic IDs wrong | synthetic ViewNodes Phase 2 |
| Prop changes invisible to RT | props_hash or explicit mark_dirty |

## 7. File Touch Map

```
crates/acme-core/src/tree.rs              # drain_dirty helpers (Phase 3)
crates/acme-widgets/src/lib.rs            # layout-from-RT APIs
crates/acme-widgets/src/view_bridge.rs    # NEW
crates/acme-widgets/tests/*.rs            # stable id + bridge tests
crates/acme-accessibility/src/lib.rs      # incremental update (Phase 4)
apps/gallery/src/main.rs                  # own RetainedTree; gate pipeline
apps/gallery/src/rt_shadow.rs             # NEW Phase 1
apps/gallery/src/render/layout.rs         # key-based GalleryNodeIds
apps/gallery/Cargo.toml                   # feature runtime_tree
```

## 8. Suggested Implementation Order

1. Add `view_bridge` + unit tests
2. Gallery shadow reconcile behind feature flag
3. Fix any DuplicateSiblingKey in Gallery pages
4. `to_layout_with_runtime` + parity tests
5. Switch Gallery layout IDs to RT
6. Props fingerprint / mark_dirty on state changes
7. Layout gate + snapshot cache
8. Semantics/hit skip gates; then paint dirty
9. Enable feature by default; remove DFS path

## 9. Non-goals (explicit deferral)

- Persistent Taffy node handles across frames
- Replacing `description()` with retained props editing
- Changing `focused: usize` to `NodeId`
- Partial GPU command encoding
