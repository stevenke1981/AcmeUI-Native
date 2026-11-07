# AcmeUI Native — STATUS

This is an implementation inventory, not a production-readiness or manual
acceptance claim. See [todos](todos.md) for tracked work. The workspace declares
Rust **1.88** MSRV and pins **1.88.0** in `rust-toolchain.toml`.

## Implemented foundations

| Component | Since | Notes |
|-----------|-------|-------|
| `acme-core` (tree, geometry, event, scene) | M1 | Keyed reorder, focus/capture, clip, event order |
| `acme-layout` (Taffy wrapper) | M1 | NodeId-based lookup, intrinsic text measure |
| `acme-text` (swash/rustybuzz shaping) | M1 | Atlas dedup, CJK/emoji; `clear()` bumps generation for recovery |
| `acme-animation` (tween engine) | M1 | Linear/ease/bounce/yoyo, delay, loop, events |
| `acme-theme` (color tokens) | M1 | V1 + V2 tokens, light/dark/high-contrast |
| `acme-platform` (event model) | M1 | Event variants, WindowId, IME caret query, GPU recovery hook |
| `acme-render-wgpu` (WGPU backend) | M3 | Persistent buffers, clip batching, pure surface-action state machine, `gpu_epoch` |
| `acme-textinput` | M4 | Cursor, selection, undo/redo, IME caret geometry, extended keys, readonly |
| `acme-widgets` (WidgetNode + submodules) | M4 | Visual states, button variants/sizes, card variants, overlay mgr |
| `acme-devtools` (inspector, metrics) | M3 | WidgetTreeDump, LayoutInspector, FrameMetrics, SurfaceStatus |

## Experimental

| Component | Notes |
|-----------|-------|
| `acme-widgets::data::Tree` | Typeahead, Arrow/Home/End, expand/collapse, visible node culling |
| `acme-widgets::data::Table` | Column resize, sort, selection, viewport virtualization, sticky header |
| `acme-widgets::data::DataGrid` | Frozen cells, merged cells, bidirectional virtualization |
| `acme-widgets::data::VirtualList` | Visible range, variable height cache, scroll anchoring |
| `acme-gallery` | 8-category nav; live Data demos (Tree/Table/DataGrid/VirtualList); Navigation demos |
| `acme-widgets::navigation` | NavRail, Sidebar, TabBar, Breadcrumb (structural; declarative selection) |
| `acme-accessibility` (adapter + tree) | Retained binding keeps widget IDs aligned with layout across frames; Gallery uses `update_with_binding`. Legacy `update` / `build_tree_update` still use per-call sequential IDs. Click/Activate use snapshot bounds; missing bounds fall back to (0,0). Screen-reader manual acceptance pending |
| Device-lost detection | `Arc<AtomicBool>` + `set_device_lost_callback` + uncaptured Internal/OOM |

## Implemented with limits

| Component | Notes |
|-----------|-------|
| Glyph atlas eviction/aging | Configurable shelf atlas; stale detection triggers a full-atlas clear, not per-glyph LRU reclamation. Automatic (overflow/pressure) eviction is deferred until `begin_frame`; overflowing glyphs may be omitted for that frame. Explicit `evict_stale()` and `clear()` act immediately and invalidate prior prepared text |
| Screenshot golden pipeline | CPU-renderer text-free pixel goldens run in CI (`acme-render-cpu/tests/golden.rs`); Gallery/text screenshot capture still not in CI |
| Full end-to-end GPU recovery on Windows | Callbacks + pure tests wired; interactive device-loss still manual — [../guides/MANUAL_VALIDATION.md](../guides/MANUAL_VALIDATION.md#a-gpu-device-loss-recovery) |

## Automated Only (manual still pending)

Human checklists (both still **NOT YET MANUALLY VALIDATED**):
[`../guides/MANUAL_VALIDATION.md`](../guides/MANUAL_VALIDATION.md)
— A. GPU device loss · B. Traditional Chinese 注音 IME.

| Component | Automated evidence | Manual still needed |
|-----------|--------------------|---------------------|
| Surface status machine | `resolve_surface_action` covers suspended/device-lost/acquire outcomes | Real GPU device loss on Windows — [checklist A](../guides/MANUAL_VALIDATION.md#a-gpu-device-loss-recovery) |
| Post-recovery text integrity | `atlas_clear_forces_reupload`; Gallery/Playground `on_gpu_recovered` clears CPU atlas | Trigger real device loss and confirm glyphs — [checklist A](../guides/MANUAL_VALIDATION.md#a-gpu-device-loss-recovery) |
| IME caret geometry | `ime_caret_area` + `resolve_ime_cursor_area` + Gallery field-relative rect | Traditional Chinese 注音 candidate placement — [checklist B](../guides/MANUAL_VALIDATION.md#b-traditional-chinese-ime-注音) |
| CJK shaping | `acme-text` shapes TC + emoji without panic | Visual glyph quality at 125/150/200% DPI |
| Multi-window routing | Unit tests for WindowId / configs | Interactive multi-window smoke |
| Theme V2 | Constructors + validation test | Visual Light/Dark/High Contrast pass |

## Not Validated

- Gallery/GPU/text screenshot golden comparison (CPU text-free pixel goldens already exist)
- Performance baseline thresholds (clean build, warm incremental, frame prep)
- `cargo-deny` / `cargo-audit` beyond the CI `audit` job (`cargo deny check bans licenses sources`, `cargo audit`)
- Manual platform acceptance beyond the automated Windows/Ubuntu/macOS CI matrix
- Compatibility below the declared Rust 1.88 MSRV (not supported or claimed)
- Manual Traditional Chinese 注音 IME — [checklist B](../guides/MANUAL_VALIDATION.md#b-traditional-chinese-ime-注音) (status: **NOT YET MANUALLY VALIDATED**)
- Manual GPU device-loss recovery on Windows — [checklist A](../guides/MANUAL_VALIDATION.md#a-gpu-device-loss-recovery) (status: **NOT YET MANUALLY VALIDATED**)

## Component inventory

`acme-ui/src/{foundations,inputs,layout,overlay,desktop,charts,mobile,browser}`
contains **199** component source modules, counting `*.rs` except `mod.rs`.
This is a source inventory, not a count of fully interactive or accessible widgets.
The category breakdown is in both READMEs; component-level maturity has not been
re-audited in this batch.
